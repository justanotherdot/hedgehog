# Variable Name Tracking in Failure Reporting

Hedgehog's variable name tracking feature provides enhanced failure reporting that makes debugging property tests much easier by showing which specific input variable caused a failure during shrinking.

## Overview

When property tests fail, Hedgehog performs shrinking to find the minimal counterexample. Traditional property testing libraries show generic shrinking steps like "Original: 15, Step 1: 12, Step 2: 10" without context about what these values represent.

Hedgehog's variable name tracking enhances this by showing:
- **Named variables** in shrinking progression
- **Haskell Hedgehog-style output** with `forAll N = value -- variable_name` format
- **Clear debugging context** for complex properties with multiple inputs

## Basic Usage

### Traditional Property Testing (Anonymous Variables)

```rust
use hedgehog::*;

// Standard property testing without variable names
let prop = for_all(Gen::int_range(5, 20), |&n| n < 10);
match prop.run(&Config::default()) {
    TestResult::Fail { .. } => {
        // Output shows generic shrinking steps:
        // ━━━ module_name ━━━
        //   ✗ property failed after 1 tests and 2 shrinks.
        //
        //     Shrinking progression:
        //       │ Original: 15
        //       │ Step 1: 12
        //       │ Step 2: 10
    }
    _ => {}
}
```

### Enhanced Property Testing (Named Variables)

```rust
use hedgehog::*;

// Property testing with named variables
let prop = for_all_named(Gen::int_range(5, 20), "n", |&n| n < 10);
match prop.run(&Config::default()) {
    TestResult::Fail { .. } => {
        // Output shows meaningful variable names:
        // ━━━ module_name ━━━
        //   ✗ property failed after 1 tests and 2 shrinks.
        //
        //     Shrinking progression:
        //       │ forAll 0 = 15 -- n
        //       │ forAll 1 = 12 -- n
        //       │ forAll 2 = 10 -- n
    }
    _ => {}
}
```

## API Reference

### Module-Level Functions

#### `for_all_named`

```rust
pub fn for_all_named<T, F>(
    generator: Gen<T>, 
    variable_name: &str, 
    condition: F
) -> Property<T>
where
    T: 'static + std::fmt::Debug,
    F: Fn(&T) -> bool + 'static,
```

Creates a property that checks a boolean condition with a named variable.

**Parameters:**
- `generator`: The generator for test values
- `variable_name`: The name to display in failure reports (e.g., "n", "text", "users")
- `condition`: The property condition to test

**Returns:** A `Property<T>` that can be run with `.run()`

### Property Methods

#### `Property::for_all_named`

```rust
impl<T> Property<T> {
    pub fn for_all_named<F>(
        generator: Gen<T>, 
        variable_name: &str, 
        condition: F
    ) -> Self
    where
        T: 'static + std::fmt::Debug,
        F: Fn(&T) -> bool + 'static,
}
```

Creates a property instance with variable name tracking.

## Output Format Comparison

### Without Variable Names

```
━━━ mymodule::tests ━━━
  ✗ property failed after 1 tests and 3 shrinks.

    Shrinking progression:
      │ Original: "HelloWorld"
      │ Step 1: "HelloWorl"
      │ Step 2: "HelloWor"
      │ Step 3: "HelloWo"

    === Boolean Condition ===
    Minimal counterexample: "HelloWo"
```

### With Variable Names

```
━━━ mymodule::tests ━━━
  ✗ property failed after 1 tests and 3 shrinks.

    Shrinking progression:
      │ forAll 0 = "HelloWorld" -- text
      │ forAll 1 = "HelloWorl" -- text
      │ forAll 2 = "HelloWor" -- text
      │ forAll 3 = "HelloWo" -- text

    === Boolean Condition ===
    Minimal counterexample: "HelloWo"
```

## Practical Examples

### Example 1: Numeric Properties

```rust
use hedgehog::*;

fn test_fibonacci_property() {
    let prop = for_all_named(
        Gen::int_range(0, 20), 
        "n", 
        |&n| {
            let fib_n = fibonacci(n);
            let fib_n_plus_1 = fibonacci(n + 1);
            let fib_n_plus_2 = fibonacci(n + 2);
            
            // Fibonacci property: F(n) + F(n+1) = F(n+2)
            fib_n + fib_n_plus_1 == fib_n_plus_2
        }
    );
    
    match prop.run(&Config::default()) {
        TestResult::Pass { tests_run } => {
            println!("Fibonacci property passed {} tests", tests_run);
        }
        TestResult::Fail { .. } => {
            println!("Fibonacci property failed:");
            println!("{}", prop.run(&Config::default()));
            // Shows: forAll 0 = 5 -- n (for example)
        }
        _ => {}
    }
}
```

### Example 2: String Properties

