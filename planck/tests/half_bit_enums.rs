use planck::{Pack, Packable, Planck};

/// The key insight: when an enum has variants carrying Planck types,
/// the variant discriminant costs ZERO extra bits if the types have
/// unused ordinal space.
///
/// Think of it like Option<Color> — 3 colors + None = 4 values = 2 bits.
/// The "discriminant" (Some vs None) costs 0 extra bits because it fits
/// in the unused slot of the 2-bit encoding.
///
/// This generalizes: an enum with A(T) and B(U) has RADIX = T::RADIX + U::RADIX.
/// The discriminant is "free" when that sum fits in the same bit width
/// as max(T::RADIX, U::RADIX) would alone.

#[derive(Debug, PartialEq, Planck)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Debug, PartialEq, Planck)]
enum Size {
    Small,
    Medium,
    Large,
}

// An enum wrapping two 3-value types.
// Without planck: you'd need 1 bit discriminant + 2 bits data = 3 bits
// With planck: RADIX = 3 + 3 = 6, needs ceil(log2(6)) = 3 bits
// Same here... but now add a third variant:
#[derive(Debug, PartialEq, Planck)]
enum ColorOrSize {
    Color(Color),   // radix 3
    Size(Size),     // radix 3
}

#[test]
fn two_variant_enum() {
    // 3 + 3 = 6 → 3 bits → 1 byte
    assert_eq!(ColorOrSize::RADIX, 6);
    assert_eq!(ColorOrSize::bit_size(), 3);

    // All 6 values round-trip
    let all = vec![
        ColorOrSize::Color(Color::Red),
        ColorOrSize::Color(Color::Green),
        ColorOrSize::Color(Color::Blue),
        ColorOrSize::Size(Size::Small),
        ColorOrSize::Size(Size::Medium),
        ColorOrSize::Size(Size::Large),
    ];
    for (i, v) in all.iter().enumerate() {
        assert_eq!(v.to_ordinal(), i as u128);
        assert_eq!(ColorOrSize::decode(&v.encode()).unwrap(), *v);
    }
}

// The "half bit" case: 3 variants wrapping 3-value types
// RADIX = 3 + 3 + 3 = 9 → 4 bits
// Three separate 3-value enums would need 2+2+2 = 6 bits
// As a single enum: 4 bits. The discriminant (which of 3 variants)
// costs ~1.58 bits, but the total is only 3.17 bits — the variant
// tag shares the fractional bits with the payload!
#[derive(Debug, PartialEq, Planck)]
enum ThreeWay {
    AsColor(Color),  // radix 3
    AsSize(Size),    // radix 3
    AsBool(bool),    // radix 2
}

#[test]
fn three_way_half_bit() {
    // 3 + 3 + 2 = 8 → 3 bits!
    assert_eq!(ThreeWay::RADIX, 8);
    assert_eq!(ThreeWay::bit_size(), 3); // Exactly 3 bits for 8 values — perfect packing

    let all = vec![
        ThreeWay::AsColor(Color::Red),
        ThreeWay::AsColor(Color::Green),
        ThreeWay::AsColor(Color::Blue),
        ThreeWay::AsSize(Size::Small),
        ThreeWay::AsSize(Size::Medium),
        ThreeWay::AsSize(Size::Large),
        ThreeWay::AsBool(false),
        ThreeWay::AsBool(true),
    ];
    assert_eq!(all.len(), 8); // exactly 2^3
    for (i, v) in all.iter().enumerate() {
        assert_eq!(v.to_ordinal(), i as u128);
        assert_eq!(ThreeWay::decode(&v.encode()).unwrap(), *v);
    }
}

// The real power: mixing differently-sized payloads
// Imagine a "sparse update" protocol where most messages are small
#[derive(Debug, PartialEq, Planck)]
struct Birthday {
    #[planck(range = 1..=12)]
    month: u8,
    #[planck(range = 1..=31)]
    day: u8,
}

#[derive(Debug, PartialEq, Planck)]
enum Update {
    Noop,                       // radix 1
    SetColor(Color),            // radix 3
    SetBirthday(Birthday),      // radix 372
}

#[test]
fn mixed_payload_sizes() {
    // 1 + 3 + 372 = 376 → 9 bits → 2 bytes
    // Naive: 2 bits discriminant + max(0, 2, 16) bits payload = 18 bits = 3 bytes
    // Planck: 2 bytes. The discriminant is essentially free!
    assert_eq!(Update::RADIX, 376);
    assert_eq!(Update::byte_size(), 2); // vs 3 bytes naive

    assert_eq!(Update::Noop.to_ordinal(), 0);
    assert_eq!(Update::SetColor(Color::Red).to_ordinal(), 1);
    assert_eq!(Update::SetBirthday(Birthday { month: 1, day: 1 }).to_ordinal(), 4);

    // Round-trip
    let v = Update::SetBirthday(Birthday { month: 6, day: 15 });
    assert_eq!(Update::decode(&v.encode()).unwrap(), v);
}

// Struct containing such enums — the savings compound
#[derive(Debug, PartialEq, Planck)]
struct Packet {
    update1: Update,        // radix 376
    update2: Update,        // radix 376
    urgent: bool,           // radix 2
}

#[test]
fn compounding_savings() {
    // 376 × 376 × 2 = 282,752 → 19 bits → 3 bytes
    // Naive: 2 * 3 bytes + 1 byte = 7 bytes
    assert_eq!(Packet::RADIX, 282752);
    assert_eq!(Packet::bit_size(), 19);
    assert_eq!(Packet::byte_size(), 3); // 3 bytes vs 7 bytes naive!

    let p = Packet {
        update1: Update::SetColor(Color::Blue),
        update2: Update::SetBirthday(Birthday { month: 12, day: 25 }),
        urgent: true,
    };
    assert_eq!(Packet::decode(&p.encode()).unwrap(), p);
}
