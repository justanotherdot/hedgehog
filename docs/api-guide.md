# Hedgehog API Guide

This guide covers the key features of the Hedgehog property-based testing library for Rust, focusing on distribution shaping and variable name tracking.

## Quick Start

```rust
use hedgehog::*;

// Basic property test
let prop = for_all(Gen::int_range(1, 100), |&n| n > 0);
assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));

// Property test with variable name tracking
let prop = for_all_named(Gen::int_range(1, 100), "n", |&n| n > 0);
assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));

// Property test with distribution shaping
let prop = for_all_named(
    Gen::<i32>::from_range(Range::exponential(1, 100)),
    "n",
    |&n| n > 0
);
assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));
```

## Core Types

### Generators

```rust
pub struct Gen<T> {
    // Internal generator function
}

impl<T> Gen<T> {
    pub fn new<F>(f: F) -> Self
    where F: Fn(Size, Seed) -> Tree<T> + 'static;
    
    pub fn generate(&self, size: Size, seed: Seed) -> Tree<T>;
    pub fn constant(value: T) -> Self where T: Clone + 'static;
    pub fn map<U, F>(self, f: F) -> Gen<U>;
    pub fn bind<U, F>(self, f: F) -> Gen<U>;
    pub fn filter<F>(self, predicate: F) -> Gen<T>;
    
    // Distribution shaping
    pub fn frequency(choices: Vec<WeightedChoice<T>>) -> Gen<T>;
    pub fn one_of(generators: Vec<Gen<T>>) -> Gen<T>;
}
```

### Properties

```rust
pub struct Property<T> {
    // Internal property data
}

impl<T> Property<T> {
    pub fn new<F>(generator: Gen<T>, test_function: F) -> Self;
    pub fn for_all<F>(generator: Gen<T>, condition: F) -> Self;
    pub fn for_all_named<F>(generator: Gen<T>, variable_name: &str, condition: F) -> Self;
    pub fn run(&self, config: &Config) -> TestResult;
    pub fn run_with_context(&self, config: &Config, property_name: Option<&str>, module_path: Option<&str>) -> TestResult;
}
```

### Ranges and Distributions

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
    pub origin: Option<T>,
    pub distribution: Distribution,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distribution {
    Uniform,
    Linear,
    Exponential,
    Constant,
}

impl<T> Range<T> {
    pub fn new(min: T, max: T) -> Self;
    pub fn linear(min: T, max: T) -> Self;
    pub fn exponential(min: T, max: T) -> Self;
    pub fn constant(value: T) -> Self;
    pub fn with_origin(mut self, origin: T) -> Self;
    pub fn contains(&self, value: &T) -> bool;
    pub fn distribution(&self) -> Distribution;
}
```

### Test Results

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestResult {
    Pass { 
        tests_run: usize, 
        property_name: Option<String>, 
        module_path: Option<String> 
    },
    Fail { 
        counterexample: String, 
        tests_run: usize, 
        shrinks_performed: usize, 
        property_name: Option<String>, 
        module_path: Option<String>, 
        assertion_type: Option<String>, 
        shrink_steps: Vec<ShrinkStep> 
    },
    Discard { 
        limit: usize, 
        property_name: Option<String>, 
        module_path: Option<String> 
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShrinkStep {
    pub counterexample: String,
    pub step: usize,
    pub variable_name: Option<String>,
}
```

## Generator Reference

### Primitive Generators

```rust
// Boolean
Gen::bool()

// Integers
Gen::int_range(min: i32, max: i32)
Gen::i64_range(min: i64, max: i64)
Gen::u32_range(min: u32, max: u32)
Gen::<i32>::positive()
Gen::<i32>::natural()

// Floats
Gen::f64_range(min: f64, max: f64)
Gen::<f64>::positive()
Gen::<f64>::natural()
Gen::<f64>::unit()

// Characters
Gen::<char>::ascii_alpha()
Gen::<char>::ascii_alphanumeric()
Gen::<char>::ascii_printable()
```

### Range-Based Generators

```rust
// Numeric generators with distribution control
Gen::<i32>::from_range(Range::new(1, 100))
Gen::<i32>::from_range(Range::linear(1, 100))
Gen::<i32>::from_range(Range::exponential(1, 100))
Gen::<i32>::from_range(Range::constant(42))

// Predefined ranges
Gen::<i32>::from_range(Range::<i32>::positive())
Gen::<i32>::from_range(Range::<i32>::natural())
Gen::<i32>::from_range(Range::<i32>::small_positive())
Gen::<f64>::from_range(Range::<f64>::unit())
Gen::<f64>::from_range(Range::<f64>::positive())
Gen::<f64>::from_range(Range::<f64>::natural())
Gen::<f64>::from_range(Range::<f64>::normal())
```

