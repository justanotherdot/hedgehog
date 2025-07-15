# Distribution Shaping and Range System

The distribution shaping system in Hedgehog provides fine-grained control over how test data is generated, allowing you to create realistic distributions that better model real-world scenarios.

## Overview

Traditional property-based testing often generates data uniformly across the entire range, which can lead to unrealistic test cases. Hedgehog's distribution shaping allows you to:

- **Control probability distributions** for more realistic test data
- **Bias generation towards edge cases** or common values
- **Eliminate pathological cases** like overly large or empty collections
- **Model real-world data patterns** in your property tests

## Core Concepts

### Distribution Types

Hedgehog provides four fundamental distribution shapes:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distribution {
    /// Uniform distribution across the range
    Uniform,
    /// Linear distribution favoring smaller values
    Linear,
    /// Exponential distribution strongly favoring smaller values
    Exponential,
    /// Constant distribution (always generates the same value)
    Constant,
}
```

### Range System

The `Range<T>` type combines bounds with distribution shapes:

```rust
pub struct Range<T> {
    pub min: T,           // Lower bound (inclusive)
    pub max: T,           // Upper bound (inclusive)
    pub origin: Option<T>, // Origin point for shrinking
    pub distribution: Distribution,
}
```

## Distribution Characteristics

### Uniform Distribution

Generates values with equal probability across the entire range.

```rust
let uniform_range = Range::new(1, 100);  // Equal probability for all values 1-100
let gen = Gen::<i32>::from_range(uniform_range);
```

**Use cases:**
- Testing with truly random data
- Boundary testing where all values are equally important
- Baseline comparisons

### Linear Distribution

Favors smaller values with linearly decreasing probability.

```rust
let linear_range = Range::linear(1, 100);  // Higher probability for smaller values
let gen = Gen::<i32>::from_range(linear_range);
```

**Mathematical model**: P(x) ∝ (max - x)  
**Use cases:**
- Length-based properties (most strings are short)
- Resource allocation (most requests are small)
- Natural number distributions

### Exponential Distribution

Strongly favors smaller values with exponentially decreasing probability.

```rust
let exponential_range = Range::exponential(1, 1000);  // Very high probability for small values
let gen = Gen::<i32>::from_range(exponential_range);
```

**Mathematical model**: P(x) ∝ e^(-λx)  
**Use cases:**
- Edge case testing (focus on boundaries)
- Performance testing (mostly small inputs, occasional large ones)
- Real-world distributions (file sizes, network packets)

### Constant Distribution

Always generates the same value.

```rust
let constant_range = Range::constant(42);  // Always generates 42
let gen = Gen::<i32>::from_range(constant_range);
```

**Use cases:**
- Regression testing with known values
- Isolating specific scenarios
- Performance benchmarking

## API Reference

### Range Construction

```rust
// Basic constructors
Range::new(min, max)              // Uniform distribution
Range::linear(min, max)           // Linear distribution
Range::exponential(min, max)      // Exponential distribution
Range::constant(value)            // Constant distribution

// With custom origin for shrinking
Range::new(min, max).with_origin(origin)
```

### Predefined Ranges

#### Integer Ranges
```rust
// i32 ranges
Range::<i32>::positive()      // [1, i32::MAX] with linear distribution
Range::<i32>::natural()       // [0, i32::MAX] with linear distribution  
Range::<i32>::small_positive() // [1, 100] with uniform distribution

// i64 ranges
Range::<i64>::positive()      // [1, i64::MAX] with linear distribution
Range::<i64>::natural()       // [0, i64::MAX] with linear distribution

// u32 ranges
Range::<u32>::positive()      // [1, u32::MAX] with linear distribution
Range::<u32>::natural()       // [0, u32::MAX] with linear distribution
```

#### Floating Point Ranges
```rust
Range::<f64>::unit()          // [0.0, 1.0] with uniform distribution
Range::<f64>::positive()      // [f64::EPSILON, f64::MAX] with exponential distribution
Range::<f64>::natural()       // [0.0, f64::MAX] with linear distribution
Range::<f64>::normal()        // [-3.0, 3.0] with uniform distribution
```

### Generator Integration

#### Numeric Generators
```rust
// Using predefined ranges
let gen = Gen::<i32>::from_range(Range::positive());

