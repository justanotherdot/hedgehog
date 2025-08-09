# Hedgehog API Guide

This guide covers the key features of the Hedgehog property-based testing library for Rust, focusing on distribution shaping, variable name tracking, and derive macros.

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

// Property test with derive macros
use hedgehog_derive::Generate;

#[derive(Generate, Debug, Clone)]
struct User {
    name: String,
    age: u32,
}

let prop = for_all_named(User::generate(), "user", |user: &User| {
    !user.name.is_empty() && user.age <= 100
});
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

### Function Generators

Function generators are designed for testing **your code** that takes functions as parameters. Instead of testing with a few hardcoded functions, you can test with hundreds of systematically generated functions.

#### The Problem Function Generators Solve

```rust
// Limited manual testing
fn test_my_data_processor() {
    let processor = MyDataProcessor::new();
    
    // Only testing with 3 specific transform functions
    assert_eq!(processor.transform(data.clone(), |x| x * 2).len(), expected1);
    assert_eq!(processor.transform(data.clone(), |x| x + 1).len(), expected2);  
    assert_eq!(processor.transform(data.clone(), |x| if x > 5 { x } else { 0 }).len(), expected3);
    
    // What about edge cases? What about other function patterns?
}
```

#### The Function Generator Solution

```rust
// Systematic testing with generated functions
fn test_data_processor_with_function_generators() {
    // Generate transform functions to test your code with
    let transform_gen = Gen::<Box<dyn Fn(i32) -> i32>>::function_of(
        Gen::int_range(1, 10),    // Keyspace: functions might map inputs 1-10
        Gen::int_range(0, 20),    // Output range: 0-20
        0                         // Default for unmapped inputs
    );
    
    // Combine data and function generators using tuples
    // Note: Hedgehog uses single-generator for_all - combine multiple generators manually
    let test_gen = Gen::new(|size, seed| {
        let (data_seed, func_seed) = seed.split();
        let data = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 10))
            .generate(size, data_seed).value;
        let transform = transform_gen.generate(size, func_seed).value;
        Tree::singleton((data, transform))
    });
    
    let property = for_all(test_gen, |(data, transform)| {
        let result = MyDataProcessor::new().transform(data.clone(), &*transform);
        
        // Test properties of YOUR code with generated functions
        result.len() <= data.len() &&                    // Sanity check
        result.iter().all(|&x| x >= 0) &&              // All outputs valid
        data.is_empty() == result.is_empty()            // Empty preservation
    });
    
    // This tests your code with hundreds of different functions:
    // {1→15, 3→8, 7→2, default→0}, {2→20, 5→1, default→0}, 
    // {} (constant function), and many more patterns!
    property.run(&Config::default());
}
```

#### Real-World Use Cases

**Testing Collection Operations:**
```rust
// Your code that processes collections with user functions
pub fn filter_map_collect<T, U>(
    items: Vec<T>, 
    filter_fn: impl Fn(&T) -> bool,
    map_fn: impl Fn(T) -> U
) -> Vec<U> {
    items.into_iter()
        .filter(filter_fn)
        .map(map_fn)
        .collect()
}

// Test with generated predicates and mappers
let predicate_gen = Gen::<Box<dyn Fn(i32) -> bool>>::predicate_from_set(
    Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 5))
);
let mapper_gen = Gen::<Box<dyn Fn(i32) -> String>>::function_of(
    Gen::int_range(1, 10),
    Gen::<String>::ascii_alpha(),
    "unknown".to_string()
);

// Property: filtered results should all satisfy the predicate (when mapped back)
```

**Testing Sorting with Custom Comparators:**
```rust
// Your sorting function
pub fn custom_sort<T>(mut items: Vec<T>, cmp: impl Fn(&T, &T) -> std::cmp::Ordering) -> Vec<T> {
    items.sort_by(cmp);
    items
}

// Test with generated comparators
let comparator_gen = Gen::<Box<dyn Fn(i32, i32) -> std::cmp::Ordering>>::comparator_from_choices(vec![
    std::cmp::Ordering::Less,
    std::cmp::Ordering::Equal,
    std::cmp::Ordering::Greater,
]);

// Property: sorted result should be sorted according to the comparator
let property = for_all(combined_gen, |(data, cmp)| {
    let sorted = custom_sort(data, &*cmp);
    // Check that adjacent elements are properly ordered
    sorted.windows(2).all(|pair| cmp(pair[0], pair[1]) != std::cmp::Ordering::Greater)
});
```

