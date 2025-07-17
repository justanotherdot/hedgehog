# Ideas for Future Hedgehog Features

This document captures ideas for extending Hedgehog's capabilities, inspired by other property testing and fuzzing tools.

## High Impact Features

### 1. Regression Corpus
**Status**: Already documented in `regression-corpus.md`
**Inspiration**: PropTest's regression files
**Benefit**: Faster regression detection, deterministic replay

### 2. Coverage-Guided Generation
**Inspiration**: Hypothesis, FuzzCheck
**What**: Use code coverage feedback to guide input generation towards unexplored code paths
**Implementation**: 
- Instrument code during compilation
- Track which branches/lines are hit by each generated input
- Bias generation towards inputs that discover new coverage
```rust
let prop = for_all(Gen::string().coverage_guided(), |s| {
    // Generator adapts based on which inputs explore new code
    parse_function(s).is_ok()
});
```

### 3. Example Integration
**Inspiration**: Hypothesis `@example` decorator
**What**: Mix explicit test cases with generated ones
```rust
#[test]
fn test_parsing() {
    let prop = for_all(Gen::string(), |s| parse(s).is_ok() || parse(s).is_err())
        .with_examples(&["", "null", "true", "false", "{}", "[]"]);
    // Always tests the examples first, then generates random inputs
}
```

### 4. Dictionary Support
**Inspiration**: AFL, libFuzzer
**What**: Domain-specific token injection for more realistic inputs
```rust
let json_dict = Dictionary::new()
    .add_tokens(&["null", "true", "false"])
    .add_tokens(&["{", "}", "[", "]", ":", ","])
    .add_strings(&["\"hello\"", "\"world\""]);

let prop = for_all(Gen::string().with_dictionary(json_dict), |s| {
    // More likely to generate JSON-like strings
    json_parser(s)
});
```

### 5. Function Generators
**Inspiration**: QuickCheck's `CoArbitrary`
**What**: Generate functions as test inputs
```rust
let prop = for_all((Gen::function(Gen::int(), Gen::bool()), Gen::int()), |(f, x)| {
    // f is a generated function from int -> bool
    let result1 = f(x);
    let result2 = f(x);
    result1 == result2 // Functions should be deterministic
});

// Use case: Testing higher-order functions
let prop = for_all(
    (Gen::vec_int(), Gen::function(Gen::int(), Gen::bool())),
    |(vec, predicate)| {
        let filtered: Vec<_> = vec.iter().filter(|&x| predicate(*x)).collect();
        filtered.len() <= vec.len()
    }
);
```

### 6. Property Classification
**Inspiration**: QuickCheck's `classify` and `collect`
**What**: See distribution of generated test data
```rust
let prop = for_all(Gen::vec_int(), |vec| {
    // Automatically track what kinds of inputs we're testing
    vec.reverse();
    vec.reverse();
    vec // Original vector
}).classify("empty", |vec| vec.is_empty())
  .classify("small", |vec| vec.len() < 10)
  .classify("large", |vec| vec.len() > 100)
  .collect("length", |vec| vec.len());

// Output shows: "80% empty, 15% small, 5% large, avg length: 3.2"
```

## Medium Impact Features

### 7. Compositional Strategies
**Inspiration**: PropTest's strategy composition
**What**: Better ways to combine and compose generators
```rust
// Current approach
let gen = Gen::tuple_of(Gen::string(), Gen::int_range(0, 100));

// Enhanced compositional approach
let user_gen = prop_compose! {
    fn user_strategy()
        (name in Gen::string().ascii_alpha())
        (age in 0..100)
        (email in format!("{}@example.com", name))
        -> User 
    {
        User { name, age, email }
    }
};
```

### 8. Custom Shrinking Strategies
**Inspiration**: PropTest's per-type shrinking
**What**: Different shrinking approaches for different types
```rust
impl Shrinkable for CustomType {
    fn shrink_strategy() -> ShrinkStrategy {
        ShrinkStrategy::new()
            .prefer_smaller_fields()
            .try_removing_optional_fields()
            .try_simplifying_nested_types()
    }
}
```

### 9. Parallel Property Testing
**Inspiration**: Quviq QuickCheck
**What**: Find race conditions and concurrency bugs
```rust
let prop = for_all_parallel(
    Gen::vec_of(Gen::int()),
    |shared_data| {
        // Run multiple threads operating on shared_data
        // Find race conditions automatically
    }
);
```

### 10. Fault Injection Testing
**Inspiration**: Quviq QuickCheck
**What**: Systematically inject failures
```rust
let prop = for_all(Gen::network_request(), |req| {
    with_fault_injection(|| {
        // Randomly inject network failures, timeouts, etc.
        make_request(req)
    })
});
```

### 11. Temporal Properties
**Inspiration**: Quviq QuickCheck
**What**: Properties over sequences of operations
```rust
let prop = temporal_property! {
    always(eventually(response_received)) &&
    never(duplicate_request) &&
    until(timeout, retries_increase)
};
```

## Lower Priority Features

### 12. Binary Instrumentation
**Inspiration**: AFL
**What**: Fuzz binaries without source code access
**Use case**: Testing closed-source libraries or C bindings

### 13. Distributed Fuzzing
**Inspiration**: AFL++
**What**: Coordinate fuzzing across multiple machines
**Use case**: Large-scale continuous fuzzing in CI/CD

### 14. Structure-Aware Mutations
**Inspiration**: FuzzCheck
**What**: Intelligent mutations that understand data structure
```rust
// Instead of random byte flipping, understand that this is JSON
// and make valid JSON mutations like adding/removing keys
```

### 15. Interactive Debugging
**What**: Step through shrinking process, examine intermediate values
```rust
// Debug mode that shows each shrinking step
prop.run_debug() // Opens interactive debugger
```

## Implementation Priority

**Phase 1** (High value, lower complexity):
1. Example integration
2. Dictionary support  
3. Property classification
4. Function generators

**Phase 2** (High value, medium complexity):
5. Coverage-guided generation
6. Compositional strategies
7. Custom shrinking strategies

**Phase 3** (Medium value, higher complexity):
8. Parallel testing
9. Fault injection
10. Temporal properties

**Future** (Specialized use cases):
11. Binary instrumentation
12. Distributed fuzzing
13. Structure-aware mutations
14. Interactive debugging

## Notes

- **Function generators** are particularly interesting as they enable testing higher-order functions and callbacks
- **Dictionary support** could be very valuable for protocol testing, parser testing, etc.
- **Coverage-guided generation** could significantly improve test effectiveness
- Many of these features could be implemented as separate crates that build on hedgehog-core

Each feature should be evaluated based on:
- **Developer demand** - What do users actually need?
- **Implementation complexity** - How much work is involved?
- **Maintenance burden** - How much ongoing work?
- **Ecosystem fit** - How well does it work with existing Rust testing tools?