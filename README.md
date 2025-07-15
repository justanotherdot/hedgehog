# Hedgehog

> Release with confidence.

Property-based testing library for Rust, inspired by the original [Hedgehog](https://hedgehog.qa/) library for Haskell.

## Why Hedgehog?

- **Explicit generators** - No type-directed magic, generators are first-class values you compose
- **Integrated shrinking** - Shrinks obey invariants by construction, built into generators
- **Excellent debugging** - Minimal counterexamples with rich failure reporting
- **Distribution shaping** - Control probability distributions for realistic test data
- **Variable name tracking** - Enhanced failure reporting with named inputs
- **Derive macros** - Automatic generator creation for custom types

## Quick Start

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
hedgehog = "0.1"

# For derive macros
hedgehog = { version = "0.1", features = ["derive"] }
```

### Basic Property Test

```rust
use hedgehog::*;

#[test]
fn prop_reverse() {
    let gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 100));
    let prop = for_all(gen, |xs: &Vec<i32>| {
        let reversed: Vec<_> = xs.iter().rev().cloned().collect();
        let double_reversed: Vec<_> = reversed.iter().rev().cloned().collect();
        *xs == double_reversed
    });
    
    match prop.run(&Config::default()) {
        TestResult::Pass { .. } => (), // Test passed
        result => panic!("Property failed: {:?}", result),
    }
}
```

### With Distribution Shaping and Variable Names

```rust
use hedgehog::*;

#[test]
fn prop_string_length() {
    // Generate strings with realistic length distribution (favor shorter strings)
    let gen = Gen::<String>::alpha_with_range(Range::exponential(1, 50));
    
    // Use named variables for better failure reporting
    let prop = for_all_named(gen, "text", |text| {
        let uppercase = text.to_uppercase();
        uppercase.len() == text.len()
    });
    
    match prop.run(&Config::default()) {
        TestResult::Pass { tests_run } => {
            println!("Property passed {} tests", tests_run);
        }
        TestResult::Fail { .. } => {
            // Shows: forAll 0 = "SomeString" -- text
            println!("Property failed:\n{}", prop.run(&Config::default()));
        }
        _ => {}
    }
}
```

### Realistic Data Generation

```rust
use hedgehog::*;

#[test]
fn prop_http_status_codes() {
    // Generate realistic HTTP status code distribution
    let status_gen = Gen::frequency(vec![
        WeightedChoice::new(70, Gen::constant(200)),    // 70% success
        WeightedChoice::new(15, Gen::constant(404)),    // 15% not found
        WeightedChoice::new(10, Gen::constant(500)),    // 10% server error
        WeightedChoice::new(5, Gen::int_range(300, 399)), // 5% redirects
    ]);
    
    let prop = for_all_named(status_gen, "status", |&status| {
        status >= 100 && status < 600
    });
    
    assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));
}
```

### Automatic Generator Creation

```rust
use hedgehog::*;
use hedgehog_derive::Generate;

#[derive(Generate, Debug, Clone)]
struct User {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Generate, Debug, Clone)]
enum PaymentMethod {
    Cash,
    Card { number: String, expiry: String },
    Digital(String),
}

#[test]
fn prop_user_validation() {
    let prop = for_all_named(User::generate(), "user", |user: &User| {
        !user.name.is_empty() && user.age <= 100
    });
    
    match prop.run(&Config::default()) {
        TestResult::Pass { .. } => println!("✓ User validation passed"),
        TestResult::Fail { counterexample, .. } => {
            println!("✗ Failed with user: {}", counterexample);
        }
        result => println!("Unexpected result: {:?}", result),
    }
}
```

## Command Line Interface

Hedgehog includes a powerful CLI for enhanced testing workflows:

```sh
# Install the CLI
cargo install --path hedgehog-cli

# Run property tests with enhanced output
cargo hedgehog test --verbose --count 500

# Generate reports in multiple formats
cargo hedgehog report --format json --output test-report.json