#### Function Generator Types

```rust
// Generate functions from lookup tables (most common)
let function_gen = Gen::<Box<dyn Fn(i32) -> String>>::function_of(
    Gen::int_range(0, 5),         // Input keyspace
    Gen::<String>::ascii_alpha(), // Output generator  
    "default".to_string()         // Default for unmapped inputs
);

// Generate constant functions
let constant_func_gen = Gen::<Box<dyn Fn(i32) -> String>>::constant_function(
    Gen::constant("hello".to_string())
);

// Generate identity functions (for compatible types)
let identity_gen = Gen::<Box<dyn Fn(i32) -> i32>>::identity_function();

// Generate binary functions  
let binary_func_gen = Gen::<Box<dyn Fn(i32, i32) -> String>>::binary_function_of(
    Gen::int_range(0, 3), Gen::int_range(0, 3), 
    Gen::<String>::ascii_alpha(), "default".to_string()
);

// Generate predicate functions
let predicate_gen = Gen::<Box<dyn Fn(i32) -> bool>>::predicate_from_set(
    Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 5))  // Accept these values
);

// Generate comparator functions
let comparator_gen = Gen::<Box<dyn Fn(i32, i32) -> std::cmp::Ordering>>::constant_comparator(
    std::cmp::Ordering::Equal  // Always return Equal
);
```

#### How Function Generation Works

Function generators use lookup tables (HashMap) internally to create **finite but representative** functions:

```rust
// A generated function might behave like this internally:
// lookup_table = {1 → "apple", 3 → "banana", 7 → "cherry"}
// default = "unknown"

let f = /* generated function */;
println!("{}", f(1));   // "apple"   (found in lookup table)
println!("{}", f(3));   // "banana"  (found in lookup table)  
println!("{}", f(5));   // "unknown" (not in table, uses default)
println!("{}", f(99));  // "unknown" (not in table, uses default)
```

**Why This Works:**
- **Deterministic:** Same input always produces the same output
- **Shrinkable:** Large tables shrink to small tables, then to constant functions  
- **Representative:** Models real-world patterns (dispatch tables, configuration maps, routing tables)
- **Debuggable:** When tests fail, you see the actual lookup table that caused the failure

**The Keyspace:** The first argument to `function_of` defines which inputs *might* get specific mappings. Larger keyspaces create more varied function behavior:

```rust
// Small keyspace: only 1,2,3 might be mapped to specific outputs
Gen::function_of(Gen::int_range(1, 3), output_gen, default)

// Large keyspace: any number 1-100 might be mapped  
Gen::function_of(Gen::int_range(1, 100), output_gen, default)
```

#### Key Insight: Testing Function Composition

Function generators excel at finding edge cases in code that chains operations:

```rust
// Your pipeline function
pub fn process_pipeline<T>(
    data: Vec<T>,
    validator: impl Fn(&T) -> bool,
    transformer: impl Fn(T) -> T,
    finalizer: impl Fn(T) -> T,
) -> Vec<T> {
    data.into_iter()
        .filter(validator)       // Step 1: filter
        .map(transformer)        // Step 2: transform  
        .map(finalizer)          // Step 3: finalize
        .collect()
}

// Generate each function in the pipeline
let validator_gen = Gen::<Box<dyn Fn(i32) -> bool>>::predicate_from_set(/*...*/);
let transformer_gen = Gen::<Box<dyn Fn(i32) -> i32>>::function_of(/*...*/);
let finalizer_gen = Gen::<Box<dyn Fn(i32) -> i32>>::function_of(/*...*/);

// Test that your pipeline behaves correctly with ANY combination of functions
// This finds interaction bugs between pipeline stages that manual testing misses!
```

Function generators turn "testing with a few examples" into "testing with systematic exploration of the function space."

#### Scope and Limitations

**What We Generate: First-Order Functions**

Hedgehog generates **first-order functions** - functions that take regular values and return regular values:

```rust
// ✅ First-order functions (what we generate)
Gen<Box<dyn Fn(i32) -> String>>         // Takes i32, returns String
Gen<Box<dyn Fn(User) -> bool>>           // Takes User, returns bool  
Gen<Box<dyn Fn(i32, i32) -> Ordering>>  // Takes two i32s, returns Ordering
```

