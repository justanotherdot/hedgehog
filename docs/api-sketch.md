# Hedgehog Rust API Sketch

## Core Types

```rust
// Core generator type - explicit, first-class value
pub struct Gen<T> {
    generator: Box<dyn Generator<T>>,
}

// Internal trait for generator implementations
trait Generator<T> {
    fn generate(&self, size: Size, seed: Seed) -> Tree<T>;
}

// Rose tree for values and their shrinks
pub struct Tree<T> {
    pub value: T,
    children: Box<dyn Fn() -> Vec<Tree<T>>>,  // Lazy shrinks
}

// Size parameter for scaling generation
pub struct Size(usize);

// Splittable random seed
pub struct Seed { /* ... */ }

// Property test configuration
pub struct Config {
    pub test_limit: usize,
    pub shrink_limit: usize,
    pub size_limit: usize,
}
```

## Basic Generator Construction

```rust
impl<T> Gen<T> {
    // Constant generator
    pub fn constant(value: T) -> Gen<T> { /* ... */ }
    
    // Range-based generators
    pub fn range(range: Range<T>) -> Gen<T> 
    where T: Clone + PartialOrd + SampleUniform + Shrinkable { /* ... */ }
    
    // Choice from a collection
    pub fn choice<I>(items: I) -> Gen<T>
    where I: IntoIterator<Item = T>, T: Clone { /* ... */ }
    
    // Frequency-weighted choice
    pub fn frequency<I>(weighted: I) -> Gen<T>
    where I: IntoIterator<Item = (u32, Gen<T>)> { /* ... */ }
}
```

## Generator Combinators

```rust
impl<T> Gen<T> {
    // Map over generated values
    pub fn map<U>(self, f: impl Fn(T) -> U + 'static) -> Gen<U> { /* ... */ }
    
    // Bind/flatmap for dependent generation
    pub fn bind<U>(self, f: impl Fn(T) -> Gen<U> + 'static) -> Gen<U> { /* ... */ }
    
    // Filter generated values
    pub fn filter(self, predicate: impl Fn(&T) -> bool + 'static) -> Gen<T> { /* ... */ }
    
    // Size-dependent generation
    pub fn sized(f: impl Fn(Size) -> Gen<T> + 'static) -> Gen<T> { /* ... */ }
    
    // Resize generator
    pub fn resize(self, new_size: Size) -> Gen<T> { /* ... */ }
    
    // Remove shrinking
    pub fn no_shrink(self) -> Gen<T> { /* ... */ }
}
```

## Collection Generators

```rust
impl Gen<T> {
    // Generate vectors
    pub fn vec(self, length: Range<usize>) -> Gen<Vec<T>> { /* ... */ }
    
    // Generate arrays
    pub fn array<const N: usize>(self) -> Gen<[T; N]> { /* ... */ }
    
    // Generate optional values
    pub fn optional(self) -> Gen<Option<T>> { /* ... */ }
}

// Combine multiple generators
pub fn zip<T, U>(gen_t: Gen<T>, gen_u: Gen<U>) -> Gen<(T, U)> { /* ... */ }
pub fn zip3<T, U, V>(gen_t: Gen<T>, gen_u: Gen<U>, gen_v: Gen<V>) -> Gen<(T, U, V)> { /* ... */ }
```

## Primitive Generators

```rust
// Numeric generators
pub fn bool() -> Gen<bool> { /* ... */ }
pub fn u8() -> Gen<u8> { /* ... */ }
pub fn i32() -> Gen<i32> { /* ... */ }
pub fn f64() -> Gen<f64> { /* ... */ }

// String generators
pub fn ascii_char() -> Gen<char> { /* ... */ }
pub fn unicode_char() -> Gen<char> { /* ... */ }
pub fn string(length: Range<usize>) -> Gen<String> { /* ... */ }
```

## Property Testing

```rust
// Property test result
pub enum TestResult {
    Pass,
    Fail { counterexample: String, shrinks: usize },
    Discard,
}

// Property test function
pub fn check<T>(
    gen: Gen<T>,
    config: Config,
    property: impl Fn(T) -> bool,
) -> TestResult { /* ... */ }

// Property test with custom assertions
pub fn check_with<T>(
    gen: Gen<T>,
    config: Config,
    property: impl Fn(T) -> Result<(), String>,
) -> TestResult { /* ... */ }
```

## Usage Examples

### Basic Property Tests

