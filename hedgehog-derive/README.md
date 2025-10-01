# hedgehog-derive

Procedural macros for automatic generator creation in Hedgehog.

## Overview

This crate provides derive macros that automatically generate `Gen<T>` instances for your custom types, eliminating boilerplate when writing property-based tests.

## Usage

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
hedgehog = { version = "0.1", features = ["derive"] }
```

Or add `hedgehog-derive` directly:

```toml
[dev-dependencies]
hedgehog-derive = "0.1"
```

## Example

```rust
use hedgehog::*;
use hedgehog_derive::Generate;

#[derive(Generate, Debug, Clone)]
struct User {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Generate, Debug, Clone)]
enum PaymentMethod {
    Cash,
    Card { number: String, expiry: String },
    Digital(String),
}

#[test]
fn prop_user_validation() {
    let prop = for_all(User::generate(), |user: &User| {
        !user.name.is_empty() && user.age <= 150
    });

    assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));
}
```

## Supported Types

The `Generate` derive macro supports:

- **Structs** (named and tuple structs)
- **Enums** (unit, tuple, and struct variants)
- Nested custom types that also implement `Generate`
- Standard library types with existing generators

## How It Works

The macro generates an associated function `generate()` that returns a `Gen<T>`. For structs, it creates a generator that produces each field independently. For enums, it randomly selects between variants with equal probability.

## Limitations

- All fields must have types that already have generators available
- No support for lifetime parameters
- Generic types are not yet supported

## License

BSD-3-Clause
