# Derive Macros

The `#[derive(Generate)]` macro automatically creates generators for your custom types.

## Setup

```toml
[dependencies]
hedgehog = { version = "0.1.0", features = ["derive"] }
```

## Basic Usage

```rust
use hedgehog::*;
use hedgehog_derive::Generate;

#[derive(Generate, Debug, Clone)]
struct User {
    name: String,
    age: u32,
    active: bool,
}

// Creates: User::generate() -> Gen<User>
let user = User::generate().generate(Size::new(10), Seed::random()).outcome();
```

## Supported Types

### Structs
```rust
#[derive(Generate, Debug, Clone)]
struct Point(i32, i32);                    // Tuple struct

#[derive(Generate, Debug, Clone)]
struct Unit;                               // Unit struct

#[derive(Generate, Debug, Clone)]
struct Person { name: String, age: u32 }   // Named fields
```

### Enums
```rust
#[derive(Generate, Debug, Clone)]
enum Status {
    Active,                                 // Unit variant
    Pending(String),                        // Tuple variant
    Error { code: u32, message: String },  // Named variant
}
```

## Built-in Types

| Type | Generated Range |
|------|-----------------|
| `String` | Alphabetic strings |
| `i32`, `u32`, `i64` | 0 to 100 |
| `u8`, `i8`, `u16`, `i16` | Type-appropriate ranges |
| `f32`, `f64` | 0.0 to 100.0 |
| `bool` | true/false |
| `char` | a-z, A-Z |

## Property Testing

```rust
#[derive(Generate, Debug, Clone)]
struct Rectangle {
    width: u32,
    height: u32,
}

#[test]
fn test_rectangle_area() {
    let prop = property::for_all(Rectangle::generate(), |rect: &Rectangle| {
        rect.width * rect.height >= rect.width
    });
    
    assert!(matches!(prop.run(&Config::default()), TestResult::Pass { .. }));
}
```

## Custom Types

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

## Limitations

- No attribute customization (yet)
- Vec/HashMap need manual implementation
- No generic type support (yet)

That's it! The derive macro handles the rest automatically.