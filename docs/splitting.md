# Splittable random number generation

This document explains what splittable random number generation is and why it's crucial for Hedgehog's deterministic property testing and shrinking capabilities.

## What is splitting?

**Splitting** is a technique for creating multiple independent random streams from a single seed. Instead of advancing a single random number generator sequentially, splitting allows you to derive separate, non-overlapping streams that can be used independently.

```rust
// Sequential RNG - problematic for property testing
let mut rng = SomeRng::new(seed);
let value1 = generator1.generate(&mut rng);  // Consumes some random numbers
let value2 = generator2.generate(&mut rng);  // Consumes the NEXT numbers

// Problem: value2 depends on how many numbers generator1 consumed!
```

```rust
// Splittable RNG - correct for property testing
let seed = Seed::new(42);
let (seed1, seed2) = seed.split();  // Two independent streams

let value1 = generator1.generate(seed1);  // Uses its own independent stream
let value2 = generator2.generate(seed2);  // Uses separate independent stream

// value2 is always the same regardless of generator1's implementation
```

## Why splitting matters for property testing

### Reproducibility
With splitting, the same seed always generates exactly the same test cases, regardless of:
- How generators are implemented internally
- How many random numbers each generator consumes
- The order in which generators are composed

### Independence
Each generator gets its own "lane" of randomness. Adding, removing, or modifying one generator doesn't affect the random values produced by others.

### Deterministic shrinking
This is where splitting becomes crucial for Hedgehog's shrinking capabilities.

## Splitting and rose trees

Rose trees represent a value along with all its possible shrinks. Each shrink is itself a rose tree, creating a tree structure of alternative test values.

Without splitting, generating this tree structure is problematic:

```rust
// BROKEN: shrinks affect each other
fn generate_tree_broken(rng: &mut SomeRng) -> Tree<i32> {
    let value = rng.gen_range(0..100);
    
    let shrinks = vec![
        generate_shrink_towards_zero(rng),    // Uses some random numbers
        generate_shrink_by_half(rng),         // Uses NEXT numbers  
        generate_shrink_remove_digits(rng),   // Depends on previous shrinks!
    ];
    
    Tree::with_children(value, shrinks)
}
```

**Problem**: The third shrink depends on the random numbers consumed by the first two shrinks. If you change the implementation of the first shrink strategy, it affects all subsequent shrinks.

With splitting, each shrink gets an independent random stream:

```rust
// CORRECT: independent shrink exploration
fn generate_tree(seed: Seed) -> Tree<i32> {
    let (value_seed, shrink_seed) = seed.split();
    let value = generate_value(value_seed);
    
    // Each shrink gets its own independent seed
    let (shrink1_seed, remaining) = shrink_seed.split();
    let (shrink2_seed, shrink3_seed) = remaining.split();
    
    let shrinks = vec![
        generate_shrink_towards_zero(shrink1_seed),   // Independent
        generate_shrink_by_half(shrink2_seed),        // Independent  
        generate_shrink_remove_digits(shrink3_seed),  // Independent
    ];
    
    Tree::with_children(value, shrinks)
}
```

## Benefits for debugging

This independence provides powerful debugging guarantees:

```rust
// This ALWAYS produces the same minimal counterexample
let seed = Seed::from_u64(12345);
let tree = generator.generate(size, seed);

if property_fails(&tree.value) {
    let minimal = find_minimal_counterexample(&tree);
    // 'minimal' is deterministic and reproducible
}
```

### Stable counterexamples
The minimal failing test case found by shrinking is always the same for a given seed, regardless of:
- Implementation changes to individual shrink strategies
- Adding new shrink alternatives
- The order in which shrinks are explored

### Reproducible bug reports
When a property test fails, you can provide the exact seed that reproduces the failure. Anyone can use that seed to get the exact same test case and shrinking behavior.

### Lazy shrink exploration
Since each branch of the rose tree is independent, shrinks can be generated on-demand without affecting other branches. This enables efficient exploration of the shrink space.

## Implementation in Hedgehog

Hedgehog implements splitting in the `Seed` type with these key methods:

```rust
impl Seed {
    // Create two independent seeds from one
    pub fn split(self) -> (Self, Self);
    
    // Generate next value and advance this seed
    pub fn next_u64(self) -> (u64, Self);
    
    // Other generation methods build on next_u64
    pub fn next_bool(self) -> (bool, Self);
    pub fn next_bounded(self, bound: u64) -> (u64, Self);
}
```

The `split()` method ensures that the two returned seeds will never produce overlapping sequences, maintaining independence across all future operations.

## Contrast with standard PRNGs

Most standard pseudo-random number generators (like those in the `rand` crate) are designed for sequential use and don't naturally support splitting:

```rust
// Standard approach - fine for general use, problematic for property testing
let mut rng = SmallRng::seed_from_u64(42);
let a = rng.gen::<u32>();
let b = rng.gen::<u32>();  // Depends on 'a' being generated first
```

For property testing, we need the guarantee that generating `b` will always produce the same value regardless of whether `a` was generated or how `a` was generated.

## Historical context

Splittable random number generators were pioneered in functional programming languages, particularly in Haskell's QuickCheck and later refined in Hedgehog. The technique enables the pure functional approach to randomness that makes property testing both powerful and predictable.

Languages with mutable state often use sequential RNGs, but property testing libraries benefit enormously from the deterministic guarantees that splitting provides.