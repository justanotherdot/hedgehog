# Distribution Shaping

Control probability distributions for realistic test data.

## Problem

Uniform random generation often produces unrealistic test cases:
- Huge numbers when you need small ones
- Empty strings when you need reasonable lengths
- Equal probability for rare and common scenarios

## Solution

Distribution shaping lets you bias generation toward realistic values.

## Basic Usage

```rust
use hedgehog::*;

// Uniform distribution (default)
let uniform = Gen::<i32>::from_range(Range::new(1, 100));

// Linear distribution (favors smaller values)
let linear = Gen::<i32>::from_range(Range::linear(1, 100));

// Exponential distribution (strongly favors smaller values)
let exponential = Gen::<i32>::from_range(Range::exponential(1, 100));

// Constant value
let constant = Gen::<i32>::from_range(Range::constant(42));
```

## String Lengths

```rust
// Realistic string lengths (1-20 characters, favoring shorter)
let short_strings = Gen::<String>::alpha_with_range(Range::exponential(1, 20));

// Controlled length range
let medium_strings = Gen::<String>::alpha_with_range(Range::linear(5, 50));
```

## Frequency-Based Generation

```rust
// Realistic HTTP status codes
let status_codes = Gen::frequency(vec![
    WeightedChoice::new(70, Gen::constant(200)),    // 70% success
    WeightedChoice::new(20, Gen::constant(404)),    // 20% not found
    WeightedChoice::new(10, Gen::constant(500)),    // 10% server error
]);

// Simple choices
let colors = Gen::one_of(vec![
    Gen::constant("red"),
    Gen::constant("green"),
    Gen::constant("blue"),
]);
```

## Range Types

```rust
// Integer ranges
Range::new(1, 100)           // Uniform: 1 to 100
Range::linear(1, 100)        // Linear: favors smaller values
Range::exponential(1, 100)   // Exponential: strongly favors smaller values
Range::constant(42)          // Always 42

// Float ranges
Range::new(0.0, 1.0)         // Unit range
Range::exponential(0.1, 10.0) // Exponential floats

// With origin (shrinks toward this value)
Range::new(1, 100).with_origin(50)
```

## Property Testing

```rust
#[test]
fn test_with_realistic_data() {
    // Test with realistic string lengths
    let prop = for_all_named(
        Gen::<String>::alpha_with_range(Range::exponential(1, 20)),
        "text",
        |text| {
            // Most strings will be short, some medium, few long
            text.len() <= 20
        }
    );
    
    assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));
}
```

## Best Practices

1. **Use exponential for sizes**: String lengths, collection sizes
2. **Use linear for bounded ranges**: Ages, percentages
3. **Use frequency for categories**: Status codes, enum variants
4. **Use uniform sparingly**: Only when all values are equally likely

Distribution shaping makes your tests more realistic and finds bugs faster.