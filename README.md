# Hedgehog for Rust

> Release with confidence.

Property-based testing library for Rust, inspired by the original [Hedgehog](https://hedgehog.qa/) library for Haskell.

## Features

- **Explicit generators** - No type-directed magic, generators are first-class values you compose
- **Integrated shrinking** - Shrinks obey invariants by construction, built into generators
- **Compositional** - Rich combinator library for building complex generators from simple ones
- **Excellent debugging** - Minimal counterexamples with rich failure reporting
- **Deterministic** - Reproducible test runs with seed-based generation

## Quick Start

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
hedgehog = "0.1"
```

Write a property test:

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
        TestResult::Pass => (), // Test passed
        result => panic!("Property failed: {:?}", result),
    }
}
```

For larger codebases, you may prefer qualified imports:

```rust
use hedgehog::{for_all, Config, TestResult};
use hedgehog::Gen;

#[test] 
fn prop_reverse_qualified() {
    let gen = Gen::<Vec<i32>>::vec_of(Gen::int_range(1, 100));
    let prop = for_all(gen, |xs: &Vec<i32>| {
        let reversed: Vec<_> = xs.iter().rev().cloned().collect();
        let double_reversed: Vec<_> = reversed.iter().rev().cloned().collect();
        *xs == double_reversed
    });
    // ... rest of test
}
```

## Core Concepts

### Explicit Generators

Unlike type-directed approaches, Hedgehog generators are explicit values you create and compose:

```rust
// Explicit generator construction
let gen_small_int = Gen::int_range(1, 10);
let gen_list = Gen::<Vec<i32>>::vec_of(gen_small_int);
let gen_pair = Gen::<(i32, String)>::tuple_of(gen_small_int, Gen::<String>::ascii_alpha());
```

### Integrated Shrinking

Shrinking is built into the generator, not separate. When a test fails, Hedgehog automatically finds the minimal counterexample:

```
*** Failed! Falsifiable (after 13 tests and 5 shrinks):
[1, 2]
```

### Compositional Design

Build complex generators from simple ones using combinators:

```rust
// Build complex generators using map and tuple_of
let gen_user_data = Gen::<(String, i32)>::tuple_of(
    Gen::<String>::ascii_alpha(),     // username
    Gen::int_range(18, 120)           // age
);

let gen_user = gen_user_data.map(|(username, age)| User { username, age });
```

## Available Generators

Hedgehog provides comprehensive generators with enhanced shrinking strategies:

### Primitive Types
- **Integers**: `Gen::int_range(min, max)` with origin-based shrinking
- **Booleans**: `Gen::bool()` 
- **Characters**: `Gen::<char>::ascii_alpha()`, `ascii_alphanumeric()`, `ascii_printable()`

### Strings
- **String generators**: `Gen::<String>::ascii_alpha()`, `ascii_alphanumeric()`, `ascii_printable()`
- **Custom strings**: `Gen::<String>::string_of(char_gen)`
- **Enhanced shrinking**: Character removal, simplification (uppercase→lowercase), substring removal

### Collections
- **Vectors**: `Gen::<Vec<T>>::vec_of(element_gen)`, `vec_int()`, `vec_bool()`
- **Options**: `Gen::<Option<T>>::option_of(inner_gen)`
- **Tuples**: `Gen::<(T, U)>::tuple_of(first_gen, second_gen)`
- **Results**: `Gen::<Result<T, E>>::result_of(ok_gen, err_gen)`, `result_of_weighted()`

### Generator Combinators
- **Map**: Transform generated values with `.map(f)`
- **Bind**: Dependent generation with `.bind(f)` 
- **Filter**: Conditional generation with `.filter(predicate)`

## Enhanced Shrinking

All generators include sophisticated shrinking strategies:
- **Integers**: Binary search towards meaningful origins (0, min, or max)
- **Strings**: Character simplification and smart removal patterns
- **Collections**: Element removal, element-wise shrinking, and empty container prioritization
- **Containers**: Type-specific shrinking (Option→None, Result→Ok bias)

## In Memory of Jacob Stanley

This library is inspired by the original Hedgehog library for Haskell, created by Jacob Stanley and the Hedgehog team. Jacob was a remarkable mentor who had a profound influence on many in the functional programming community, including the author of this Rust port.

Jacob's vision of property-based testing with integrated shrinking revolutionized how we think about testing. His approach of making shrinking a first-class concern, built into the generator rather than bolted on afterwards, makes finding minimal counterexamples both automatic and reliable.

His foundational talk "Gens N' Roses: Appetite for Reduction" at YOW! Lambda Jam 2017 explained the core insights behind integrated shrinking and continues to influence property-based testing libraries across many languages.

Jacob passed away unexpectedly on April 9th, 2021. His absence is deeply felt, but his impact on property-based testing and the broader programming community remains. This Rust port aims to honor his memory by bringing his innovative approach to a new language and community.

**RIP, Jake.** Your mentorship and ideas live on.

## Project Status

This is a work-in-progress implementation. See [docs/roadmap.md](docs/roadmap.md) for the development plan.

## Contributing

Contributions are welcome! Please see the [roadmap](docs/roadmap.md) for planned features and current progress.

## License

This project is licensed under the BSD-3-Clause License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Jacob Stanley and the original Hedgehog team for the foundational ideas
- The Haskell, F#, and R Hedgehog ports for implementation insights
- The Rust community for excellent tooling and ecosystem support