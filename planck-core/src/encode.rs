use crate::error::DecodeError;
use crate::traits::Packable;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// High-level encode/decode that handles byte-level packing.
///
/// This trait is automatically implemented for all [`Packable`] types via a blanket impl.
/// It converts ordinals to and from little-endian byte representations using the minimum
/// number of bytes needed for the type's [`RADIX`](Packable::RADIX).
///
/// # Example
///
/// ```
/// use planck_pack_core::{Packable, Pack};
///
/// // bool has RADIX = 2, so it encodes to 1 byte
/// assert_eq!(bool::byte_size(), 1);
/// assert_eq!(bool::bit_size(), 1);
///
/// let bytes = true.encode();
/// assert_eq!(bool::decode(&bytes).unwrap(), true);
/// ```
pub trait Pack: Packable {
    /// Number of bits needed to represent this type's full radix.
    ///
    /// Computed as `⌈log₂(RADIX)⌉`. Returns 0 for types with `RADIX ≤ 1`.
    fn bit_size() -> u32 {
        if Self::RADIX <= 1 {
            0
        } else {
            128 - (Self::RADIX - 1).leading_zeros()
        }
    }

    /// Number of bytes needed (byte-aligned).
    ///
    /// This is `⌈bit_size() / 8⌉`. Every value of this type encodes to exactly
    /// this many bytes.
    fn byte_size() -> usize {
        (Self::bit_size() as usize + 7) / 8
    }

    /// Encode this value to bytes (little-endian).
    ///
    /// The returned `Vec` has exactly [`byte_size()`](Self::byte_size) bytes.
    #[cfg(feature = "alloc")]
    fn encode(&self) -> Vec<u8> {
        let ord = self.to_ordinal();
        let len = Self::byte_size();
        if len == 0 {
            return Vec::new();
        }
        ord.to_le_bytes()[..len].to_vec()
    }

    /// Encode this value into a fixed-size buffer. Returns number of bytes written.
    ///
    /// The buffer must be at least [`byte_size()`](Self::byte_size) bytes long.
    /// Useful in `no_std` environments where allocation is unavailable.
    fn encode_to_buf(&self, buf: &mut [u8]) -> usize {
        let ord = self.to_ordinal();
        let len = Self::byte_size();
        let bytes = ord.to_le_bytes();
        buf[..len].copy_from_slice(&bytes[..len]);
        len
    }

    /// Decode from bytes (little-endian).
    ///
    /// Reads exactly [`byte_size()`](Self::byte_size) bytes from the front of the slice.
    /// Returns [`DecodeError::InsufficientData`] if the slice is too short.
    fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = Self::byte_size();
        if len == 0 {
            return Self::from_ordinal(0);
        }
        if bytes.len() < len {
            return Err(DecodeError::InsufficientData {
                expected: len,
                got: bytes.len(),
            });
        }
        let mut buf = [0u8; 16];
        buf[..len].copy_from_slice(&bytes[..len]);
        let ord = u128::from_le_bytes(buf);
        Self::from_ordinal(ord)
    }
}

// Blanket implementation: every Packable gets Pack for free.
impl<T: Packable> Pack for T {}
