# getting started

This guide will help you start using Hedgehog for property-based testing in Rust.

## installation

Add Hedgehog to your `Cargo.toml`:

```toml
[dev-dependencies]
hedgehog = "0.1"
```

For derive macro support:

```toml
[dev-dependencies]
hedgehog = { version = "0.1", features = ["derive"] }
```

## your first property test

Let's write a simple property test that verifies reversing a vector twice returns the original:

```rust
use hedgehog::*;

#[test]
fn prop_reverse_twice() {
    // Create a generator for vectors of integers
    let gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 100));

    // Define the property
    let prop = for_all(gen, |xs: &Vec<i32>| {
        let reversed: Vec<_> = xs.iter().rev().cloned().collect();
        let double_reversed: Vec<_> = reversed.iter().rev().cloned().collect();
        *xs == double_reversed
    });

    // Run the test
    match prop.run(&Config::default()) {
        TestResult::Pass { .. } => (), // Test passed
        result => panic!("Property failed: {:?}", result),
    }
}
```

Run your test:

```sh
cargo test
```

## core concepts

### generators

Generators (`Gen<T>`) produce random values of type `T`. Hedgehog provides many built-in generators:

```rust
Gen::int_range(1, 100)           // integers from 1 to 100
Gen::<String>::ascii_alpha()     // alphabetic strings
Gen::<Vec<i32>>::vec_of(gen)     // vectors using another generator
Gen::<bool>::bool()              // booleans
```

### properties

Properties are testable assertions about your code:

```rust
// Simple property
let prop = for_all(Gen::int_range(0, 100), |&x| {
    x >= 0 && x <= 100
});

// Named property (better error messages)
let prop = for_all_named(Gen::int_range(0, 100), "n", |&n| {
    n >= 0 && n <= 100
});
```

### shrinking

When a test fails, Hedgehog automatically finds the minimal failing case:

```rust
let prop = for_all(Gen::int_range(-100, 100), |&x| {
    x != 0  // Will fail
});

// Hedgehog will shrink to find x = 0 as the minimal counterexample
```

### configuration

Customize test runs with `Config`:

```rust
let config = Config::default()
    .with_tests(1000)           // Run 1000 tests
    .with_shrinks(500)          // Try up to 500 shrinks
    .with_size_limit(100);      // Maximum generator size

prop.run(&config);
```

## common patterns

### testing with examples

Mix specific test cases with random generation:

```rust
let critical_cases = vec![
    (10, 0),        // Division by zero
    (i32::MAX, 1),  // Maximum value
    (i32::MIN, -1), // Overflow
];

let prop = for_all(
    Gen::<(i32, i32)>::tuple_of(
        Gen::int_range(-50, 50),
        Gen::int_range(-5, 5)
    ),
    |&(a, b)| {
        // Your property here
        safe_divide(a, b).is_some() == (b != 0)
    }
).with_examples(critical_cases);
```

### realistic data distributions

Use `Range::exponential` for realistic distributions:

```rust
// Favor shorter strings (more realistic for testing)
let gen = Gen::<String>::alpha_with_range(Range::exponential(1, 50));

// Or use frequency weights
let status_gen = Gen::frequency(vec![
    WeightedChoice::new(70, Gen::constant(200)),  // 70% success
    WeightedChoice::new(15, Gen::constant(404)),  // 15% not found
    WeightedChoice::new(10, Gen::constant(500)),  // 10% errors
    WeightedChoice::new(5, Gen::int_range(300, 399)),
]);
```

### custom generators

Combine generators with `map` and `flat_map`:

```rust
// Map: transform generated values
let email_gen = Gen::<String>::ascii_alpha()
    .map(|name| format!("{name}@example.com"));

// FlatMap: generate dependent values
let range_gen = Gen::int_range(1, 100).flat_map(|max| {
    Gen::int_range(0, max)
});
```

## next steps

- [API Reference](api.md) - Complete API documentation
- [Distribution Shaping](distribution-shaping.md) - Control probability distributions
- [Advanced Features](advanced-features.md) - State machines, parallel testing, targeted testing
- [Examples](../../examples/) - Browse example code

## common issues

**Test takes too long:**
```rust
// Reduce test count or size limit
Config::default().with_tests(100).with_size_limit(50)
```

**Counterexamples aren't minimal:**
```rust
// Increase shrink limit
Config::default().with_shrinks(1000)
```

**Need deterministic tests:**
```rust
// Use a fixed seed
Config::default().with_seed(Seed::new(12345, 67890))
```
