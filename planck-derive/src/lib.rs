use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attrs;
mod codegen;

/// Derive `Packable` for structs and enums.
///
/// Generates an optimal mixed-radix encoding where each field or variant contributes
/// its own radix. The total `RADIX` is the product of field radixes (for structs) or
/// the sum of variant radixes (for enums).
///
/// # Structs
///
/// ```ignore
/// use planck_pack::{Planck, Pack, Packable};
///
/// #[derive(Debug, PartialEq, Planck)]
/// struct Birthday {
///     #[planck(range = 2000..=2100)]
///     year: u16,
///     #[planck(range = 1..=12)]
///     month: u8,
///     #[planck(range = 1..=31)]
///     day: u8,
/// }
///
/// // 101 × 12 × 31 = 37,572 → 16 bits → 2 bytes
/// assert_eq!(Birthday::byte_size(), 2);
/// ```
///
/// # Enums
///
/// Unit variants have radix 1. Data-carrying variants have radix equal to the product
/// of their fields' radixes. The enum's total radix is the sum across all variants:
///
/// ```ignore
/// use planck_pack::{Planck, Packable};
///
/// #[derive(Planck)]
/// enum Color { Red, Green, Blue }
///
/// #[derive(Planck)]
/// enum Command {
///     Noop,                // radix 1
///     Paint(Color),        // radix 3
///     SetAlpha(bool),      // radix 2
/// }
///
/// // 1 + 3 + 2 = 6 — the variant tag is free
/// assert_eq!(Command::RADIX, 6);
/// ```
///
/// # Attributes
///
/// - `#[planck(range = a..=b)]` — constrain an integer field to a specific range,
///   setting its radix to `b - a + 1`. Supports both inclusive (`..=`) and exclusive (`..`) ranges.
#[proc_macro_derive(Planck, attributes(planck))]
pub fn derive_planck(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match codegen::generate(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
