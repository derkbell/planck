use planck_pack::{Pack, Packable, Planck};

#[derive(Debug, PartialEq, Planck)]
enum Color {
    Red,
    Green,
    Blue,
}

#[test]
fn color_radix() {
    assert_eq!(Color::RADIX, 3);
    assert_eq!(Color::bit_size(), 2);
    assert_eq!(Color::byte_size(), 1);
}

#[test]
fn color_round_trip() {
    for (i, color) in [Color::Red, Color::Green, Color::Blue].iter().enumerate() {
        assert_eq!(color.to_ordinal(), i as u128);
        let decoded = Color::from_ordinal(i as u128).unwrap();
        assert_eq!(&decoded, color);
    }
    assert!(Color::from_ordinal(3).is_err());
}

#[test]
fn color_encode_decode() {
    let c = Color::Green;
    let bytes = c.encode();
    assert_eq!(bytes.len(), 1);
    assert_eq!(Color::decode(&bytes).unwrap(), Color::Green);
}

#[derive(Debug, PartialEq, Planck)]
struct Pixel {
    fg: Color,
    bg: Color,
    bold: bool,
}

#[test]
fn pixel_packing() {
    // 3 × 3 × 2 = 18 combinations → 5 bits → 1 byte
    assert_eq!(Pixel::RADIX, 18);
    assert_eq!(Pixel::bit_size(), 5);
    assert_eq!(Pixel::byte_size(), 1);

    // Exhaustive round-trip
    let colors = [Color::Red, Color::Green, Color::Blue];
    for fg in &colors {
        for bg in &colors {
            for bold in [false, true] {
                let p = Pixel {
                    fg: Color::from_ordinal(fg.to_ordinal()).unwrap(),
                    bg: Color::from_ordinal(bg.to_ordinal()).unwrap(),
                    bold,
                };
                let decoded = Pixel::decode(&p.encode()).unwrap();
                assert_eq!(decoded, p);
            }
        }
    }
}

#[derive(Debug, PartialEq, Planck)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, PartialEq, Planck)]
struct Movement {
    direction: Direction,
    color: Color,
    #[planck(range = 1..=100)]
    speed: u8,
}

#[test]
fn movement_savings() {
    // 4 × 3 × 100 = 1200 combinations → 11 bits → 2 bytes
    // Naive: direction(2 bits) + color(2 bits) + speed(7 bits) = 11 bits
    // Same here, but the savings show up in less aligned cases
    assert_eq!(Movement::RADIX, 1200);
    assert_eq!(Movement::bit_size(), 11);

    let m = Movement {
        direction: Direction::South,
        color: Color::Blue,
        speed: 42,
    };
    let decoded = Movement::decode(&m.encode()).unwrap();
    assert_eq!(decoded, m);
}
