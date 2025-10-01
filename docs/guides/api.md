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

## Dictionary Support

Dictionary support allows you to inject domain-specific realistic values into your property-based tests. This is essential for testing with meaningful data that exercises real-world edge cases and scenarios.

### Design Philosophy

Hedgehog's dictionary support follows key design principles that distinguish it from other approaches:

#### 1. **Composable Architecture**
All built-in domain generators are constructed from the same core API primitives (`from_elements`, `from_dictionary`) that users have access to. This means:

- **No magic**: Users can recreate any built-in generator
- **Consistent behavior**: All generators follow the same patterns
- **Extensibility**: Users can build complex generators by composing simple ones

```rust
// Built-in HTTP generator uses the same API you have access to:
Gen::from_dictionary(
    common_statuses,           // ← from_elements under the hood
    Gen::int_range(100, 599),  // ← existing generator
    85, 15                     // ← explicit weights
).unwrap()
```

#### 2. **Explicit Over Implicit**
Following Hedgehog's core philosophy, dictionary support avoids "magic" in favor of explicit, understandable behavior:

- **Explicit weights**: `Gen::from_dictionary(dict, random, 80, 20)` - clear probability distribution
- **Explicit dictionaries**: No hidden/global dictionaries, you provide the values
- **Explicit composition**: Combine generators explicitly rather than through annotations or reflection

#### 3. **Realistic Data Distribution**
Dictionary generators reflect real-world frequency patterns, not uniform distributions:

```rust
// HTTP status codes: heavily weight common codes, include rare ones
Gen::from_dictionary(common_codes, all_valid_codes, 85, 15)

// Network ports: different weights for well-known vs dynamic ranges  
Gen::frequency([
    (40, well_known_ports),  // 40% well-known ports
    (35, registered_ports),  // 35% registered ports  
    (25, dynamic_ports)      // 25% dynamic ports
])
```

**Principle**: Start with how values actually appear in production, then add exploration.

#### 4. **User Extensible by Design**
The API is designed for users to create domain-specific generators, not just consume built-in ones:

- **Pattern documentation**: Clear patterns for different use cases
- **Domain examples**: Real examples across multiple domains (financial, gaming, medical, etc.)
- **Composition guidance**: How to layer multiple dictionaries for complex scenarios

#### 5. **Proper Shrinking Behavior**
Dictionary generators maintain Hedgehog's excellent shrinking properties:

- **Dictionary-first shrinking**: When a dictionary value fails, try other dictionary values first
- **Structural shrinking**: Then fall back to underlying generator shrinking (e.g., string length)
- **Shrink validity**: All shrink candidates remain valid within the problem domain

#### 6. **Statistical Integration**
Dictionary generators work seamlessly with Hedgehog's property classification system:

```rust
let prop = for_all(Gen::<u16>::http_status_code(), |&status| {
    validate_http_response(status)
}).classify("success", |&s| s >= 200 && s <= 299)  // ← Built on classification
  .classify("client_error", |&s| s >= 400 && s <= 499);
```

This provides visibility into what your dictionary generators are actually producing.

### Why These Principles Matter

#### **For Users**
- **Predictable**: Behavior follows clear, documented patterns
- **Debuggable**: No hidden behavior, explicit weights and choices
- **Extensible**: Can create domain-specific generators following the same patterns
- **Composable**: Mix dictionary generators with any other Hedgehog features

#### **For Testing Quality**  
- **Realistic**: Tests exercise values that actually occur in production
- **Comprehensive**: Still explores unknown territory through random generation
- **Minimal counterexamples**: Proper shrinking finds the simplest failing case
- **Measurable**: Statistical validation ensures generators work as intended

#### **For Maintainability**
- **No surprises**: Clear principles guide future development
- **Consistent**: All dictionary features follow the same design patterns  
- **Focused scope**: Core API stays minimal, domain generators build on top

### Core Dictionary Methods

#### `Gen::from_elements`

Generate values exclusively from a predefined list:

```rust
use hedgehog::*;

// Test with specific user roles
let roles = vec!["admin", "user", "guest", "moderator"];
let role_gen = Gen::from_elements(roles.iter().map(|s| s.to_string()).collect()).unwrap();

let prop = for_all(role_gen, |role| {
    // All generated roles will be from the predefined list
    ["admin", "user", "guest", "moderator"].contains(&role.as_str())
});
```

#### `Gen::from_dictionary`

Mix dictionary values with random generation using weighted probabilities:

