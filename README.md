# planck

Mixed-radix bit-packing serialization for Rust. Encodes structs and enums using the theoretical minimum number of bits by treating each field as a digit in a mixed-radix number system.

## The Problem

Traditional serialization rounds each field up to power-of-2 bit widths:

```
struct Birthday { year: u16, month: u8, day: u8 }
// Naive: 16 + 8 + 8 = 32 bits (4 bytes)
// But year is 2000-2100, month is 1-12, day is 1-31
// → only 101 × 12 × 31 = 37,572 combinations → 16 bits
```

Three fields into **2 bytes**. That's half the naive size — and the savings compound across structs with multiple constrained fields.

## How It Works

Each field has a **radix** — the number of distinct values it can take. Planck encodes the struct as a single mixed-radix number:

```
field radixes:  101 × 12 × 31 = 37,572 total combinations
bits needed:    ⌈log₂(37,572)⌉ = 16 bits (2 bytes)
```

Encoding uses Horner's method (multiply-and-add), decoding uses successive div-and-mod. Fast, simple, and optimal.

The tighter your constraints, the more you save:

```
year range      radix    × month × day = total       bits   bytes   naive
0..=2500        2501     × 12    × 31  = 930,372     20     3       4
1900..=2100     201      × 12    × 31  = 74,772      17     3       4
2000..=2100     101      × 12    × 31  = 37,572      16     2       4
```

## Usage

```rust
use planck::{Planck, Pack, Packable};

#[derive(Debug, PartialEq, Planck)]
struct Birthday {
    #[planck(range = 2000..=2100)]
    year: u16,
    #[planck(range = 1..=12)]
    month: u8,
    #[planck(range = 1..=31)]
    day: u8,
}

let bday = Birthday { year: 2024, month: 3, day: 14 };
let bytes = bday.encode();
let decoded = Birthday::decode(&bytes).unwrap();
assert_eq!(decoded, bday);

assert_eq!(Birthday::RADIX, 37_572);  // only 37,572 possible values
assert_eq!(Birthday::bit_size(), 16);   // 16 bits — 3 fields in 2 bytes!
assert_eq!(bytes.len(), 2);             // vs 4 bytes naive
```

## Enums

Unit enums get their variant count as the radix:

```rust
#[derive(Planck)]
enum Color { Red, Green, Blue }  // RADIX = 3

// Option<Color> has RADIX = 4 (None + 3 colors)
// Fits in exactly 2 bits — None uses the "fourth slot" for free
```

Data-carrying enums sum their variant radixes. The discriminant costs zero extra bits — it's absorbed into the mixed-radix encoding:

```rust
#[derive(Planck)]
enum Update {
    Noop,                    // radix 1
    SetColor(Color),         // radix 3
    SetBirthday(Birthday),   // radix 37,572
}
// RADIX = 1 + 3 + 37,572 = 37,576 → 16 bits → 2 bytes
// The variant tag is essentially free!
```

## Composing Types

Planck types nest naturally. Radixes multiply across struct fields and add across enum variants:

```rust
#[derive(Planck)]
struct Packet {
    update1: Update,  // radix 37,576
    update2: Update,  // radix 37,576
    urgent: bool,     // radix 2
}
// RADIX = 37,576 × 37,576 × 2 = 2,823,912,576 → 32 bits → 4 bytes
// Naive (tag + payload per field): ~11 bytes
```

## Supported Types

| Type | RADIX | Notes |
|------|-------|-------|
| `bool` | 2 | |
| `u8`, `u16`, `u32`, `u64` | 2^N | Full range |
| `i8`, `i16`, `i32`, `i64` | 2^N | Mapped via unsigned |
| `Option<T>` | T::RADIX + 1 | None = 0, Some(v) = v + 1 |
| `()` | 1 | Zero bits |
| `#[planck(range = a..=b)]` | b - a + 1 | Constrained integers |
| `#[derive(Planck)]` structs | product of field radixes | |
| `#[derive(Planck)]` enums | sum of variant radixes | Unit, tuple, and named variants |

## Traits

- **`Packable`** — core trait. Declares `RADIX` and provides `to_ordinal()` / `from_ordinal()`.
- **`Pack`** — blanket-implemented for all `Packable` types. Provides `encode()` / `decode()` and `bit_size()` / `byte_size()`.

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `planck` | Facade — re-exports everything, add this to your `Cargo.toml` |
| `planck-core` | Core traits and primitive impls (`no_std` compatible) |
| `planck-derive` | `#[derive(Planck)]` proc macro |

## When Is This Useful?

- **Low-bandwidth protocols** — IoT, satellite, mesh networks where every bit counts
- **Long-term storage** — billions of records with constrained fields
- **Compact IDs** — encoding structured identifiers into minimal bytes
- **Game state** — packing game objects with known value ranges
- **Embedded systems** — memory-constrained environments

## License

MIT OR Apache-2.0
