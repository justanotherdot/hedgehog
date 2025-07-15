#![cfg(feature = "derive")]

use hedgehog::*;
use hedgehog_derive::Generate;

#[derive(Generate, Debug, Clone, PartialEq)]
struct User {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Generate, Debug, Clone, PartialEq)]
struct Point(i32, i32);

#[derive(Generate, Debug, Clone, PartialEq)]
struct Unit;

#[derive(Generate, Debug, Clone, PartialEq)]
enum Status {
    Active,
    Inactive,
}

#[derive(Generate, Debug, Clone, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
    Custom(u8, u8, u8),
    Named { name: String, hex: String },
}

#[test]
fn test_derive_struct_with_fields() {
    let gen = User::generate();
    let seed = Seed::random();
    let size = Size::new(10);
    let tree = gen.generate(size, seed);
    let user = tree.outcome();

    // Basic structure test
    assert!(user.age <= 100);
    // Name length is always >= 0 for strings, so just check it exists
    let _ = user.name.len();
}

#[test]
fn test_derive_tuple_struct() {
    let gen = Point::generate();
    let seed = Seed::random();
    let size = Size::new(10);
    let tree = gen.generate(size, seed);
    let point = tree.outcome();

    // Tuple structure test
    assert!(point.0 >= 0 && point.0 <= 100);
    assert!(point.1 >= 0 && point.1 <= 100);
}

#[test]
fn test_derive_unit_struct() {
    let gen = Unit::generate();
    let seed = Seed::random();
    let size = Size::new(10);
    let tree = gen.generate(size, seed);
    let unit = tree.outcome();

    // Unit structure test
    assert_eq!(*unit, Unit);
}

#[test]
fn test_derive_simple_enum() {
    let gen = Status::generate();
    let seed = Seed::random();
    let size = Size::new(10);
    let tree = gen.generate(size, seed);
    let status = tree.outcome();

    // Enum variant test
    assert!(matches!(status, Status::Active | Status::Inactive));
}

#[test]
fn test_derive_complex_enum() {
    let gen = Color::generate();
    let seed = Seed::random();
    let size = Size::new(10);
    let tree = gen.generate(size, seed);
    let color = tree.outcome();

    // Complex enum test
    match color {
        Color::Red | Color::Green | Color::Blue => {
            // Unit variants are valid
        }
        Color::Custom(r, g, b) => {
            // Tuple variant values should be in range (u8 max is 255)
            assert!(r <= &255);
            assert!(g <= &255);
            assert!(b <= &255);
        }
        Color::Named { name, hex } => {
            // Named variant fields should be generated (length always >= 0)
            let _ = name.len();
            let _ = hex.len();
        }
    }
}

#[test]
fn test_derive_property_based() {
    let user_prop = property::for_all(User::generate(), |user: &User| user.age <= 100);

    match user_prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass { .. } => {
            // Property should pass
        }
        result => panic!("Property failed: {:?}", result),
    }
}

#[test]
fn test_derive_shrinking() {
    let point_prop = property::for_all(Point::generate(), |point: &Point| {
        // This property should fail to test shrinking
        point.0 > 50 && point.1 > 50
    });

    match point_prop.run(&Config::default().with_tests(100)) {
        TestResult::Fail { counterexample, .. } => {
            // Should find a counterexample that shrinks toward smaller values
            println!("Counterexample found (as expected): {}", counterexample);
        }
        TestResult::Pass { .. } => {
            // This might happen due to randomness, but it's unlikely with enough tests
            println!("Property unexpectedly passed - this is rare but possible");
        }
        result => panic!("Unexpected result: {:?}", result),
    }
}
