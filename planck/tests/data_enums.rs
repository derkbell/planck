use planck::{Pack, Packable, Planck};

// --- Enum with data-carrying variants ---

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

// An enum where some variants carry Planck data
#[derive(Debug, PartialEq, Planck)]
enum Shape {
    Circle { radius: u8 },                // radix = 256
    Rectangle { width: u8, height: u8 },  // radix = 256 * 256 = 65536
    Point,                                 // radix = 1
}

#[test]
fn shape_radix() {
    // 256 + 65536 + 1 = 65793
    assert_eq!(Shape::RADIX, 65793);
}

#[test]
fn shape_round_trip() {
    let values = vec![
        Shape::Circle { radius: 0 },
        Shape::Circle { radius: 255 },
        Shape::Circle { radius: 42 },
        Shape::Rectangle { width: 10, height: 20 },
        Shape::Rectangle { width: 0, height: 0 },
        Shape::Rectangle { width: 255, height: 255 },
        Shape::Point,
    ];
    for v in &values {
        let ord = v.to_ordinal();
        assert!(ord < Shape::RADIX, "{:?} ordinal {} >= {}", v, ord, Shape::RADIX);
        let decoded = Shape::from_ordinal(ord).unwrap();
        assert_eq!(&decoded, v);

        // Also test byte-level encode/decode
        let bytes = v.encode();
        let decoded2 = Shape::decode(&bytes).unwrap();
        assert_eq!(&decoded2, v);
    }
}

#[test]
fn shape_ordinal_ranges() {
    // Circle occupies [0, 256)
    assert_eq!(Shape::Circle { radius: 0 }.to_ordinal(), 0);
    assert_eq!(Shape::Circle { radius: 255 }.to_ordinal(), 255);

    // Rectangle occupies [256, 256 + 65536) = [256, 65792)
    assert_eq!(Shape::Rectangle { width: 0, height: 0 }.to_ordinal(), 256);

    // Point occupies [65792, 65793)
    assert_eq!(Shape::Point.to_ordinal(), 65792);
}

// --- Tuple variants ---

#[derive(Debug, PartialEq, Planck)]
enum Message {
    Quit,
    Text(u8),
    Pair(u8, bool),
}

#[test]
fn tuple_variants() {
    // Quit: 1, Text: 256, Pair: 256 * 2 = 512
    // Total: 1 + 256 + 512 = 769
    assert_eq!(Message::RADIX, 769);

    let values = vec![
        Message::Quit,
        Message::Text(0),
        Message::Text(42),
        Message::Text(255),
        Message::Pair(100, false),
        Message::Pair(200, true),
    ];
    for v in &values {
        let decoded = Message::from_ordinal(v.to_ordinal()).unwrap();
        assert_eq!(&decoded, v);
    }
}

// --- Enum variants carrying other Planck types ---

#[derive(Debug, PartialEq, Planck)]
enum Event {
    Nothing,
    ColorChanged(Color),           // radix = 3
    BirthdaySet { bday: Birthday }, // radix = 372
}

#[test]
fn enum_with_planck_fields() {
    // 1 + 3 + 372 = 376
    assert_eq!(Event::RADIX, 376);
    assert_eq!(Event::bit_size(), 9);  // ceil(log2(376)) = 9
    assert_eq!(Event::byte_size(), 2);

    let values = vec![
        Event::Nothing,
        Event::ColorChanged(Color::Red),
        Event::ColorChanged(Color::Green),
        Event::ColorChanged(Color::Blue),
        Event::BirthdaySet { bday: Birthday { month: 1, day: 1 } },
        Event::BirthdaySet { bday: Birthday { month: 12, day: 31 } },
        Event::BirthdaySet { bday: Birthday { month: 6, day: 15 } },
    ];
    for v in &values {
        let ord = v.to_ordinal();
        assert!(ord < Event::RADIX);
        let decoded = Event::from_ordinal(ord).unwrap();
        assert_eq!(&decoded, v);
        // Byte round-trip
        assert_eq!(Event::decode(&v.encode()).unwrap(), *v);
    }
}

// --- The Option<Color> case: None is the "fourth slot" ---
// This already works via the built-in Option<T> impl, but let's
// show it also works as a field inside a data-carrying enum variant.

#[derive(Debug, PartialEq, Planck)]
enum Instruction {
    Noop,
    SetColor(Option<Color>), // radix = 4 (None + 3 colors)
}

#[test]
fn enum_with_option_planck_field() {
    // 1 + 4 = 5
    assert_eq!(Instruction::RADIX, 5);
    assert_eq!(Instruction::bit_size(), 3); // ceil(log2(5)) = 3
    assert_eq!(Instruction::byte_size(), 1);

    let values = vec![
        Instruction::Noop,
        Instruction::SetColor(None),
        Instruction::SetColor(Some(Color::Red)),
        Instruction::SetColor(Some(Color::Green)),
        Instruction::SetColor(Some(Color::Blue)),
    ];
    for (i, v) in values.iter().enumerate() {
        assert_eq!(v.to_ordinal(), i as u128);
        assert_eq!(Instruction::decode(&v.encode()).unwrap(), *v);
    }
}

// --- Exhaustive test: enum with range-constrained fields in variants ---

#[derive(Debug, PartialEq, Planck)]
enum Compact {
    A,
    B {
        #[planck(range = 0..=5)]
        x: u8,
    },
    C(bool),
}

#[test]
fn enum_with_range_in_variant() {
    // A: 1, B: 6, C: 2 → total = 9
    assert_eq!(Compact::RADIX, 9);

    // Exhaustive round-trip
    let mut all = vec![Compact::A];
    for x in 0..=5u8 {
        all.push(Compact::B { x });
    }
    all.push(Compact::C(false));
    all.push(Compact::C(true));

    assert_eq!(all.len(), 9);
    let mut seen = std::collections::HashSet::new();
    for v in &all {
        let ord = v.to_ordinal();
        assert!(ord < 9);
        assert!(seen.insert(ord), "duplicate ordinal");
        assert_eq!(Compact::from_ordinal(ord).unwrap(), *v);
    }
}