# Check test coverage
cargo hedgehog coverage --detailed
```

The CLI provides:
- **Enhanced output formatting** with color-coded test results
- **Multiple report formats** (JSON, HTML, markdown)
- **Test coverage analysis** showing property test coverage
- **Configurable test parameters** (count, patterns, features)

See [hedgehog-cli/README.md](hedgehog-cli/README.md) for detailed CLI documentation.

## Key Features

### Explicit Generators

Unlike type-directed approaches, Hedgehog generators are explicit values you create and compose:

```rust
let gen_small_int = Gen::int_range(1, 10);
let gen_list = Gen::<Vec<i32>>::vec_of(gen_small_int);
let gen_pair = Gen::<(i32, String)>::tuple_of(gen_small_int, Gen::<String>::ascii_alpha());
```

### Distribution Shaping

Control probability distributions for realistic test data:

```rust
// Uniform distribution (equal probability)
Gen::<i32>::from_range(Range::new(1, 100))

// Linear distribution (favors smaller values)
Gen::<i32>::from_range(Range::linear(1, 100))

// Exponential distribution (strongly favors smaller values)
Gen::<i32>::from_range(Range::exponential(1, 100))

// Weighted choice
Gen::frequency(vec![
    WeightedChoice::new(8, Gen::constant("common")),
    WeightedChoice::new(2, Gen::constant("rare")),
])
```

### Variable Name Tracking

Enhanced failure reporting with named inputs:

```rust
// Without variable names
let prop = for_all(gen, |input| test_condition(input));
// Output: │ Original: 42

// With variable names
let prop = for_all_named(gen, "input", |input| test_condition(input));
// Output: │ forAll 0 = 42 -- input
```

### Integrated Shrinking

When a test fails, Hedgehog automatically finds the minimal counterexample:

```
✗ property failed after 13 tests and 5 shrinks.

    Shrinking progression:
      │ forAll 0 = [1, 2, 3, 4, 5] -- list
      │ forAll 1 = [1, 2, 3] -- list
      │ forAll 2 = [1, 2] -- list
      │ forAll 3 = [2] -- list
      │ forAll 4 = [1] -- list
      │ forAll 5 = [] -- list

    Minimal counterexample: []
```

## Documentation

- **[API Guide](docs/api-guide.md)** - Comprehensive API reference and examples
- **[Distribution Shaping](docs/distribution-shaping.md)** - Control probability distributions for realistic test data
- **[Variable Name Tracking](docs/variable-name-tracking.md)** - Enhanced failure reporting with named inputs
- **[Derive Macros](docs/derive-macros.md)** - Automatic generator creation for custom types
- **[Roadmap](docs/roadmap.md)** - Development plan and project status

## Examples

Run the examples to see Hedgehog in action:

```bash
# Distribution shaping examples
cargo run --example distribution-shaping

# Variable name tracking examples  
cargo run --example variable-name-tracking

# Basic usage examples
cargo run --example basic

# Derive macro examples
cargo run --example derive-macro --features derive
```

## In Memory of Jacob Stanley

This library is inspired by the original Hedgehog library for Haskell, created by Jacob Stanley and the Hedgehog team. Jacob was a remarkable mentor who had a profound influence on many in the functional programming community, including the author of this Rust port.

Jacob's vision of property-based testing with integrated shrinking revolutionized how we think about testing. His approach of making shrinking a first-class concern, built into the generator rather than bolted on afterwards, makes finding minimal counterexamples both automatic and reliable.

Jacob passed away unexpectedly on April 9th, 2021. His absence is deeply felt, but his impact on property-based testing and the broader programming community remains. This Rust port aims to honor his memory by bringing his innovative approach to a new language and community.

**RIP, Jake.** Your mentorship and ideas live on.

## Project Status

This is a work-in-progress implementation. See [docs/roadmap.md](docs/roadmap.md) for the development plan.

### Recently Completed

- ✅ **Distribution Shaping and Range System** - Control probability distributions with uniform, linear, exponential, and constant distributions
- ✅ **Variable Name Tracking** - Enhanced failure reporting showing named inputs in shrinking progression
- ✅ **Frequency-Based Generation** - Weighted choice and one-of generators for realistic data patterns
- ✅ **Enhanced String Generation** - Controlled length ranges and distribution-based character generation

## Contributing

Contributions are welcome! Please see the [roadmap](docs/roadmap.md) for planned features and current progress.

## License

This project is licensed under the BSD-3-Clause License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Jacob Stanley and the original Hedgehog team for the foundational ideas
- The Haskell, F#, and R Hedgehog ports for implementation insights
- The Rust community for excellent tooling and ecosystem support