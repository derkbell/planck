use planck_pack::{Pack, Packable, Planck};

#[derive(Debug, PartialEq, Planck)]
struct Birthday {
    #[planck(range = 1..=12)]
    month: u8,
    #[planck(range = 1..=31)]
    day: u8,
}

#[test]
fn birthday_radix() {
    // 12 months × 31 days = 372 combinations
    assert_eq!(Birthday::RADIX, 372);
    assert_eq!(Birthday::bit_size(), 9); // ceil(log2(372)) = 9
    assert_eq!(Birthday::byte_size(), 2); // ceil(9/8) = 2
}

#[test]
fn birthday_round_trip() {
    for month in 1..=12u8 {
        for day in 1..=31u8 {
            let b = Birthday { month, day };
            let encoded = b.encode();
            assert_eq!(encoded.len(), 2);
            let decoded = Birthday::decode(&encoded).unwrap();
            assert_eq!(decoded, b);
        }
    }
}

#[test]
fn birthday_unique_ordinals() {
    let mut seen = std::collections::HashSet::new();
    for month in 1..=12u8 {
        for day in 1..=31u8 {
            let b = Birthday { month, day };
            let ord = b.to_ordinal();
            assert!(ord < 372, "ordinal {ord} >= 372");
            assert!(seen.insert(ord), "duplicate ordinal {ord}");
        }
    }
    assert_eq!(seen.len(), 372);
}

#[test]
fn birthday_specific_values() {
    // Jan 1 → ordinal = (1-1) + 12*(1-1) = 0
    let jan1 = Birthday { month: 1, day: 1 };
    assert_eq!(jan1.to_ordinal(), 0);

    // Dec 31 → ordinal = (12-1) + 12*(31-1) = 11 + 360 = 371
    let dec31 = Birthday { month: 12, day: 31 };
    assert_eq!(dec31.to_ordinal(), 371);
}

#[derive(Debug, PartialEq, Planck)]
struct WithBool {
    flag: bool,
    #[planck(range = 0..=10)]
    value: u8,
}

#[test]
fn bool_and_range() {
    // bool(2) × range(11) = 22 combinations → 5 bits → 1 byte
    assert_eq!(WithBool::RADIX, 22);
    assert_eq!(WithBool::bit_size(), 5);
    assert_eq!(WithBool::byte_size(), 1);

    for flag in [false, true] {
        for value in 0..=10u8 {
            let v = WithBool { flag, value };
            let decoded = WithBool::decode(&v.encode()).unwrap();
            assert_eq!(decoded, v);
        }
    }
}

#[derive(Debug, PartialEq, Planck)]
struct Nested {
    birthday: Birthday,
    flag: bool,
}

#[test]
fn nested_struct() {
    // 372 × 2 = 744
    assert_eq!(Nested::RADIX, 744);

    let v = Nested {
        birthday: Birthday { month: 6, day: 15 },
        flag: true,
    };
    let decoded = Nested::decode(&v.encode()).unwrap();
    assert_eq!(decoded, v);
}
