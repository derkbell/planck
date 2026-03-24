use crate::error::DecodeError;
use crate::traits::Packable;

// --- bool ---

impl Packable for bool {
    const RADIX: u128 = 2;

    fn to_ordinal(&self) -> u128 {
        *self as u128
    }

    fn from_ordinal(ord: u128) -> Result<Self, DecodeError> {
        match ord {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::OrdinalOutOfRange {
                ordinal: ord,
                radix: 2,
            }),
        }
    }
}

// --- Unit type ---

impl Packable for () {
    const RADIX: u128 = 1;

    fn to_ordinal(&self) -> u128 {
        0
    }

    fn from_ordinal(ord: u128) -> Result<Self, DecodeError> {
        if ord == 0 {
            Ok(())
        } else {
            Err(DecodeError::OrdinalOutOfRange {
                ordinal: ord,
                radix: 1,
            })
        }
    }
}

// --- Unsigned integers ---

macro_rules! impl_packable_unsigned {
    ($ty:ty, $radix:expr) => {
        impl Packable for $ty {
            const RADIX: u128 = $radix;

            fn to_ordinal(&self) -> u128 {
                *self as u128
            }

            fn from_ordinal(ord: u128) -> Result<Self, DecodeError> {
                if ord < $radix {
                    Ok(ord as $ty)
                } else {
                    Err(DecodeError::OrdinalOutOfRange {
                        ordinal: ord,
                        radix: $radix,
                    })
                }
            }
        }
    };
}

impl_packable_unsigned!(u8, 256);
impl_packable_unsigned!(u16, 65536);
impl_packable_unsigned!(u32, 1 << 32);
impl_packable_unsigned!(u64, 1 << 64);

// --- Signed integers ---

macro_rules! impl_packable_signed {
    ($ty:ty, $unsigned:ty, $radix:expr, $offset:expr) => {
        impl Packable for $ty {
            const RADIX: u128 = $radix;

            fn to_ordinal(&self) -> u128 {
                (*self as $unsigned) as u128
            }

            fn from_ordinal(ord: u128) -> Result<Self, DecodeError> {
                if ord < $radix {
                    Ok((ord as $unsigned) as $ty)
                } else {
                    Err(DecodeError::OrdinalOutOfRange {
                        ordinal: ord,
                        radix: $radix,
                    })
                }
            }
        }
    };
}

impl_packable_signed!(i8, u8, 256, 128);
impl_packable_signed!(i16, u16, 65536, 32768);
impl_packable_signed!(i32, u32, 1 << 32, 1 << 31);
impl_packable_signed!(i64, u64, 1 << 64, 1 << 63);

// --- Option<T> ---

impl<T: Packable> Packable for Option<T> {
    const RADIX: u128 = T::RADIX + 1;

    fn to_ordinal(&self) -> u128 {
        match self {
            None => 0,
            Some(v) => v.to_ordinal() + 1,
        }
    }

    fn from_ordinal(ord: u128) -> Result<Self, DecodeError> {
        if ord == 0 {
            Ok(None)
        } else if ord <= T::RADIX {
            Ok(Some(T::from_ordinal(ord - 1)?))
        } else {
            Err(DecodeError::OrdinalOutOfRange {
                ordinal: ord,
                radix: Self::RADIX,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_round_trip() {
        assert_eq!(bool::from_ordinal(false.to_ordinal()).unwrap(), false);
        assert_eq!(bool::from_ordinal(true.to_ordinal()).unwrap(), true);
        assert!(bool::from_ordinal(2).is_err());
    }

    #[test]
    fn u8_round_trip() {
        for v in 0..=255u8 {
            assert_eq!(u8::from_ordinal(v.to_ordinal()).unwrap(), v);
        }
        assert!(u8::from_ordinal(256).is_err());
    }

    #[test]
    fn i8_round_trip() {
        for v in i8::MIN..=i8::MAX {
            assert_eq!(i8::from_ordinal(v.to_ordinal()).unwrap(), v);
        }
    }

    #[test]
    fn option_round_trip() {
        let none: Option<bool> = None;
        assert_eq!(Option::<bool>::from_ordinal(none.to_ordinal()).unwrap(), None);
        assert_eq!(
            Option::<bool>::from_ordinal(Some(true).to_ordinal()).unwrap(),
            Some(true)
        );
        assert_eq!(Option::<bool>::RADIX, 3);
    }

    #[test]
    fn unit_round_trip() {
        assert_eq!(<()>::from_ordinal(().to_ordinal()).unwrap(), ());
        assert!(<()>::from_ordinal(1).is_err());
    }
}
