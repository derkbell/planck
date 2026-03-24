use planck_pack::{Pack, Packable, Planck};

#[derive(Debug, PartialEq, Planck)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Debug, PartialEq, Planck)]
struct Birthday {
    #[planck(range = 1..=12)]
    month: u8,
    #[planck(range = 1..=31)]
    day: u8,
}

// --- Combinations: Birthday + enum in one struct ---

#[derive(Debug, PartialEq, Planck)]
struct BirthdayWithColor {
    birthday: Birthday,
    color: Color,
}

#[test]
fn birthday_with_color() {
    // 372 × 3 = 1116 combinations → 11 bits → 2 bytes
    assert_eq!(BirthdayWithColor::RADIX, 1116);
    assert_eq!(BirthdayWithColor::bit_size(), 11);
    assert_eq!(BirthdayWithColor::byte_size(), 2);

    let v = BirthdayWithColor {
        birthday: Birthday { month: 3, day: 14 },
        color: Color::Green,
    };
    assert_eq!(BirthdayWithColor::decode(&v.encode()).unwrap(), v);
}

// --- Option<Color>: None uses the "fourth slot" ---

#[derive(Debug, PartialEq, Planck)]
struct MaybeColor {
    color: Option<Color>,
}

#[test]
fn option_color_uses_fourth_slot() {
    // Option<Color> has RADIX = 3 + 1 = 4, exactly 2 bits!
    assert_eq!(<Option<Color>>::RADIX, 4);
    assert_eq!(MaybeColor::RADIX, 4);
    assert_eq!(MaybeColor::bit_size(), 2);
    assert_eq!(MaybeColor::byte_size(), 1);

    // Round-trip all 4 values
    let values = [
        MaybeColor { color: None },
        MaybeColor { color: Some(Color::Red) },
        MaybeColor { color: Some(Color::Green) },
        MaybeColor { color: Some(Color::Blue) },
    ];
    for (i, v) in values.iter().enumerate() {
        assert_eq!(v.to_ordinal(), i as u128);
        assert_eq!(MaybeColor::decode(&v.encode()).unwrap(), *v);
    }
}

// --- Multiple Options compound the savings ---

#[derive(Debug, PartialEq, Planck)]
struct TwoOptionalColors {
    primary: Option<Color>,
    secondary: Option<Color>,
}

#[test]
fn two_optional_colors() {
    // 4 × 4 = 16 combinations → 4 bits → 1 byte
    // Naive: each Option<Color> would be 1 byte (tag) + 1 byte (value) = 4 bytes
    assert_eq!(TwoOptionalColors::RADIX, 16);
    assert_eq!(TwoOptionalColors::bit_size(), 4);
    assert_eq!(TwoOptionalColors::byte_size(), 1); // 1 byte vs 4 bytes naive!

    let v = TwoOptionalColors {
        primary: Some(Color::Blue),
        secondary: None,
    };
    assert_eq!(TwoOptionalColors::decode(&v.encode()).unwrap(), v);
}