```rust
use hedgehog::*;

// Mix common HTTP status codes (80%) with random valid codes (20%)
let common_statuses = vec![200, 404, 500, 302, 401];
let status_gen = Gen::from_dictionary(
    common_statuses,
    Gen::int_range(100, 599),  // Random valid HTTP codes
    80, // 80% from dictionary
    20  // 20% random
).unwrap();

let prop = for_all(status_gen, |&status| {
    status >= 100 && status <= 599  // All should be valid HTTP codes
});
```

### Domain-Specific Generators

Hedgehog provides built-in generators for common domains:

#### Web Domain Generation

```rust
use hedgehog::*;

let domain_gen = Gen::<String>::web_domain();
// Generates: "example.com", "test.org", "api.io", etc.

let prop = for_all(domain_gen, |domain| {
    domain.contains('.') && domain.len() > 4
});
```

#### Email Address Generation

```rust
use hedgehog::*;

let email_gen = Gen::<String>::email_address();
// Generates: "user@gmail.com", "test@company.org", etc.

let prop = for_all(email_gen, |email| {
    email.contains('@') && email.contains('.')
});
```

#### HTTP Status Code Generation

```rust
use hedgehog::*;

let status_gen = Gen::<u16>::http_status_code();
// Heavily weights common codes (200, 404, 500) while including others

let prop = for_all(status_gen, |&status| {
    status >= 100 && status <= 599
}).classify("success", |&s| s >= 200 && s <= 299)
  .classify("client_error", |&s| s >= 400 && s <= 499)
  .classify("server_error", |&s| s >= 500 && s <= 599);
```

#### Network Port Generation

```rust
use hedgehog::*;

let port_gen = Gen::<u16>::network_port();
// Mixes well-known (22, 80, 443), registered, and dynamic ports

let prop = for_all(port_gen, |&port| {
    port > 0  // All ports should be valid
}).classify("well_known", |&p| p <= 1023)
  .classify("registered", |&p| p >= 1024 && p <= 49151)
  .classify("dynamic", |&p| p >= 49152);
```

#### SQL Identifier Generation

```rust
use hedgehog::*;

// Safe identifiers only
let safe_gen = Gen::<String>::sql_identifier(false);

// Mix SQL keywords (30%) with random identifiers (70%) 
let risky_gen = Gen::<String>::sql_identifier(true);

let prop = for_all(safe_gen, |identifier| {
    !identifier.is_empty() && 
    identifier.chars().all(|c| c.is_ascii_alphabetic())
});
```

#### Programming Language Tokens

```rust
use hedgehog::*;

let rust_keywords = ["fn", "let", "mut", "pub", "struct", "enum", "impl", "trait"];
let token_gen = Gen::<String>::programming_tokens(&rust_keywords);
// Generates 40% keywords, 60% random identifiers

let prop = for_all(token_gen, |token| {
    !token.is_empty() && 
    token.chars().all(|c| c.is_ascii_alphabetic())
});
```

### Real-World Use Cases

#### API Testing

```rust
use hedgehog::*;

// Test API endpoint validation
let endpoints = vec!["/users", "/posts", "/comments", "/auth"];
let endpoint_gen = Gen::from_dictionary(
    endpoints.iter().map(|s| s.to_string()).collect(),
    Gen::<String>::alpha_with_range(Range::linear(5, 20)).map(|s| format!("/{}", s)),
    70, // 70% known endpoints
    30  // 30% random endpoints
).unwrap();

let prop = for_all(endpoint_gen, |endpoint| {
    endpoint.starts_with('/') && !endpoint.is_empty()
});
```

#### Database Testing

```rust
use hedgehog::*;

// Test query builder with realistic table/column names
let table_gen = Gen::from_elements(vec![
    "users".to_string(), "orders".to_string(), "products".to_string()
]).unwrap();

let column_gen = Gen::from_elements(vec![
    "id".to_string(), "name".to_string(), "created_at".to_string()
]).unwrap();

fn build_query(table: &str, column: &str) -> String {
    format!("SELECT {} FROM {}", column, table)
}

let prop = for_all(
    Gen::<(String, String)>::tuple_of(table_gen, column_gen),
    |(table, column)| {
        let query = build_query(table, column);
        query.contains("SELECT") && query.contains("FROM")
    }
);
```

#### Network Service Testing

