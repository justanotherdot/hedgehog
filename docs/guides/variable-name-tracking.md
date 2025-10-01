# Variable Name Tracking

Enhanced failure reporting that shows variable names in counterexamples.

## Problem

Default property testing shows anonymous variables:
```
forAll 0 = 42
```

## Solution

Named variables provide context:
```
forAll 0 = 42 -- number
```

## Usage

### Basic Named Variables

```rust
use hedgehog::*;

// Instead of:
let prop = for_all(Gen::int_range(1, 100), |&n| n > 0);

// Use:
let prop = for_all_named(Gen::int_range(1, 100), "number", |&n| n > 0);
```

### Multiple Variables

```rust
let prop = for_all(
    (Gen::int_range(1, 100), Gen::int_range(1, 100)),
    |(a, b)| a + b > *a.max(b)
);

// Better:
let prop = for_all_named(
    (Gen::int_range(1, 100), Gen::int_range(1, 100)),
    ("left", "right"),
    |(a, b)| a + b > *a.max(b)
);
```

### Property Methods

```rust
let prop = Property::for_all_named(
    Gen::<String>::ascii_alpha(),
    "text",
    |text| !text.is_empty()
);
```

## Output Comparison

**Without names:**
```
forAll 0 = ""
```

**With names:**
```
forAll 0 = "" -- text
```

## Best Practices

1. **Use descriptive names**: `"email"` not `"s"`
2. **Match domain terms**: `"account_balance"` not `"number"`
3. **Keep names short**: `"user"` not `"user_account_object"`

That's it! Variable names make debugging much easier.