```rust
use hedgehog::*;

fn test_string_reversal_property() {
    let prop = for_all_named(
        Gen::<String>::alpha_with_range(Range::linear(1, 50)),
        "text",
        |text| {
            let reversed = text.chars().rev().collect::<String>();
            let double_reversed = reversed.chars().rev().collect::<String>();
            
            // Reversal is involutive: reverse(reverse(x)) = x
            double_reversed == *text
        }
    );
    
    match prop.run(&Config::default()) {
        TestResult::Fail { .. } => {
            println!("String reversal property failed:");
            println!("{}", prop.run(&Config::default()));
            // Shows: forAll 0 = "AbCdEf" -- text (for example)
        }
        _ => {}
    }
}
```

### Example 3: Complex Data Structures

```rust
use hedgehog::*;

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    email: String,
}

fn user_generator() -> Gen<User> {
    Gen::new(|size, seed| {
        let (name_seed, rest) = seed.split();
        let (age_seed, email_seed) = rest.split();
        
        let name = Gen::<String>::alpha_with_range(Range::linear(2, 20))
            .generate(size, name_seed).outcome();
        let age = Gen::<u32>::from_range(Range::linear(18, 80))
            .generate(size, age_seed).outcome();
        let email = Gen::<String>::alpha_with_range(Range::linear(5, 30))
            .generate(size, email_seed).outcome();
        
        Tree::singleton(User { name, age, email })
    })
}

fn test_user_validation_property() {
    let prop = for_all_named(
        user_generator(),
        "user",
        |user| {
            // User validation rules
            !user.name.is_empty() && 
            user.age >= 18 && 
            user.email.contains('@')  // This will fail!
        }
    );
    
    match prop.run(&Config::default()) {
        TestResult::Fail { .. } => {
            println!("User validation property failed:");
            println!("{}", prop.run(&Config::default()));
            // Shows: forAll 0 = User { name: "abc", age: 25, email: "noatsign" } -- user
        }
        _ => {}
    }
}
```

## Multiple Input Variables

When testing properties with multiple inputs, use descriptive variable names:

```rust
use hedgehog::*;

fn test_addition_property() {
    // For multiple inputs, you can combine generators
    let combined_gen = Gen::new(|size, seed| {
        let (x_seed, y_seed) = seed.split();
        let x = Gen::int_range(0, 100).generate(size, x_seed).outcome();
        let y = Gen::int_range(0, 100).generate(size, y_seed).outcome();
        Tree::singleton((x, y))
    });
    
    let prop = for_all_named(
        combined_gen,
        "(x, y)",
        |(x, y)| {
            // Commutativity: x + y = y + x
            x + y == y + x
        }
    );
    
    // Future enhancement: support for multiple named variables
    // let prop = for_all_named2(
    //     Gen::int_range(0, 100), "x",
    //     Gen::int_range(0, 100), "y",
    //     |x, y| x + y == y + x
    // );
}
```

## Best Practices

### 1. Use Descriptive Variable Names

```rust
// Good: Descriptive names that explain the domain
let prop = for_all_named(age_gen, "age", |&age| age >= 0 && age <= 150);
let prop = for_all_named(username_gen, "username", |name| is_valid_username(name));
let prop = for_all_named(request_gen, "request", |req| validate_request(req));

// Avoid: Generic names that don't add meaning
let prop = for_all_named(int_gen, "x", |&x| x > 0);
let prop = for_all_named(string_gen, "s", |s| !s.is_empty());
```

### 2. Match Domain Terminology

```rust
// Use terminology that matches your domain
let prop = for_all_named(account_gen, "account", |acc| acc.balance >= 0);
let prop = for_all_named(transaction_gen, "transaction", |tx| tx.amount > 0);
let prop = for_all_named(user_gen, "user", |user| user.is_valid());
```

### 3. Use Singular Names for Single Values

```rust
// Good: Singular names for single values
let prop = for_all_named(item_gen, "item", |item| item.is_valid());

// Avoid: Plural names for single values
let prop = for_all_named(item_gen, "items", |item| item.is_valid());
```

### 4. Consider Complex Properties

```rust
// For complex properties, use descriptive names that help debugging
let prop = for_all_named(
    database_state_gen,
    "initial_state",
    |state| {
        let result = apply_transaction(state, &transaction);
        result.is_consistent()
    }
);
```

## Migration Guide

### From `for_all` to `for_all_named`

```rust
// Before: Anonymous variables
let prop = for_all(Gen::int_range(1, 100), |&n| n > 0);

// After: Named variables
let prop = for_all_named(Gen::int_range(1, 100), "n", |&n| n > 0);
```

### Gradual Adoption

You can adopt variable name tracking gradually:

```rust
// Keep existing tests unchanged
let old_prop = for_all(gen, |input| test_condition(input));

// Add names to new tests
let new_prop = for_all_named(gen, "input", |input| test_condition(input));

// Gradually migrate important tests
let important_prop = for_all_named(gen, "critical_input", |input| test_condition(input));
```

## Implementation Details

### Data Structure

