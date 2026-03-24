#![cfg_attr(not(feature = "std"), no_std)]

//! Mixed-radix bit-packing serialization for Rust.
//!
//! Planck encodes structs and enums using the theoretical minimum number of bits
//! by treating each field as a digit in a mixed-radix number system. Fields with
//! constrained ranges (enums, bounded integers, booleans) are packed together so
//! that no bits are wasted on unused values.
//!
//! # Quick Start
//!
//! Derive [`Planck`] on your types and use [`Pack::encode`] / [`Pack::decode`]:
//!
//! ```
//! use planck::{Planck, Pack, Packable};
//!
//! #[derive(Debug, PartialEq, Planck)]
//! struct Birthday {
//!     #[planck(range = 2000..=2100)]
//!     year: u16,
//!     #[planck(range = 1..=12)]
//!     month: u8,
//!     #[planck(range = 1..=31)]
//!     day: u8,
//! }
//!
//! let bday = Birthday { year: 2024, month: 3, day: 14 };
//!
//! // 3 fields packed into just 2 bytes (vs 4 bytes with raw u16 + u8 + u8)
//! let bytes = bday.encode();
//! assert_eq!(bytes.len(), 2);
//!
//! let decoded = Birthday::decode(&bytes).unwrap();
//! assert_eq!(decoded, bday);
//! ```
//!
//! # Deriving on Enums
//!
//! Enums are packed by their variant count. Data-carrying variants include their
//! payload in the radix — the discriminant is absorbed for free:
//!
//! ```
//! use planck::{Planck, Pack, Packable};
//!
//! #[derive(Debug, PartialEq, Planck)]
//! enum Color { Red, Green, Blue }  // RADIX = 3
//!
//! // Option<Color> gets RADIX = 4 (3 colors + None) — exactly 2 bits
//! assert_eq!(<Option<Color>>::RADIX, 4);
//!
//! #[derive(Debug, PartialEq, Planck)]
//! enum Command {
//!     Noop,               // radix 1
//!     SetColor(Color),    // radix 3
//!     SetLevel(bool),     // radix 2
//! }
//! // 1 + 3 + 2 = 6 total values — the variant tag costs zero extra bits
//! assert_eq!(Command::RADIX, 6);
//! ```
//!
//! # Manual Implementation
//!
//! You can implement [`Packable`] by hand for custom encoding logic:
//!
//! ```
//! use planck::{Packable, Pack, DecodeError};
//!
//! #[derive(Debug, PartialEq)]
//! enum Suit { Hearts, Diamonds, Clubs, Spades }
//!
//! impl Packable for Suit {
//!     const RADIX: u128 = 4;
//!
//!     fn to_ordinal(&self) -> u128 {
//!         match self {
//!             Suit::Hearts => 0,
//!             Suit::Diamonds => 1,
//!             Suit::Clubs => 2,
//!             Suit::Spades => 3,
//!         }
//!     }
//!
//!     fn from_ordinal(ord: u128) -> Result<Self, DecodeError> {
//!         match ord {
//!             0 => Ok(Suit::Hearts),
//!             1 => Ok(Suit::Diamonds),
//!             2 => Ok(Suit::Clubs),
//!             3 => Ok(Suit::Spades),
//!             _ => Err(DecodeError::OrdinalOutOfRange { ordinal: ord, radix: 4 }),
//!         }
//!     }
//! }
//!
//! // Once Packable is implemented, Pack is available automatically
//! let bytes = Suit::Clubs.encode();
//! assert_eq!(Suit::decode(&bytes).unwrap(), Suit::Clubs);
//! assert_eq!(Suit::bit_size(), 2);
//! ```
//!
//! # Encoding and Decoding at the Ordinal Level
//!
//! For lower-level control, use [`Packable::to_ordinal`] and [`Packable::from_ordinal`]
//! directly — for example to embed a value into a larger encoding scheme:
//!
//! ```
//! use planck::{Planck, Packable};
//!
//! #[derive(Debug, PartialEq, Planck)]
//! enum Color { Red, Green, Blue }
//!
//! assert_eq!(Color::Green.to_ordinal(), 1);
//! assert_eq!(Color::from_ordinal(2).unwrap(), Color::Blue);
//! assert!(Color::from_ordinal(3).is_err()); // only 3 values exist
//! ```

pub use planck_core::{DecodeError, Pack, Packable};

#[cfg(feature = "derive")]
pub use planck_derive::Planck;
