use crate::error::DecodeError;

/// A type that can be packed into a mixed-radix representation.
///
/// Each implementor declares how many distinct values it can take ([`RADIX`](Self::RADIX))
/// and provides bidirectional mapping between values and ordinals in `[0, RADIX)`.
///
/// Planck uses these ordinals to encode structs as mixed-radix numbers: each field
/// becomes a digit with its own base. The total number of bits needed is
/// `⌈log₂(r₁ × r₂ × ... × rₙ)⌉`, which is always ≤ the sum of individual field bit widths.
///
/// # Derive
///
/// The easiest way to implement this trait is via `#[derive(Planck)]` from the `planck` crate.
///
/// # Manual Implementation
///
/// ```
/// use planck_core::{Packable, DecodeError};
///
/// struct DieRoll(u8); // 1-6
///
/// impl Packable for DieRoll {
///     const RADIX: u128 = 6;
///
///     fn to_ordinal(&self) -> u128 {
///         (self.0 - 1) as u128
///     }
///
///     fn from_ordinal(ord: u128) -> Result<Self, DecodeError> {
///         if ord < 6 {
///             Ok(DieRoll(ord as u8 + 1))
///         } else {
///             Err(DecodeError::OrdinalOutOfRange { ordinal: ord, radix: 6 })
///         }
///     }
/// }
///
/// assert_eq!(DieRoll(3).to_ordinal(), 2);
/// assert_eq!(DieRoll::RADIX, 6);
/// ```
pub trait Packable: Sized {
    /// The number of distinct values this type can take.
    ///
    /// For a `bool` this is 2, for an enum with 3 variants this is 3,
    /// for a `u8` constrained to `0..=10` this is 11.
    ///
    /// For structs, `RADIX` is the product of all field radixes.
    /// For enums, `RADIX` is the sum of all variant radixes.
    const RADIX: u128;

    /// Convert this value to its ordinal position in `[0, RADIX)`.
    ///
    /// The returned value must always be less than [`RADIX`](Self::RADIX).
    fn to_ordinal(&self) -> u128;

    /// Reconstruct from an ordinal. Returns `Err` if `ord >= RADIX`.
    fn from_ordinal(ord: u128) -> Result<Self, DecodeError>;
}
