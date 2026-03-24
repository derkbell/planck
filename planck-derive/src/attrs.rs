use syn::{Attribute, Expr, ExprLit, ExprRange, Lit, RangeLimits};

/// Parsed `#[planck(...)]` attributes on a field.
#[derive(Default)]
pub struct FieldAttrs {
    /// A range constraint like `1..=12`, yielding radix = 12 and offset = 1.
    pub range: Option<RangeConstraint>,
}

pub struct RangeConstraint {
    pub start: i128,
    pub end_inclusive: i128,
}

impl RangeConstraint {
    pub fn radix(&self) -> u128 {
        (self.end_inclusive - self.start + 1) as u128
    }
}

impl FieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = FieldAttrs::default();

        for attr in attrs {
            if !attr.path().is_ident("planck") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("range") {
                    let _eq: syn::Token![=] = meta.input.parse()?;
                    let range_expr: ExprRange = meta.input.parse()?;

                    let start = extract_int_from_expr(range_expr.start.as_deref())?;
                    let end = extract_int_from_expr(range_expr.end.as_deref())?;

                    let end_inclusive = match range_expr.limits {
                        RangeLimits::HalfOpen(_) => end - 1,
                        RangeLimits::Closed(_) => end,
                    };

                    if end_inclusive < start {
                        return Err(syn::Error::new_spanned(
                            &range_expr,
                            "planck range: end must be >= start",
                        ));
                    }

                    result.range = Some(RangeConstraint {
                        start,
                        end_inclusive,
                    });
                    Ok(())
                } else {
                    Err(meta.error("unknown planck attribute"))
                }
            })?;
        }

        Ok(result)
    }
}

fn extract_int_from_expr(expr: Option<&Expr>) -> syn::Result<i128> {
    let expr = expr.ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "planck range bounds must be explicit"))?;

    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit_int),
            ..
        }) => lit_int.base10_parse::<i128>(),
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            if let Expr::Lit(ExprLit {
                lit: Lit::Int(lit_int),
                ..
            }) = unary.expr.as_ref()
            {
                let val: i128 = lit_int.base10_parse()?;
                Ok(-val)
            } else {
                Err(syn::Error::new_spanned(expr, "expected integer literal"))
            }
        }
        _ => Err(syn::Error::new_spanned(expr, "expected integer literal")),
    }
}