**What We Don't Generate: Higher-Order Functions**

We do **not** generate higher-order functions - functions that take or return other functions:

```rust
// ❌ Higher-order functions (too complex for Rust's ownership model)
Gen<Box<dyn Fn(Box<dyn Fn(i32) -> String>) -> Box<dyn Fn(i32) -> bool>>>>
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//           Takes a function, returns a function
```

**Why Not Higher-Order?**

Rust's ownership model makes higher-order function generation impractical:
- Functions don't implement `Clone`, making shrinking impossible
- Nested closures create lifetime conflicts  
- Multiple levels of `Box<dyn Fn>` become unwieldy
- Can't inspect or debug function arguments

**Workaround: The Tuple Trick**

For higher-order testing, generate the "configuration" and let users curry:

```rust
// Instead of generating: (A -> B) -> (A -> C)  
// Generate the transformation configuration:
type TransformConfig = HashMap<(i32, String), bool>;
let config_gen = Gen::<TransformConfig>::new(/* ... */);

// Users curry this into higher-order forms:
let make_transformer = |config: TransformConfig| {
    move |f: Box<dyn Fn(i32) -> String>| {
        Box::new(move |x: i32| {
            let intermediate = f(x);
            config.get(&(x, intermediate)).copied().unwrap_or(false)
        })
    }
};

// Now test your higher-order code:
let transformer = make_transformer(config_gen.sample());
let result_fn = transformer(some_first_order_function);
```

This gives you the power of higher-order testing while working within Rust's constraints.

## Derive Macros

Hedgehog provides derive macros for automatic generator creation. Enable with the `derive` feature:

```toml
[dependencies]
hedgehog = { version = "0.1.0", features = ["derive"] }
```

### Basic Usage

```rust
use hedgehog::*;
use hedgehog_derive::Generate;

#[derive(Generate, Debug, Clone)]
struct User {
    name: String,
    age: u32,
    active: bool,
}

// Automatically generates:
// impl User {
//     pub fn generate() -> Gen<Self> { ... }
// }

let user_gen = User::generate();
```

### Supported Types

- **Named structs**: `struct User { name: String, age: u32 }`
- **Tuple structs**: `struct Point(i32, i32)`
- **Unit structs**: `struct Unit`
- **Enums**: All variant types (unit, tuple, named fields)

### Built-in Type Mappings

| Type | Generator | Range |
|------|-----------|--------|
| `String` | `Gen::<String>::ascii_alpha()` | Variable length |
| `i32`, `u32`, `i64` | `Gen::from_range(Range::new(0, 100))` | 0 to 100 |
| `f64` | `Gen::from_range(Range::new(0.0, 100.0))` | 0.0 to 100.0 |
| `bool` | `Gen::bool()` | true/false |
| `char` | `Gen::<char>::ascii_alpha()` | a-z, A-Z |
| `u8`, `u16`, `i8`, `i16`, `f32` | Mapped from larger types | Type-appropriate ranges |

### Custom Types

```rust
#[derive(Generate, Debug, Clone)]
struct Address {
    street: String,
    city: String,
}

#[derive(Generate, Debug, Clone)]
struct Person {
    name: String,
    address: Address,  // Uses Address::generate()
}
```

### Complete Example

```rust
#[derive(Generate, Debug, Clone)]
enum PaymentMethod {
    Cash,
    Card { number: String, expiry: String },
    Digital(String),
}

#[derive(Generate, Debug, Clone)]
struct Order {
    id: u32,
    payment: PaymentMethod,
    total: f64,
}

fn test_order_validation() {
    let order_prop = for_all_named(Order::generate(), "order", |order: &Order| {
        order.total >= 0.0 && order.id > 0
    });
    
    match order_prop.run(&Config::default()) {
        TestResult::Pass { .. } => println!("✓ Order validation passed"),
        TestResult::Fail { counterexample, .. } => {
            println!("✗ Failed with order: {}", counterexample);
        }
        result => println!("Unexpected result: {:?}", result),
    }
}
```

For comprehensive documentation, see `docs/derive-macros.md`.

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

## Example Integration

Example integration allows mixing explicit test examples with property-based testing to ensure critical edge cases are always tested while getting broad coverage from generated values.