// Using custom ranges
let gen = Gen::<i32>::from_range(Range::exponential(1, 1000));

// For different numeric types
let int_gen = Gen::<i32>::from_range(Range::linear(1, 100));
let float_gen = Gen::<f64>::from_range(Range::unit());
```

#### String Generators
```rust
// General string generation with custom character generator
let gen = Gen::<String>::with_range(
    Range::linear(1, 20),           // Length range with linear distribution
    Gen::<char>::ascii_alpha()      // Character generator
);

// Convenience methods
let gen = Gen::<String>::alpha_with_range(Range::linear(1, 20));
let gen = Gen::<String>::alphanumeric_with_range(Range::exponential(1, 50));
let gen = Gen::<String>::printable_with_range(Range::new(5, 15));
```

## Frequency-Based Generation

For more complex distributions, use frequency-based generators:

### Weighted Choice

```rust
let gen = Gen::frequency(vec![
    WeightedChoice::new(1, Gen::constant(0)),           // 10% zeros
    WeightedChoice::new(9, Gen::int_range(1, 100)),     // 90% positive numbers
]);
```

### Equal Choice

```rust
let gen = Gen::one_of(vec![
    Gen::constant("small"),
    Gen::constant("medium"), 
    Gen::constant("large"),
]);
```

### Complex Distributions

```rust
// Realistic string length distribution
let string_gen = Gen::frequency(vec![
    WeightedChoice::new(1, Gen::constant(String::new())),                    // 5% empty
    WeightedChoice::new(4, Gen::<String>::alpha_with_range(Range::new(1, 5))), // 20% short
    WeightedChoice::new(10, Gen::<String>::alpha_with_range(Range::linear(5, 20))), // 50% medium
    WeightedChoice::new(4, Gen::<String>::alpha_with_range(Range::exponential(20, 100))), // 20% long
    WeightedChoice::new(1, Gen::<String>::alpha_with_range(Range::new(100, 1000))), // 5% very long
]);
```

## Practical Examples

### Example 1: Realistic User Data

```rust
// Generate realistic user ages (most people are 20-40, few are very young or old)
let age_gen = Gen::frequency(vec![
    WeightedChoice::new(1, Gen::int_range(0, 17)),      // 10% minors
    WeightedChoice::new(6, Gen::int_range(18, 40)),     // 60% young adults
    WeightedChoice::new(2, Gen::int_range(41, 65)),     // 20% middle-aged
    WeightedChoice::new(1, Gen::int_range(66, 100)),    // 10% seniors
]);

// Generate realistic usernames (mostly short, occasionally long)
let username_gen = Gen::<String>::alpha_with_range(Range::exponential(3, 20));

// Test user registration
let user_gen = Gen::new(|size, seed| {
    let (age_seed, username_seed) = seed.split();
    let age = age_gen.generate(size, age_seed).outcome();
    let username = username_gen.generate(size, username_seed).outcome();
    
    Tree::singleton(User { age, username })
});
```

### Example 2: Database Performance Testing

```rust
// Generate realistic database queries
let query_gen = Gen::frequency(vec![
    WeightedChoice::new(7, small_query_gen()),      // 70% small queries
    WeightedChoice::new(2, medium_query_gen()),     // 20% medium queries  
    WeightedChoice::new(1, large_query_gen()),      // 10% large queries
]);

// Generate realistic batch sizes (mostly small, occasionally large)
let batch_size_gen = Gen::<usize>::from_range(Range::exponential(1, 10000));

// Test database performance
let prop = for_all(query_gen, |query| {
    let execution_time = database.execute(query);
    execution_time < Duration::from_secs(30)  // Should complete within 30s
});
```

### Example 3: Web API Testing

```rust
// Generate realistic request payloads
let payload_gen = Gen::frequency(vec![
    WeightedChoice::new(8, small_payload_gen()),    // 80% small payloads
    WeightedChoice::new(1, medium_payload_gen()),   // 10% medium payloads
    WeightedChoice::new(1, large_payload_gen()),    // 10% large payloads
]);

// Generate realistic concurrent users (mostly low, occasionally high)
let concurrent_users_gen = Gen::<u32>::from_range(Range::exponential(1, 1000));

