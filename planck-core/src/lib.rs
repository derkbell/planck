#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
mod traits;
mod encode;
mod radix;

pub use error::DecodeError;
pub use traits::Packable;
pub use encode::Pack;