```rust
use hedgehog::*;

// Test service configuration with realistic ports
let service_gen = Gen::from_elements(vec![
    "web".to_string(), "api".to_string(), "database".to_string()
]).unwrap();

let port_gen = Gen::<u16>::network_port();

let prop = for_all(
    Gen::<(String, u16)>::tuple_of(service_gen, port_gen),
    |(service, &port)| {
        // Test that service configurations are reasonable
        match service.as_str() {
            "web" => port == 80 || port == 443 || port >= 8000,
            "api" => port != 22, // API shouldn't use SSH port
            "database" => port != 80 && port != 443, // DB shouldn't use web ports
            _ => true
        }
    }
).classify("web_service", |(service, _)| service == "web")
  .classify("high_port", |(_, &port)| port >= 49152);
```

### Best Practices

#### 1. Balance Dictionary vs Random

Use roughly 70-80% dictionary values and 20-30% random generation:

```rust
// Good balance for comprehensive testing
let gen = Gen::from_dictionary(
    known_values,
    random_gen,
    75, // Known edge cases
    25  // Exploration of unknown space
).unwrap();
```

#### 2. Use Domain-Specific Dictionaries

Create dictionaries that reflect your actual problem domain:

```rust
// For testing a shopping cart
let product_categories = vec![
    "electronics", "books", "clothing", "food", "toys"
];

// For testing user permissions
let permission_levels = vec![
    "read", "write", "admin", "super_admin"  
];
```

#### 3. Combine with Classification

Use dictionary generators with property classification to understand test coverage:

```rust
let prop = for_all(Gen::<String>::web_domain(), |domain| {
    validate_domain(domain)
}).classify("com_domain", |d| d.ends_with(".com"))
  .classify("org_domain", |d| d.ends_with(".org"))
  .classify("io_domain", |d| d.ends_with(".io"));
```

#### 4. Layer Multiple Dictionaries

Combine different dictionary generators for complex scenarios:

```rust
let user_gen = Gen::from_elements(vec!["alice", "bob", "charlie"]).unwrap();
let action_gen = Gen::from_elements(vec!["read", "write", "delete"]).unwrap();
let resource_gen = Gen::from_elements(vec!["file", "database", "api"]).unwrap();

let authorization_test = Gen::<(String, String, String)>::tuple_of(
    user_gen, action_gen, resource_gen
);
```

Dictionary support enables realistic testing by ensuring your properties are exercised with meaningful, domain-relevant data while still maintaining the exploratory power of random generation.

### Creating Custom Domain Generators

The dictionary API is designed for easy extension. You can create domain-specific generators for your application's needs:

#### Financial Domain
```rust
use hedgehog::*;

// Credit card types
let card_types = vec!["Visa", "Mastercard", "Amex", "Discover"];
let card_type_gen = Gen::from_elements(
    card_types.iter().map(|s| s.to_string()).collect()
).unwrap();

// Currency codes (mix common + random)
let common_currencies = vec!["USD", "EUR", "GBP", "JPY", "CAD"];
let currency_gen = Gen::from_dictionary(
    common_currencies.iter().map(|s| s.to_string()).collect(),
    Gen::<String>::alpha().map(|s| s.chars().take(3).collect()), // 3-letter codes
    85, 15
).unwrap();
```

#### Gaming Domain
```rust
use hedgehog::*;

// Game difficulty levels
let difficulties = vec!["Easy", "Normal", "Hard", "Expert", "Nightmare"];
let difficulty_gen = Gen::from_elements(
    difficulties.iter().map(|s| s.to_string()).collect()
).unwrap();

// Player actions (mix common actions with rare ones)
let common_actions = vec!["move", "jump", "attack", "defend", "use_item"];
let action_gen = Gen::from_dictionary(
    common_actions.iter().map(|s| s.to_string()).collect(),
    Gen::<String>::alpha_with_range(Range::linear(4, 12)), // Random actions
    75, 25
).unwrap();
```

#### E-commerce Domain
```rust
use hedgehog::*;

// Product categories
let categories = vec![
    "electronics", "books", "clothing", "home", "sports", 
    "beauty", "automotive", "toys", "food", "health"
];
let category_gen = Gen::from_elements(
    categories.iter().map(|s| s.to_string()).collect()
).unwrap();

// Shipping methods (realistic distribution)
let shipping = vec![("standard", 60), ("express", 25), ("overnight", 10), ("pickup", 5)];
let shipping_gen = Gen::frequency(
    shipping.into_iter().map(|(method, weight)| 
        WeightedChoice::new(weight, Gen::constant(method.to_string()))
    ).collect()
).unwrap();
```

