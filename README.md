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
    let gen = Gen::range(1..=100).vec(0..=20);
    let result = check(gen, Config::default(), |xs| {
        let reversed: Vec<_> = xs.iter().rev().cloned().collect();
        let double_reversed: Vec<_> = reversed.iter().rev().cloned().collect();
        xs == double_reversed
    });
    assert!(matches!(result, TestResult::Pass));
}
```

## Core Concepts

### Explicit Generators

Unlike type-directed approaches, Hedgehog generators are explicit values you create and compose:

```rust
// Explicit generator construction
let gen_small_int = Gen::range(1..=10);
let gen_list = Gen::list(gen_small_int, 0..=5);
let gen_pair = Gen::zip(gen_small_int, gen_list);
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
let gen_person = zip3(
    Gen::string(1..=20),           // name
    Gen::range(0..=120),           // age  
    Gen::string(5..=30),           // email
).map(|(name, age, email)| Person { name, age, email });
```

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