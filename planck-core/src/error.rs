use core::fmt;

/// Errors that can occur during decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The ordinal value exceeds the type's radix.
    OrdinalOutOfRange { ordinal: u128, radix: u128 },
    /// After decoding all fields, remaining ordinal was nonzero.
    ExcessData,
    /// Input bytes are too short.
    InsufficientData { expected: usize, got: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::OrdinalOutOfRange { ordinal, radix } => {
                write!(f, "ordinal {ordinal} out of range for radix {radix}")
            }
            DecodeError::ExcessData => {
                write!(f, "excess data after decoding all fields")
            }
            DecodeError::InsufficientData { expected, got } => {
                write!(f, "insufficient data: expected {expected} bytes, got {got}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}
