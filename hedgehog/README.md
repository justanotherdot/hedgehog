# hedgehog

The main Hedgehog crate - property-based testing with integrated shrinking for Rust.

## Overview

This is the primary user-facing crate for Hedgehog. It re-exports the core functionality and provides the main API for writing property-based tests.

## Usage

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
hedgehog = "0.1"
```

Or with derive macros:

```toml
[dev-dependencies]
hedgehog = { version = "0.1", features = ["derive"] }
```

## Quick Example

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
        TestResult::Pass { .. } => (),
        result => panic!("Property failed: {:?}", result),
    }
}
```

## Features

- `derive` - Enable derive macros for automatic generator creation

## Documentation

For comprehensive documentation, examples, and guides, see the [main repository README](https://github.com/hedgehogqa/rust-hedgehog).

## Architecture

This crate provides:
- Re-exports of `hedgehog-core` functionality
- Optional derive macro support via `hedgehog-derive`
- User-friendly prelude
- Curated corpus collections for realistic test data

## License

BSD-3-Clause