// Test API under load
let prop = for_all_named(concurrent_users_gen, "users", |&users| {
    let response_time = api.handle_concurrent_requests(users);
    response_time < Duration::from_millis(500)  // Should respond within 500ms
});
```

## Best Practices

### 1. Model Real-World Distributions

```rust
// Bad: Uniform distribution doesn't reflect reality
let file_size_gen = Gen::int_range(1, 1_000_000_000);  // 1 byte to 1GB uniformly

// Good: Exponential distribution reflects real file sizes
let file_size_gen = Gen::<u64>::from_range(Range::exponential(1, 1_000_000_000));
```

### 2. Use Frequency for Complex Patterns

```rust
// Model realistic HTTP status codes
let status_gen = Gen::frequency(vec![
    WeightedChoice::new(70, Gen::constant(200)),    // 70% success
    WeightedChoice::new(10, Gen::constant(404)),    // 10% not found
    WeightedChoice::new(5, Gen::constant(500)),     // 5% server error
    WeightedChoice::new(15, Gen::int_range(300, 399)), // 15% redirects
]);
```

### 3. Avoid Pathological Cases

```rust
// Bad: Can generate very large collections
let list_gen = Gen::<Vec<i32>>::vec_int();

// Good: Reasonable size limits with realistic distribution
let list_gen = Gen::new(|size, seed| {
    let length_range = Range::linear(0, size.get().min(100));
    let length_gen = Gen::<usize>::from_range(length_range);
    let item_gen = Gen::int_range(1, 1000);
    
    // Generate vector with controlled size
    // ... implementation
});
```

### 4. Use Origins for Better Shrinking

```rust
// Configure shrinking to prefer common values
let positive_gen = Gen::<i32>::from_range(
    Range::exponential(1, 1000).with_origin(1)  // Shrink towards 1
);

let percentage_gen = Gen::<f64>::from_range(
    Range::new(0.0, 1.0).with_origin(0.5)  // Shrink towards 50%
);
```

## Migration Guide

### From Uniform Generation

```rust
// Before
let gen = Gen::int_range(1, 100);

// After - choose appropriate distribution
let gen = Gen::<i32>::from_range(Range::linear(1, 100));        // Favor smaller values
let gen = Gen::<i32>::from_range(Range::exponential(1, 100));   // Strongly favor smaller values
let gen = Gen::<i32>::from_range(Range::new(1, 100));           // Keep uniform if needed
```

### From Size-Dependent Generation

```rust
// Before - size-dependent string generation
let string_gen = Gen::<String>::ascii_alpha();  // Length depends on Size parameter

// After - explicit length control
let string_gen = Gen::<String>::alpha_with_range(Range::linear(1, 20));  // Explicit length range
```

## Performance Considerations

### Distribution Sampling Overhead

- **Uniform**: O(1) - single random number generation
- **Linear**: O(1) - two random numbers, take minimum
- **Exponential**: O(1) - lookup table for small ranges
- **Constant**: O(1) - no randomness required

### Memory Usage

- **Range<T>**: 32 bytes (4 fields × 8 bytes average)
- **WeightedChoice<T>**: 24 bytes + size of wrapped generator
- **Distribution enum**: 1 byte

### Recommendation

Distribution shaping has minimal performance impact and provides significant benefits for test quality. Use it liberally for more realistic and effective property testing.

## Troubleshooting

### Common Issues

1. **Empty ranges**: Ensure `min <= max` for all ranges
2. **Overflow**: Use appropriate numeric types for your ranges
3. **Slow generation**: Very large exponential ranges may be slow
4. **Unexpected distributions**: Verify weights in frequency generators

### Debugging Tips

```rust
// Inspect generated values
let gen = Gen::<i32>::from_range(Range::exponential(1, 100));
for i in 0..10 {
    let seed = Seed::from_u64(i);
    let value = gen.generate(Size::new(50), seed).outcome();
    println!("Generated: {}", value);
}

// Test distribution shape
let values: Vec<i32> = (0..1000)
    .map(|i| gen.generate(Size::new(50), Seed::from_u64(i)).outcome())
    .collect();

let avg = values.iter().sum::<i32>() as f64 / values.len() as f64;
println!("Average: {:.2} (should be lower for exponential)", avg);
```

This distribution shaping system provides the foundation for realistic, effective property-based testing that better reflects real-world scenarios and catches more meaningful bugs.