```rust
use hedgehog::*;

// Test that reverse is involutive
let gen = Gen::range(1..=100).vec(0..=20);
let result = check(gen, Config::default(), |xs| {
    let reversed = xs.iter().rev().cloned().collect::<Vec<_>>();
    let double_reversed = reversed.iter().rev().cloned().collect::<Vec<_>>();
    xs == double_reversed
});

// Test string concatenation
let gen = zip(
    Gen::string(0..=10),
    Gen::string(0..=10),
);
let result = check(gen, Config::default(), |(s1, s2)| {
    let concatenated = format!("{}{}", s1, s2);
    concatenated.len() == s1.len() + s2.len()
});
```

### Custom Generators

```rust
// Generate a custom struct
#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: u8,
    email: String,
}

fn gen_person() -> Gen<Person> {
    zip3(
        Gen::string(1..=20),
        Gen::range(0..=120),
        Gen::string(5..=30).map(|s| format!("{}@example.com", s)),
    ).map(|(name, age, email)| Person { name, age, email })
}

// Test person validation
let result = check(gen_person(), Config::default(), |person| {
    person.age <= 120 && !person.name.is_empty() && person.email.contains('@')
});
```

### Recursive Generators

```rust
// Generate a binary tree
#[derive(Debug, Clone)]
enum Tree<T> {
    Leaf(T),
    Branch(Box<Tree<T>>, Box<Tree<T>>),
}

fn gen_tree<T>(gen_leaf: Gen<T>) -> Gen<Tree<T>> {
    Gen::sized(|size| {
        if size.0 <= 1 {
            gen_leaf.map(Tree::Leaf)
        } else {
            let subtree = gen_tree(gen_leaf.clone()).resize(Size(size.0 / 2));
            zip(subtree.clone(), subtree)
                .map(|(left, right)| Tree::Branch(Box::new(left), Box::new(right)))
        }
    })
}

// Test tree properties
let tree_gen = gen_tree(Gen::range(1..=100));
let result = check(tree_gen, Config::default(), |tree| {
    count_nodes(&tree) > 0
});
```

### Dependent Generation

```rust
// Generate vectors with their indices
let gen = Gen::range(1..=10).bind(|length| {
    zip(
        Gen::constant(length),
        Gen::range(0..length).vec(length..=length),
    )
});

let result = check(gen, Config::default(), |(length, indices)| {
    indices.len() == length && indices.iter().all(|&i| i < length)
});
```

### Integration with Test Frameworks

```rust
// With standard test framework
#[test]
fn test_sort_property() {
    let gen = Gen::range(1..=1000).vec(0..=100);
    let result = check(gen, Config::default(), |mut xs| {
        let original_len = xs.len();
        xs.sort();
        xs.len() == original_len && xs.windows(2).all(|w| w[0] <= w[1])
    });
    
    match result {
        TestResult::Pass => {},
        TestResult::Fail { counterexample, shrinks } => {
            panic!("Property failed after {} shrinks: {}", shrinks, counterexample);
        },
        TestResult::Discard => panic!("Too many discards"),
    }
}

// Macro for ergonomic property testing
macro_rules! property_test {
    ($name:ident, $gen:expr, $prop:expr) => {
        #[test]
        fn $name() {
            let result = check($gen, Config::default(), $prop);
            assert!(matches!(result, TestResult::Pass));
        }
    };
}

property_test!(reverse_involutive, 
    Gen::range(1..=100).vec(0..=20),
    |xs| {
        let rev_rev: Vec<_> = xs.iter().rev().rev().cloned().collect();
        xs == rev_rev
    }
);
```

## Configuration

```rust
impl Config {
    pub fn default() -> Config {
        Config {
            test_limit: 100,
            shrink_limit: 1000,
            size_limit: 100,
        }
    }
    
    pub fn with_tests(mut self, tests: usize) -> Config {
        self.test_limit = tests;
        self
    }
    
    pub fn with_shrinks(mut self, shrinks: usize) -> Config {
        self.shrink_limit = shrinks;
        self
    }
}
```

## Key Design Principles

1. **Explicit generators** - No type-directed magic, generators are values you create and compose
2. **Compositional** - Rich combinator library for building complex generators
3. **Integrated shrinking** - Shrinks are built into the generator, not separate
4. **Lazy evaluation** - Shrinks are computed on-demand to avoid memory issues
5. **Deterministic** - Same seed produces same results for reproducible tests
6. **Ergonomic** - Clean API that feels natural in Rust

This API maintains Hedgehog's core philosophy while feeling idiomatic in Rust.