### Basic Usage

```rust
use hedgehog::*;

// Test a parsing function with known problematic inputs
let problematic_cases = vec![
    "".to_string(),           // Empty string
    "-1".to_string(),         // Negative number  
    "abc".to_string(),        // Non-numeric
    "4294967296".to_string(), // Overflow
];

let prop = for_all(
    Gen::<String>::ascii_printable(),
    |s| parse_number(s).is_ok() || parse_number(s).is_err() // No panics
).with_examples(problematic_cases);

// Examples are tested first, then random strings are generated
prop.run(&Config::default());
```

### How Example Integration Works with Test Iterations

Examples are integrated into the **total test count** specified by `Config.test_limit` (default: 100). They don't add extra tests - instead, examples take up some of those test iterations.

```rust
// With 10 total tests and 3 examples:
let config = Config::default().with_tests(10);
let examples = vec![0, -1, i32::MAX];

// ExamplesFirst strategy:
// Test sequence: [ex1, ex2, ex3, gen1, gen2, gen3, gen4, gen5, gen6, gen7] 
// Total: exactly 10 tests (3 examples + 7 generated)

let prop = for_all(gen, test_fn)
    .with_examples(examples)
    .run(&config); // Always runs exactly 10 tests
```

**Key Points:**
- **Fixed Total**: Always runs exactly `config.test_limit` tests  
- **No Extra Cost**: Examples don't increase test time beyond your configured limit
- **Guaranteed Coverage**: Critical examples always tested within your test budget
- **Early Termination**: If any test fails (example or generated), testing stops immediately

### Example Integration Strategies

Control how examples are mixed with generated values using `ExampleStrategy`:

```rust
use hedgehog::property::ExampleStrategy;

let examples = vec![0, -1, i32::MAX];

// Strategy 1: Examples first (default)
let prop1 = for_all(gen, test_fn)
    .with_examples(examples.clone());

// Strategy 2: Mixed throughout testing
let prop2 = for_all(gen, test_fn)
    .with_examples_strategy(examples.clone(), ExampleStrategy::Mixed);

// Strategy 3: Generated values first, then examples
let prop3 = for_all(gen, test_fn)
    .with_examples_strategy(examples.clone(), ExampleStrategy::GeneratedFirst);

// Strategy 4: Examples only for first N tests
let prop4 = for_all(gen, test_fn)
    .with_examples_strategy(examples.clone(), ExampleStrategy::ExamplesUpTo(5));
```

### Strategy Descriptions

#### `ExamplesFirst` (Default)
Tests all examples first, then generates random values:
```
// With 8 tests and 3 examples:
Test sequence: [ex1, ex2, ex3, gen1, gen2, gen3, gen4, gen5]
Examples used: 3/8 = 37.5% of tests
Generated: 5/8 = 62.5% of tests
```

**Best for:**
- Ensuring critical cases are always tested first
- Quick feedback on known edge cases
- Regression testing

#### `Mixed` 
Distributes examples throughout the test run:
```
// With 10 tests and 3 examples (approximate pattern):
Test sequence: [ex1, gen1, ex2, gen2, ex3, gen3, gen4, gen5, gen6, gen7]
Examples distributed throughout, not just at start
```

**Best for:**
- Balanced coverage throughout testing
- When examples represent typical rather than edge cases
- Simulating realistic usage patterns

#### `GeneratedFirst`
Generates random values first, then tests examples, then continues generating:
```
// With 10 tests and 3 examples:
// Threshold = min(3, 10) = 3, so examples start at test 4
Test sequence: [gen1, gen2, gen3, ex1, ex2, ex3, gen4, gen5, gen6, gen7]
Random exploration → targeted validation → more exploration
```

**Best for:**
- Exploring the input space before testing known cases
- When examples are expensive to execute
- Complement random exploration with specific validation

#### `ExamplesUpTo(n)`
Uses examples only for the first `n` tests:
```
// With 10 tests, 5 examples, and ExamplesUpTo(3):
Test sequence: [ex1, ex2, ex3, gen1, gen2, gen3, gen4, gen5, gen6, gen7]
Only first 3 examples used, remaining 2 examples ignored
```

**Best for:**
- Limited number of critical examples
- Hybrid approach with controlled example usage
- Performance-sensitive testing with expensive examples