### String Generators

```rust
// Size-dependent strings
Gen::<String>::ascii_alpha()
Gen::<String>::ascii_alphanumeric()
Gen::<String>::ascii_printable()

// Range-controlled strings
Gen::<String>::with_range(Range::new(1, 10), Gen::<char>::ascii_alpha())
Gen::<String>::alpha_with_range(Range::new(1, 10))
Gen::<String>::alphanumeric_with_range(Range::linear(1, 20))
Gen::<String>::printable_with_range(Range::exponential(1, 50))
```

### Collection Generators

```rust
// Vectors
Gen::<Vec<T>>::vec_of(element_gen)
Gen::<Vec<i32>>::vec_int()
Gen::<Vec<bool>>::vec_bool()

// Options
Gen::<Option<T>>::option_of(inner_gen)

// Tuples
Gen::<(T, U)>::tuple_of(first_gen, second_gen)

// Results
Gen::<Result<T, E>>::result_of(ok_gen, err_gen)
Gen::<Result<T, E>>::result_of_weighted(ok_gen, err_gen, ok_weight)
```

### Frequency-Based Generators

```rust
// Weighted choice
Gen::frequency(vec![
    WeightedChoice::new(7, Gen::constant("common")),
    WeightedChoice::new(2, Gen::constant("uncommon")),
    WeightedChoice::new(1, Gen::constant("rare")),
])

// Equal choice
Gen::one_of(vec![
    Gen::constant("red"),
    Gen::constant("green"),
    Gen::constant("blue"),
])
```

## Property Testing Functions

### Module-Level Functions

```rust
// Basic property testing
pub fn for_all<T, F>(generator: Gen<T>, condition: F) -> Property<T>
where
    T: 'static + std::fmt::Debug,
    F: Fn(&T) -> bool + 'static;

// Property testing with variable names
pub fn for_all_named<T, F>(generator: Gen<T>, variable_name: &str, condition: F) -> Property<T>
where
    T: 'static + std::fmt::Debug,
    F: Fn(&T) -> bool + 'static;

// Generic property creation
pub fn property<T, F>(generator: Gen<T>, test_function: F) -> Property<T>
where
    T: 'static + std::fmt::Debug,
    F: Fn(&T) -> TestResult + 'static;
```

## Configuration

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub test_limit: usize,
    pub shrink_limit: usize,
    pub size_limit: usize,
    pub discard_limit: usize,
}

impl Config {
    pub fn with_tests(mut self, tests: usize) -> Self;
    pub fn with_shrinks(mut self, shrinks: usize) -> Self;
    pub fn with_size_limit(mut self, size: usize) -> Self;
}

impl Default for Config {
    fn default() -> Self {
        Config {
            test_limit: 100,
            shrink_limit: 1000,
            size_limit: 100,
            discard_limit: 100,
        }
    }
}
```

## Complete Examples

### Basic Property Testing

```rust
use hedgehog::*;

fn test_reverse_property() {
    let prop = for_all(
        Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 100)),
        |vec| {
            let reversed: Vec<i32> = vec.iter().rev().cloned().collect();
            let double_reversed: Vec<i32> = reversed.iter().rev().cloned().collect();
            double_reversed == *vec
        }
    );
    
    assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));
}
```

### Property Testing with Variable Names

```rust
use hedgehog::*;

fn test_string_length_property() {
    let prop = for_all_named(
        Gen::<String>::alpha_with_range(Range::linear(1, 50)),
        "text",
        |text| {
            let uppercase = text.to_uppercase();
            uppercase.len() == text.len()
        }
    );
    
    match prop.run(&Config::default()) {
        TestResult::Pass { tests_run } => {
            println!("String length property passed {} tests", tests_run);
        }
        TestResult::Fail { .. } => {
            println!("Property failed:");
            println!("{}", prop.run(&Config::default()));
        }
        _ => {}
    }
}
```

### Distribution Shaping

```rust
use hedgehog::*;