The variable name is stored in the `ShrinkStep` structure:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShrinkStep {
    pub counterexample: String,
    pub step: usize,
    pub variable_name: Option<String>,  // The variable name
}
```

### Property Structure

Properties track variable names internally:

```rust
pub struct Property<T> {
    generator: Gen<T>,
    test_function: Box<dyn Fn(&T) -> TestResult>,
    variable_name: Option<String>,  // The variable name
}
```

### Shrinking Integration

Variable names are automatically passed through the shrinking process:

```rust
// During shrinking, variable names are preserved
shrink_steps.push(ShrinkStep {
    counterexample: format!("{:?}", shrink_value),
    step: shrink_count,
    variable_name: self.variable_name.clone(),  // Preserved from property
});
```

## Performance Considerations

### Memory Overhead

- Each `ShrinkStep` stores an optional `String` for the variable name
- Typical overhead: 24 bytes per shrink step (8 bytes for `Option<String>` + 16 bytes for `String` if present)
- For most properties, this represents < 1KB of additional memory

### Runtime Overhead

- Variable name tracking has minimal runtime impact
- String cloning during shrinking is negligible compared to property evaluation
- No impact on generation speed, only on failure reporting

### Recommendation

Use variable name tracking liberally - the debugging benefits far outweigh the minimal overhead.

## Comparison with Other Libraries

### QuickCheck (Rust)

```rust
// QuickCheck: No variable name tracking
quickcheck! {
    fn prop(n: i32) -> bool {
        n + 0 == n
    }
}
// Failure output: "Assertion failed: n + 0 == n"
```

### Haskell Hedgehog

```haskell
-- Haskell Hedgehog: Automatic variable tracking
prop_reverse :: Property
prop_reverse = property $ do
  xs <- forAll $ Gen.list (Range.linear 0 100) Gen.alpha
  reverse (reverse xs) === xs

-- Failure output:
-- ✗ prop_reverse failed at size 2 after 1 test and 2 shrinks.
--   ┏━━ test/Test.hs ━━━
--   20 ┃ prop_reverse = property $ do
--   21 ┃   xs <- forAll $ Gen.list (Range.linear 0 100) Gen.alpha
--   22 ┃   reverse (reverse xs) === xs
--      ┃   │ forAll 0 = "ab"
--      ┃   │ forAll 1 = "a"
--      ┃   │ forAll 2 = ""
```

### Rust Hedgehog

```rust
// Rust Hedgehog: Explicit variable tracking
let prop = for_all_named(
    Gen::<Vec<char>>::vec_of(Gen::<char>::ascii_alpha()),
    "xs",
    |xs| {
        let reversed: Vec<char> = xs.iter().rev().cloned().collect();
        let double_reversed: Vec<char> = reversed.iter().rev().cloned().collect();
        double_reversed == *xs
    }
);

// Failure output:
// ✗ property failed after 1 tests and 2 shrinks.
//   Shrinking progression:
//     │ forAll 0 = ['a', 'b'] -- xs
//     │ forAll 1 = ['a'] -- xs
//     │ forAll 2 = [] -- xs
```

## Troubleshooting

### Variable Names Not Showing

**Problem:** Using `for_all` instead of `for_all_named`:
```rust
// This won't show variable names
let prop = for_all(gen, |input| test_condition(input));
```

**Solution:** Use `for_all_named`:
```rust
let prop = for_all_named(gen, "input", |input| test_condition(input));
```

### Empty Shrink Steps

**Problem:** Properties that don't shrink won't show variable names:
```rust
let prop = for_all_named(Gen::constant(42), "n", |&n| n == 42);
// This always passes, no shrinking occurs
```

**Solution:** Test with generators that can produce failing values:
```rust
let prop = for_all_named(Gen::int_range(0, 100), "n", |&n| n < 50);
// This can fail and will show shrinking progression
```

### Confusing Variable Names

**Problem:** Non-descriptive variable names:
```rust
let prop = for_all_named(complex_gen, "x", |x| complex_condition(x));
```

**Solution:** Use descriptive names:
```rust
let prop = for_all_named(user_gen, "user", |user| user.is_valid());
```

## Future Enhancements

### Multiple Named Variables

Support for properties with multiple named inputs:

```rust
// Future API (not yet implemented)
let prop = for_all_named2(
    Gen::int_range(0, 100), "x",
    Gen::int_range(0, 100), "y",
    |x, y| x + y == y + x
);

// Would show:
// │ forAll 0 = 15 -- x
// │ forAll 0 = 23 -- y
```

### Source Location Integration

Automatic variable name detection from source code:

```rust
// Future API (not yet implemented)
let prop = for_all_auto(Gen::int_range(0, 100), |n| n > 0);
// Would automatically detect "n" from the closure parameter
```

### IDE Integration

Enhanced IDE support for variable name tracking:
- Hover information showing variable names
- Jump to definition for failing variables
- Inline failure information

This variable name tracking system provides a significant improvement to the debugging experience, making it much easier to understand which specific inputs caused property failures during shrinking.