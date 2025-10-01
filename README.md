# Hedgehog

> Release with confidence.

Property-based testing library for Rust, inspired by the original [Hedgehog](https://hedgehog.qa/) library for Haskell.

## Why Hedgehog?

- **Explicit generators** - No type-directed magic, generators are first-class values you compose
- **Integrated shrinking** - Shrinks obey invariants by construction, built into generators
- **Excellent debugging** - Minimal counterexamples with rich failure reporting
- **Distribution shaping** - Control probability distributions for realistic test data
- **Variable name tracking** - Enhanced failure reporting with named inputs
- **Property classification** - Inspect test data distribution and statistics
- **Example integration** - Mix explicit test examples with generated values
- **Dictionary support** - Inject domain-specific realistic values (HTTP codes, SQL keywords, web domains)
- **Parallel testing** - Speed up tests and detect race conditions with multi-threaded execution
- **State machine testing** - Sequential and parallel command testing with linearizability verification
- **Targeted testing** - Search-guided generation to find inputs that maximize/minimize specific objectives
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

### With Example Integration

```rust
use hedgehog::*;

#[test]
fn prop_division_safety() {
    // Critical edge cases that must always be tested
    let critical_cases = vec![
        (10, 0),        // Division by zero
        (i32::MAX, 1),  // Maximum value
        (i32::MIN, -1), // Potential overflow
    ];
    
    let prop = for_all_named(
        Gen::<(i32, i32)>::tuple_of(
            Gen::int_range(-50, 50),
            Gen::int_range(-5, 5)
        ), 
        "input",
        |&(a, b)| {
            match safe_divide(a, b) {
                Some(result) => b != 0 && result == a / b,
                None => b == 0 || (a == i32::MIN && b == -1)
            }
        }
    ).with_examples(critical_cases); // Examples tested first, then random pairs
    
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

### State Machine Testing

Test stateful systems by generating sequences of commands and verifying behavior:

```rust
use hedgehog_core::state::*;
use hedgehog_core::gen::Gen;

#[test]
fn test_counter_state_machine() {
    // Define your state model
    #[derive(Clone, Debug)]
    struct Counter {
        value: i32,
        max: i32,
    }

    // Define command inputs
    #[derive(Clone, Debug)]
    struct IncrementInput { amount: i32 }

    impl std::fmt::Display for IncrementInput {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "+{}", self.amount)
        }
    }

    let mut generator = ActionGenerator::new();

    // Define increment command with precondition, update, and postcondition
    let increment: Command<IncrementInput, i32, Counter, i32> = Command::new(
        "increment".to_string(),
        |state: &Counter| {
            if state.value < state.max {
                Some(Gen::constant(IncrementInput { amount: 1 }))
            } else {
                None // Can't increment when at max
            }
        },
        |input: IncrementInput| input.amount,
    )
    .with_require(|state: &Counter, input: &IncrementInput| {
        state.value + input.amount <= state.max
    })
    .with_update(|state: &mut Counter, input: &IncrementInput, _output| {
        state.value += input.amount;
    })
    .with_ensure(|old: &Counter, new: &Counter, input: &IncrementInput, output: &i32| {
        if new.value != old.value + input.amount {
            Err("Value not updated correctly".to_string())
        } else if *output != input.amount {
            Err("Wrong output".to_string())
        } else {
            Ok(())
        }
    });

    generator.add_command(increment);

    // Sequential testing
    let initial = Counter { value: 0, max: 100 };
    let sequential = generator.generate_sequential(initial.clone(), 10);
    execute_sequential(initial.clone(), sequential).unwrap();

    // Parallel testing with linearizability checking
    let parallel = generator.generate_parallel(initial.clone(), 2, 3);
    execute_parallel(initial, parallel).unwrap();
}
```

**For more details, see:**
- Tutorial: `hedgehog-core/docs/state-machine-testing.md`
- API Reference: `hedgehog-core/docs/api-reference.md`

### Targeted Property Testing

Find inputs that maximize or minimize specific objectives using search-guided generation:

```rust
use hedgehog::*;

#[test]
fn prop_find_slow_inputs() {
    // Function that gets slower with larger inputs
    fn computation_time(n: i32) -> std::time::Duration {
        let start = std::time::Instant::now();
        expensive_function(n);
        start.elapsed()
    }
    
    let generator = Gen::<i32>::from_range(Range::new(0, 1000));
    
    // Utility function - what we want to maximize
    let utility_function = |input: &i32, _result: &TargetedResult| -> f64 {
        computation_time(*input).as_micros() as f64
    };
    
    // Test function - the property being tested
    let test_function = |input: &i32| -> TargetedResult {
        let result = expensive_function(*input);
        if result > threshold {
            TargetedResult::Fail {
                counterexample: format!("input {} took too long", input),
                tests_run: 1,
                utility: 0.0, // Will be filled by utility function
                // ... other fields
            }
        } else {
            TargetedResult::Pass {
                tests_run: 1,
                utility: 0.0, // Will be filled by utility function  
                // ... other fields
            }
        }
    };
    
    // Neighborhood function - how to generate similar inputs
    let neighborhood = IntegerNeighborhood::new(10);
    
    // Configure the search
    let config = TargetedConfig {
        objective: SearchObjective::Maximize, // Find inputs that maximize utility
        search_steps: 1000,
        initial_temperature: 100.0,
        cooling_rate: 0.95,
        ..Default::default()
    };
    
    let search = for_all_targeted_with_config(
        generator,
        utility_function,
        test_function,
        neighborhood,
        config,
    );
    
    let (result, stats) = search.search(&Config::default());
    
    println!("Found {} evaluations, best utility: {}", 
             stats.evaluations, stats.best_utility);
}
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

# Property classification examples
cargo run --example classification

# Example integration examples
cargo run --example example-integration

# Dictionary support examples
cargo run --example dictionary-support

# Function generator examples
cargo run --example function-generators

# Targeted property testing examples  
cargo run --example targeted-testing

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


## Documentation

- [Property Classification Guide](docs/property-classification.md) - Inspecting test data distribution and statistics
- [Targeted Testing Comparison](docs/targeted-testing-comparison.md) - Comparison with PROPER's approach and implementation choices
- [Targeted Testing Effectiveness Analysis](docs/targeted-testing-effectiveness-analysis.md) - Detailed analysis of systematic search behavior and estimated efficiency gains
- [Targeted Testing Future Improvements](docs/targeted-testing-future-improvements.md) - Roadmap for extending targeted testing capabilities
- [Implementation Plan](docs/implementation-plan.md) - Detailed implementation roadmap
- [Roadmap](docs/roadmap.md) - Project status and future plans
- [Ideas](docs/ideas.md) - Comprehensive feature survey from other property testing libraries

## Contributing

Contributions are welcome! Please see the [roadmap](docs/roadmap.md) for planned features and current progress.

## License

This project is licensed under the BSD-3-Clause License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Jacob Stanley and the original Hedgehog team for the foundational ideas
- The Haskell, F#, and R Hedgehog ports for implementation insights
- The Rust community for excellent tooling and ecosystem support