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

fn main() {
    let seed = Seed::random();
    let size = Size::new(10);

    println!("Generated User:");
    let user_gen = User::generate();
    let user_tree = user_gen.generate(size, seed);
    let user = user_tree.outcome();
    println!("{:?}", user);

    println!("\nGenerated Point:");
    let point_gen = Point::generate();
    let point_tree = point_gen.generate(size, seed);
    let point = point_tree.outcome();
    println!("{:?}", point);

    println!("\nGenerated Unit:");
    let unit_gen = Unit::generate();
    let unit_tree = unit_gen.generate(size, seed);
    let unit = unit_tree.outcome();
    println!("{:?}", unit);

    println!("\nGenerated Status:");
    let status_gen = Status::generate();
    let status_tree = status_gen.generate(size, seed);
    let status = status_tree.outcome();
    println!("{:?}", status);

    // Test with a property
    println!("\nTesting with property:");
    let result = property::for_all(User::generate(), |user: &User| {
        user.name.len() > 0 && user.age <= 100
    });

    match result.run(&Config::default()) {
        TestResult::Pass { .. } => println!("✓ Property passed"),
        TestResult::Fail { counterexample, .. } => {
            println!("✗ Property failed with counterexample: {}", counterexample);
        }
        result => println!("Unexpected result: {:?}", result),
    }
}