fn test_realistic_user_ages() {
    // Generate realistic age distribution
    let age_gen = Gen::frequency(vec![
        WeightedChoice::new(1, Gen::int_range(0, 17)),      // 10% minors
        WeightedChoice::new(6, Gen::int_range(18, 40)),     // 60% young adults
        WeightedChoice::new(2, Gen::int_range(41, 65)),     // 20% middle-aged
        WeightedChoice::new(1, Gen::int_range(66, 100)),    // 10% seniors
    ]);
    
    let prop = for_all_named(age_gen, "age", |&age| {
        age >= 0 && age <= 100
    });
    
    assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));
}
```

### Complex Properties

```rust
use hedgehog::*;

#[derive(Debug, Clone)]
struct User {
    id: u32,
    name: String,
    age: u32,
    email: String,
}

fn user_generator() -> Gen<User> {
    Gen::new(|size, seed| {
        let (id_seed, rest) = seed.split();
        let (name_seed, rest) = rest.split();
        let (age_seed, email_seed) = rest.split();
        
        let id = Gen::<u32>::from_range(Range::linear(1, 1000000))
            .generate(size, id_seed).outcome();
        let name = Gen::<String>::alpha_with_range(Range::exponential(2, 20))
            .generate(size, name_seed).outcome();
        let age = Gen::<u32>::from_range(Range::linear(18, 80))
            .generate(size, age_seed).outcome();
        let email = format!("{}@example.com", 
            Gen::<String>::alpha_with_range(Range::linear(3, 15))
                .generate(size, email_seed).outcome());
        
        Tree::singleton(User { id, name, age, email })
    })
}

fn test_user_property() {
    let prop = for_all_named(user_generator(), "user", |user| {
        user.id > 0 && 
        !user.name.is_empty() && 
        user.age >= 18 && 
        user.email.contains('@')
    });
    
    assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));
}
```

## Best Practices

### 1. Choose Appropriate Distributions

```rust
// Use exponential for sizes (most things are small)
let size_gen = Gen::<usize>::from_range(Range::exponential(1, 10000));

// Use linear for counts (favor smaller counts)
let count_gen = Gen::<usize>::from_range(Range::linear(0, 100));

// Use uniform for true random sampling
let random_gen = Gen::<i32>::from_range(Range::new(1, 100));
```

### 2. Use Descriptive Variable Names

```rust
// Good
let prop = for_all_named(user_gen, "user", |user| user.is_valid());
let prop = for_all_named(request_gen, "request", |req| req.is_authorized());

// Avoid
let prop = for_all_named(gen, "x", |x| condition(x));
```

### 3. Model Real-World Distributions

```rust
// HTTP status codes
let status_gen = Gen::frequency(vec![
    WeightedChoice::new(70, Gen::constant(200)),    // 70% success
    WeightedChoice::new(15, Gen::constant(404)),    // 15% not found
    WeightedChoice::new(10, Gen::constant(500)),    // 10% server error
    WeightedChoice::new(5, Gen::int_range(300, 399)), // 5% redirects
]);

// File sizes (exponential distribution)
let file_size_gen = Gen::<u64>::from_range(Range::exponential(1, 1_000_000_000));
```

### 4. Use Appropriate Test Counts

```rust
// For expensive properties, use fewer tests
let expensive_prop = for_all_named(complex_gen, "input", expensive_test);
expensive_prop.run(&Config::default().with_tests(10));

// For fast properties, use more tests
let fast_prop = for_all_named(simple_gen, "input", fast_test);
fast_prop.run(&Config::default().with_tests(1000));
```

## Common Patterns

### Testing Invariants

```rust
// Property: sorting doesn't change length
let prop = for_all_named(
    Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 100)),
    "list",
    |list| {
        let mut sorted = list.clone();
        sorted.sort();
        sorted.len() == list.len()
    }
);
```

### Testing Round-Trip Properties

```rust
// Property: serialize/deserialize is identity
let prop = for_all_named(data_gen, "data", |data| {
    let serialized = serialize(data);
    let deserialized = deserialize(&serialized);
    deserialized == Ok(data.clone())
});
```

### Testing Error Conditions

```rust
// Property: invalid input produces appropriate errors
let prop = for_all_named(invalid_input_gen, "input", |input| {
    match process_input(input) {
        Ok(_) => false,  // Should not succeed
        Err(e) => e.is_validation_error(),
    }
});
```

### Testing Performance Properties

```rust
// Property: operation completes within time limit
let prop = for_all_named(input_gen, "input", |input| {
    let start = std::time::Instant::now();
    let _result = expensive_operation(input);
    let duration = start.elapsed();
    duration < std::time::Duration::from_millis(100)
});
```

This API guide provides a comprehensive reference for using Hedgehog's distribution shaping and variable name tracking features effectively.