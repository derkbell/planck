use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields};

use crate::attrs::FieldAttrs;

pub fn generate(input: &DeriveInput) -> syn::Result<TokenStream> {
    match &input.data {
        Data::Struct(data) => generate_struct(input, data),
        Data::Enum(data) => generate_enum(input, data),
        Data::Union(_) => Err(syn::Error::new_spanned(input, "planck does not support unions")),
    }
}

fn generate_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &data.fields {
        Fields::Named(f) => &f.named,
        Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(input, "planck does not yet support tuple structs"));
        }
        Fields::Unit => {
            // Unit struct: RADIX = 1
            return Ok(quote! {
                impl #impl_generics planck_core::Packable for #name #ty_generics #where_clause {
                    const RADIX: u128 = 1;

                    fn to_ordinal(&self) -> u128 { 0 }

                    fn from_ordinal(ord: u128) -> Result<Self, planck_core::DecodeError> {
                        if ord == 0 {
                            Ok(#name)
                        } else {
                            Err(planck_core::DecodeError::OrdinalOutOfRange { ordinal: ord, radix: 1 })
                        }
                    }
                }
            });
        }
    };

    let mut radix_parts = Vec::new();
    let mut encode_parts = Vec::new(); // built in reverse order
    let mut decode_parts = Vec::new();
    let mut field_names = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let attrs = FieldAttrs::from_attrs(&field.attrs)?;

        field_names.push(field_name.clone());

        if let Some(range) = &attrs.range {
            let radix = range.radix();
            let start = range.start;

            radix_parts.push(quote! { #radix });
            encode_parts.push(quote! {
                {
                    let val = self.#field_name as i128;
                    (val - #start) as u128
                }
            });
            decode_parts.push(quote! {
                let #field_name = {
                    let ord_val = __ord % #radix;
                    __ord /= #radix;
                    (ord_val as i128 + #start) as _
                };
            });
        } else {
            radix_parts.push(quote! { <#field_ty as planck_core::Packable>::RADIX });
            encode_parts.push(quote! {
                planck_core::Packable::to_ordinal(&self.#field_name)
            });
            decode_parts.push(quote! {
                let #field_name = {
                    let ord_val = __ord % <#field_ty as planck_core::Packable>::RADIX;
                    __ord /= <#field_ty as planck_core::Packable>::RADIX;
                    <#field_ty as planck_core::Packable>::from_ordinal(ord_val)?
                };
            });
        }
    }

    // RADIX = product of all field radixes
    let radix_expr = if radix_parts.is_empty() {
        quote! { 1u128 }
    } else {
        let mut expr = radix_parts[0].clone();
        for part in &radix_parts[1..] {
            expr = quote! { #expr * #part };
        }
        expr
    };

    // Encode: Horner's method, fields in reverse order
    // ordinal = field_n + radix_n * (field_{n-1} + radix_{n-1} * (...))
    // But since field 1 is least significant, we accumulate:
    // acc = 0; for field in reverse: acc = acc * radix_field + val_field
    let encode_body = if encode_parts.is_empty() {
        quote! { 0u128 }
    } else {
        let mut expr = quote! { 0u128 };
        for (i, enc) in encode_parts.iter().enumerate().rev() {
            let radix = &radix_parts[i];
            // For the last (outermost) iteration, we don't need the multiply
            // But for correctness in Horner's method, we always do acc * radix + val
            // except that for i=last (first in reverse), acc is 0, so it simplifies
            expr = quote! { (#expr) * #radix + #enc };
        }
        expr
    };

    Ok(quote! {
        impl #impl_generics planck_core::Packable for #name #ty_generics #where_clause {
            const RADIX: u128 = #radix_expr;

            fn to_ordinal(&self) -> u128 {
                #encode_body
            }

            fn from_ordinal(mut __ord: u128) -> Result<Self, planck_core::DecodeError> {
                #(#decode_parts)*
                if __ord != 0 {
                    return Err(planck_core::DecodeError::ExcessData);
                }
                Ok(Self { #(#field_names),* })
            }
        }
    })
}

fn generate_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(input, "planck requires at least one enum variant"));
    }

    // For each variant, compute its radix (product of field radixes) and generate
    // encode/decode logic. Total enum RADIX = sum of variant radixes.
    //
    // Encoding: ordinal = base_offset + mixed_radix_encode(fields)
    // where base_offset = sum of radixes of all preceding variants.

    struct VariantInfo {
        variant_radix_expr: TokenStream,
        from_ordinal_arm: TokenStream,
    }

    let mut variants_info: Vec<VariantInfo> = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Unit => {
                variants_info.push(VariantInfo {
                    variant_radix_expr: quote! { 1u128 },
                    from_ordinal_arm: quote! {
                        if __ord < __base + 1u128 {
                            return Ok(#name::#variant_name);
                        }
                        __base += 1u128;
                    },
                });
            }
            Fields::Named(fields_named) => {
                let fields: Vec<_> = fields_named.named.iter().collect();
                let (radix_expr, _encode_expr, decode_block, _pattern, construct) =
                    generate_variant_fields(&fields, name, variant_name, true)?;

                variants_info.push(VariantInfo {
                    variant_radix_expr: radix_expr.clone(),
                    from_ordinal_arm: quote! {
                        if __ord < __base + #radix_expr {
                            let mut __local = __ord - __base;
                            #decode_block
                            return Ok(#name::#variant_name { #construct });
                        }
                        __base += #radix_expr;
                    },
                });
            }
            Fields::Unnamed(fields_unnamed) => {
                let fields: Vec<_> = fields_unnamed.unnamed.iter().collect();
                let (radix_expr, _encode_expr, decode_block, _pattern, construct) =
                    generate_variant_fields(&fields, name, variant_name, false)?;

                variants_info.push(VariantInfo {
                    variant_radix_expr: radix_expr.clone(),
                    from_ordinal_arm: quote! {
                        if __ord < __base + #radix_expr {
                            let mut __local = __ord - __base;
                            #decode_block
                            return Ok(#name::#variant_name(#construct));
                        }
                        __base += #radix_expr;
                    },
                });
            }
        }
    }

    // RADIX = sum of all variant radixes
    let radix_exprs: Vec<_> = variants_info.iter().map(|v| &v.variant_radix_expr).collect();
    let total_radix = if radix_exprs.len() == 1 {
        radix_exprs[0].clone()
    } else {
        let first = &radix_exprs[0];
        let rest = &radix_exprs[1..];
        quote! { #first #(+ #rest)* }
    };

    let from_ordinal_arms: Vec<_> = variants_info.iter().map(|v| &v.from_ordinal_arm).collect();

    // For to_ordinal, we compute base offsets as consts per variant.
    let mut base_increments = Vec::new();
    let mut to_ordinal_arms_with_base = Vec::new();

    for (i, _) in variants_info.iter().enumerate() {
        let base_name = quote::format_ident!("__BASE_{}", i);
        if i == 0 {
            base_increments.push(quote! { const #base_name: u128 = 0; });
        } else {
            let prev_base = quote::format_ident!("__BASE_{}", i - 1);
            let prev_radix = &variants_info[i - 1].variant_radix_expr;
            base_increments.push(quote! { const #base_name: u128 = #prev_base + #prev_radix; });
        }

        // Rewrite the to_ordinal arm to use the const base
        let variant_name = &data.variants[i].ident;
        match &data.variants[i].fields {
            Fields::Unit => {
                to_ordinal_arms_with_base.push(quote! {
                    #name::#variant_name => #base_name,
                });
            }
            Fields::Named(fields_named) => {
                let field_names: Vec<_> = fields_named.named.iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                let fields: Vec<_> = fields_named.named.iter().collect();
                let (_, encode_expr, _, _, _) = generate_variant_fields(&fields, name, variant_name, true)?;
                to_ordinal_arms_with_base.push(quote! {
                    #name::#variant_name { #(#field_names),* } => {
                        #base_name + #encode_expr
                    },
                });
            }
            Fields::Unnamed(fields_unnamed) => {
                let fields: Vec<_> = fields_unnamed.unnamed.iter().collect();
                let binding_names: Vec<_> = (0..fields.len())
                    .map(|i| quote::format_ident!("__f{}", i))
                    .collect();
                let (_, encode_expr, _, _, _) = generate_variant_fields(&fields, name, variant_name, false)?;
                to_ordinal_arms_with_base.push(quote! {
                    #name::#variant_name(#(#binding_names),*) => {
                        #base_name + #encode_expr
                    },
                });
            }
        }
    }

    Ok(quote! {
        impl #impl_generics planck_core::Packable for #name #ty_generics #where_clause {
            const RADIX: u128 = #total_radix;

            fn to_ordinal(&self) -> u128 {
                #(#base_increments)*
                match self {
                    #(#to_ordinal_arms_with_base)*
                }
            }

            fn from_ordinal(__ord: u128) -> Result<Self, planck_core::DecodeError> {
                let mut __base: u128 = 0;
                #(#from_ordinal_arms)*
                Err(planck_core::DecodeError::OrdinalOutOfRange {
                    ordinal: __ord,
                    radix: Self::RADIX,
                })
            }
        }
    })
}

/// Generate encode/decode logic for a variant's fields.
/// Returns (radix_expr, encode_expr, decode_block, pattern_tokens, construct_tokens).
fn generate_variant_fields(
    fields: &[&syn::Field],
    _enum_name: &syn::Ident,
    _variant_name: &syn::Ident,
    named: bool,
) -> syn::Result<(TokenStream, TokenStream, TokenStream, TokenStream, TokenStream)> {
    if fields.is_empty() {
        return Ok((
            quote! { 1u128 },
            quote! { 0u128 },
            quote! {},
            quote! {},
            quote! {},
        ));
    }

    let mut radix_parts = Vec::new();
    let mut binding_names = Vec::new();
    let mut field_radixes = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let field_ty = &field.ty;

        let binding = if named {
            let name = field.ident.as_ref().unwrap();
            binding_names.push(quote! { #name });
            quote! { #name }
        } else {
            let name = quote::format_ident!("__f{}", i);
            binding_names.push(quote! { #name });
            quote! { #name }
        };

        if let Some(range) = &attrs.range {
            let radix = range.radix();
            let start = range.start;
            radix_parts.push(quote! { #radix });
            field_radixes.push((binding, quote! { #radix }, Some(start)));
        } else {
            radix_parts.push(quote! { <#field_ty as planck_core::Packable>::RADIX });
            field_radixes.push((binding, quote! { <#field_ty as planck_core::Packable>::RADIX }, None));
        }
    }

    // Radix = product
    let radix_expr = {
        let mut expr = radix_parts[0].clone();
        for part in &radix_parts[1..] {
            expr = quote! { #expr * #part };
        }
        expr
    };

    // Encode: Horner's method (reverse field order)
    let mut encode_expr = quote! { 0u128 };
    for (binding, radix, offset) in field_radixes.iter().rev() {
        let val_expr = if let Some(start) = offset {
            quote! { (*#binding as i128 - #start) as u128 }
        } else {
            quote! { planck_core::Packable::to_ordinal(#binding) }
        };
        encode_expr = quote! { (#encode_expr) * #radix + #val_expr };
    }

    // Decode: successive div/mod on __local
    let mut decode_stmts = Vec::new();
    for (binding, radix, offset) in field_radixes.iter() {
        if let Some(start) = offset {
            decode_stmts.push(quote! {
                let #binding = (__local % #radix) as i128 + #start;
                let #binding = #binding as _;
                __local /= #radix;
            });
        } else {
            let field_ty = &fields[decode_stmts.len()].ty;
            decode_stmts.push(quote! {
                let #binding = <#field_ty as planck_core::Packable>::from_ordinal(__local % #radix)?;
                __local /= #radix;
            });
        }
    }
    let decode_block = quote! { #(#decode_stmts)* };

    // Pattern and construct tokens
    let pattern = if named {
        let names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
        quote! { #(ref #names),* }
    } else {
        let names: Vec<_> = (0..fields.len()).map(|i| quote::format_ident!("__f{}", i)).collect();
        quote! { #(ref #names),* }
    };

    let construct = if named {
        let names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
        quote! { #(#names),* }
    } else {
        let names: Vec<_> = (0..fields.len()).map(|i| quote::format_ident!("__f{}", i)).collect();
        quote! { #(#names),* }
    };

    Ok((radix_expr, encode_expr, decode_block, pattern, construct))
}