### Real-World Use Cases

#### Regression Testing
Capture bugs that occurred in the past:

```rust
// Historical failures that should never happen again
let regression_cases = vec![
    vec![], // Empty input caused panic in v1.0
    vec![0], // Zero handling bug in v1.1
    vec![i32::MIN], // Overflow bug in v1.2
];

let prop = for_all(
    Gen::<Vec<i32>>::vec_of(Gen::int_range(-100, 100)),
    |data| process_data(data).len() <= data.len() // Never grows
).with_examples(regression_cases);
```

#### API Testing
Test web APIs with known problematic requests:

```rust
let problematic_requests = vec![
    Request::new("GET", "/", ""), // Empty body
    Request::new("POST", "/users", "{}"), // Empty JSON
    Request::new("PUT", "/data", "invalid"), // Invalid JSON
    Request::new("DELETE", "/admin", ""), // Sensitive endpoint
];

let prop = for_all(
    request_generator(),
    |req| api_handler(req).status_code < 500 // No server errors
).with_examples(problematic_requests);
```

#### Mathematical Functions
Ensure edge cases in mathematical operations:

```rust
let critical_values = vec![
    (0, 1), (1, 0), // Identity cases
    (i32::MAX, 1), (i32::MIN, -1), // Overflow cases
    (100, 0), // Division by zero
];

let prop = for_all(
    Gen::<(i32, i32)>::tuple_of(Gen::int_range(-50, 50), Gen::int_range(-10, 10)),
    |&(a, b)| {
        match safe_divide(a, b) {
            Some(result) => b != 0 && result == a / b,
            None => b == 0 || (a == i32::MIN && b == -1)
        }
    }
).with_examples(critical_values);
```

#### File System Operations
Test with problematic file paths:

```rust
let problematic_paths = vec![
    "".to_string(), // Empty path
    "/".to_string(), // Root
    "...".to_string(), // Multiple dots
    "file\0name".to_string(), // Null byte
    "very_long_filename_that_exceeds_limits".repeat(10), // Long name
];

let prop = for_all(
    path_generator(),
    |path| file_operation(path).is_ok() || file_operation(path).is_err()
).with_examples(problematic_paths);
```

### Integration with Property Classification

Combine examples with classification to analyze coverage:

```rust
let prop = for_all(
    Gen::int_range(-100, 100),
    |&x| x.abs() >= 0 // Always true
)
.with_examples(vec![0, -1, i32::MIN])
.classify("negative", |&x| x < 0)
.classify("zero", |&x| x == 0) 
.classify("positive", |&x| x > 0)
.classify("from_example", |&x| [0, -1, i32::MIN].contains(&x));

match prop.run(&Config::default()) {
    TestResult::PassWithStatistics { statistics, .. } => {
        // Shows both generated distribution and example usage
        println!("Examples used: {}%", 
            statistics.classifications.get("from_example").unwrap_or(&0));
    }
    _ => {}
}
```

### Best Practices

#### Choose Appropriate Examples
```rust
// Good: Critical edge cases that are hard to generate randomly
let good_examples = vec![
    "", // Empty string
    "\0", // Null character
    "€🦀", // Unicode edge cases
    format!("{}", "a".repeat(10000)), // Very long input
];

// Avoid: Random typical cases (let the generator handle these)
let avoid_examples = vec![
    "hello", "world", "test123" // Generator will find these naturally
];
```

#### Use Examples for Known Failures
```rust
// Convert discovered failures into permanent examples
let discovered_failures = vec![
    UserInput { name: "", age: -1 }, // Found by fuzzer
    UserInput { name: "x".repeat(1000), age: 0 }, // Load testing failure
];

let prop = for_all(user_generator(), |user| validate_user(user))
    .with_examples(discovered_failures);
```

#### Combine with Shrinking
Examples work seamlessly with Hedgehog's shrinking:

```rust
// If an example fails, it shrinks normally
let prop = for_all(gen, |&x| x > 0)
    .with_examples(vec![-5]); // This will fail and shrink toward 0

// Output shows shrinking progression starting from the example
```

Example integration provides the best of both worlds: comprehensive property-based testing with guaranteed coverage of critical cases.

This API guide provides a comprehensive reference for using Hedgehog's distribution shaping and variable name tracking features effectively.