#### Medical/Healthcare Domain
```rust
use hedgehog::*;

// Medical specialties
let specialties = vec![
    "cardiology", "neurology", "pediatrics", "oncology", 
    "dermatology", "psychiatry", "radiology", "surgery"
];
let specialty_gen = Gen::from_elements(
    specialties.iter().map(|s| s.to_string()).collect()
).unwrap();

// ICD-10 style codes (realistic prefixes + random)
let icd_prefixes = vec![
    "A01", "B02", "C03", "D04", "E05", "F06", "G07", "H08", "I09", "J10"
];
let diagnosis_gen = Gen::from_dictionary(
    icd_prefixes.iter().map(|s| s.to_string()).collect(),
    Gen::int_range(100, 999).map(|code| format!("Z{:02}", code)), // Random codes
    70, 30
).unwrap();
```

#### IoT/Device Domain  
```rust
use hedgehog::*;

// Device types
let device_types = vec!["sensor", "actuator", "gateway", "controller", "display"];
let device_gen = Gen::from_elements(
    device_types.iter().map(|s| s.to_string()).collect()
).unwrap();

// Status codes (mix normal operation with error states)
let normal_statuses = vec!["online", "idle", "active"];
let error_statuses = vec!["offline", "error", "maintenance", "low_battery"];

let status_gen = Gen::frequency(vec![
    WeightedChoice::new(70, Gen::from_elements(
        normal_statuses.iter().map(|s| s.to_string()).collect()
    ).unwrap()),
    WeightedChoice::new(30, Gen::from_elements(
        error_statuses.iter().map(|s| s.to_string()).collect()
    ).unwrap()),
]).unwrap();
```

#### Configuration/Settings Domain
```rust
use hedgehog::*;

// Log levels (realistic distribution)
let log_levels = vec![
    ("ERROR", 5), ("WARN", 15), ("INFO", 60), ("DEBUG", 15), ("TRACE", 5)
];
let log_level_gen = Gen::frequency(
    log_levels.into_iter().map(|(level, weight)|
        WeightedChoice::new(weight, Gen::constant(level.to_string()))
    ).collect()
).unwrap();

// Feature flags (mostly enabled, some disabled)  
let feature_flags = vec!["new_ui", "beta_api", "advanced_search", "mobile_app"];
let flag_gen = Gen::from_elements(feature_flags.iter().map(|s| s.to_string()).collect()).unwrap();
let flag_state_gen = Gen::frequency(vec![
    WeightedChoice::new(80, Gen::constant(true)),  // 80% enabled
    WeightedChoice::new(20, Gen::constant(false)), // 20% disabled  
]).unwrap();
```

### Extension Patterns

#### Pattern 1: Pure Dictionary Selection
Use when you have a closed set of valid values:
```rust
let valid_values = vec!["option1", "option2", "option3"];
let gen = Gen::from_elements(valid_values.iter().map(|s| s.to_string()).collect()).unwrap();
```

#### Pattern 2: Weighted Reality
Use when some values are much more common than others:
```rust
let common_values = vec!["common1", "common2"];
let gen = Gen::from_dictionary(
    common_values.iter().map(|s| s.to_string()).collect(),
    random_generator, // Your random generator
    80, 20 // 80% common, 20% random
).unwrap();
```

#### Pattern 3: Frequency-Based Distribution  
Use when you know exact probability distributions:
```rust
let weighted_choices = vec![
    ("frequent", 60), ("common", 25), ("rare", 10), ("very_rare", 5)
];
let gen = Gen::frequency(
    weighted_choices.into_iter().map(|(value, weight)|
        WeightedChoice::new(weight, Gen::constant(value.to_string()))
    ).collect()
).unwrap();
```

#### Pattern 4: Hierarchical Composition
Use for complex domains with multiple related concepts:
```rust
let user_gen = Gen::from_elements(vec!["alice", "bob", "charlie"]).unwrap();
let permission_gen = Gen::from_elements(vec!["read", "write", "admin"]).unwrap();
let resource_gen = Gen::from_elements(vec!["file", "database", "api"]).unwrap();

let authorization_test = Gen::<(String, String, String)>::tuple_of(
    user_gen, permission_gen, resource_gen
);
```

The key principle: **Start with your domain's real data distribution**. What values appear most often in production? What are the edge cases that cause bugs? Build your dictionaries to reflect this reality while still exploring unknown territory through random generation.

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