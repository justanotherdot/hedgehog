# Hedgehog Rust Port Design Document

## Overview

This document captures the analysis and strategic approach for porting the Hedgehog property-based testing library from Haskell to Rust. Based on analysis of the original Haskell implementation and a previous failed Rust port attempt, this document outlines the key challenges, design decisions, and implementation strategy.

## Hedgehog's Core Value Proposition

Hedgehog's fundamental innovation over QuickCheck is **explicit, compositional generators**:

- **Explicit generators** - you choose exactly how to generate data, not type-directed magic
- **Integrated shrinking** - shrinks are built into the generator, not separate type class methods
- **Compositional** - you build complex generators from simple ones using combinators
- **No magic** - no hidden type class instances determining behavior via type inference

This allows precise control over generation:
```rust
let gen_small_int = Gen::range(1..=10);           // Explicit choice
let gen_list = Gen::list(gen_small_int, 0..=5);   // Composed from smaller generators
let gen_pair = Gen::zip(gen_small_int, gen_list);  // Further composition
```

**Not** type-directed generation a la QuickCheck:
```rust
let value: (i32, Vec<i32>) = Gen::generate(); // Type determines generator - a la QuickCheck!
```

**Critical insight**: If we end up with type-directed shrinking, we've completely missed the point of Hedgehog and might as well use QuickCheck. The port must preserve explicit, compositional generators as first-class values.

## Analysis Summary

### Original Haskell Hedgehog Architecture

#### Core Abstractions

1. **Property** - Represents a property test with configuration and test logic
2. **Generator (Gen/GenT)** - Monadic generator for random test data with integrated shrinking
3. **Tree (Tree/TreeT)** - Rose tree structure for representing values with shrinks
4. **Property Test (PropertyT)** - Monad transformer for property test execution
5. **Range** - Describes bounds for number generation with size scaling

#### Key Design Patterns

1. **Integrated Shrinking** - Values generated with built-in shrink trees
2. **Size-Driven Generation** - Recursive structures scale with size parameter (0-99)
3. **Monad Transformer Stack** - `PropertyT` over `TestT` over `GenT`
4. **Lazy Rose Trees** - Efficient on-demand shrinking via lazy evaluation
5. **Splittable Random** - Deterministic, reproducible generation using splittable seeds

#### Architecture Layers

- **Public API Layer** - Main user-facing modules (`Hedgehog.hs`, `Hedgehog/Gen.hs`)
- **Core Implementation** - Internal modules (`Property.hs`, `Gen.hs`, `Tree.hs`, `Runner.hs`)
- **Supporting Modules** - Utilities (`Seed.hs`, `Range.hs`, `Report.hs`, `Shrink.hs`)

### Previous Rust Port Analysis

#### What Was Implemented

- Core data structures (Tree, Gen, Range, Seed, Property)
- Functional programming patterns using `Rc<dyn Fn>` extensively
- Lazy evaluation with custom `Lazy<RefCell<T>>` type
- Basic generator combinators and shrinking strategies

#### Fatal Flaws

1. **Performance Issues**
   - Inefficient lazy rose trees using `Rc<RefCell<T>>`
   - Excessive cloning due to ownership constraints
   - Runtime overhead from interior mutability

2. **Ergonomic Issues**
   - Verbose function signatures with complex lifetime bounds
   - Repetitive boilerplate code patterns
   - Type inference problems

3. **Architectural Misalignment**
   - Direct translation of Haskell functional patterns
   - Fighting Rust's ownership model
   - Lazy evaluation inefficiency in eager language

#### Key Lesson

**Direct translation of Haskell patterns to Rust is fundamentally flawed and leads to poor performance and ergonomics.**

## Rust-Specific Challenges and Opportunities

### Major Challenges

1. **Lazy Evaluation** - Rust's eager evaluation makes lazy rose trees inefficient
2. **Higher-Kinded Types** - No direct equivalent to Haskell's type system
3. **Memory Management** - `Rc<RefCell<T>>` creates overhead and complexity
4. **Monadic Composition** - Awkward without do-notation and higher-kinded types

### Opportunities

1. **Performance** - Systems language allows fine-tuned optimization
2. **Safety** - Rust's type system prevents many runtime errors
3. **Ergonomics** - Procedural macros can create excellent DSLs
4. **Ecosystem** - Rich crate ecosystem for testing and utilities

## Strategic Implementation Approach

### 1. Abandon Direct Translation

The previous Rust port failed because it tried to directly translate Haskell's functional patterns. Instead, we must design from scratch using Rust idioms while preserving Hedgehog's essential concepts.

### 2. Eager Data Structures

Replace lazy rose trees with eager alternatives:
- Pre-compute shrink sequences using `Vec<T>`
- Use iterators for lazy evaluation where needed
- Implement custom iterator types for shrinking

### 3. State Machine Generators

Instead of monadic generators, use state machines:

```rust
pub struct Gen<T> {
    state: Box<dyn GeneratorState<T>>,
}

trait GeneratorState<T> {
    fn generate(&mut self, size: usize, rng: &mut Rng) -> (T, Vec<T>);
}
```

### 4. Procedural Macros for Ergonomics

Create ergonomic property testing DSL:

```rust
#[quickcheck]
fn prop_reverse(xs: Vec<i32>) -> bool {
    let ys = xs.iter().cloned().rev().collect::<Vec<_>>();
    ys.iter().cloned().rev().collect::<Vec<_>>() == xs
}
```

### 5. Trait-Based Design

Use Rust's trait system instead of higher-order functions:

```rust
trait Generator<T> {
    fn generate(&self, size: usize, rng: &mut Rng) -> GeneratedValue<T>;
}

struct GeneratedValue<T> {
    value: T,
    shrinks: Vec<T>,
}
```

## Implementation Strategy

### Phase 1: Core Infrastructure
- Splittable random number generator
- Size-based scaling system
- Basic range types
- Simple generator trait

### Phase 2: Generator Combinators
- Primitive generators (integers, booleans, strings)
- Combinator functions (map, filter, bind)
- Collection generators (Vec, HashMap, etc.)

### Phase 3: Shrinking System
- Eager shrink sequence generation
- Shrinking strategies for primitive types
- Automatic shrinking for composite types

### Phase 4: Property Testing Framework
- Property definition and execution
- Test runner with configurable limits
- Rich failure reporting

### Phase 5: Advanced Features
- State machine testing
- Coverage tracking
- Integration with existing test frameworks

## Tradeoffs and Compromises

### Tractable Features
- Core generator combinators
- Basic shrinking strategies
- Property testing framework
- Deterministic random generation

### Challenging but Possible
- Complex shrinking (requires careful design)
- State machine testing (needs different approach)
- Coverage tracking (performance implications)

### Potentially Intractable
- Exact Haskell API compatibility
- Zero-cost lazy evaluation
- Full monadic composition

### Recommended Compromises
- Use eager evaluation with iterators
- Implement generators as traits, not closures
- Pre-compute shrinks instead of lazy generation
- Use procedural macros for ergonomic APIs
- Accept some performance overhead for better ergonomics

## Success Criteria

A successful Rust port should:

1. **Preserve Core Concepts** - Integrated shrinking, size-driven generation
2. **Achieve Good Performance** - Competitive with existing Rust testing libraries
3. **Provide Excellent Ergonomics** - Easy to use with minimal boilerplate
4. **Maintain Type Safety** - Leverage Rust's type system for correctness
5. **Integrate Well** - Work with existing Rust testing ecosystem

## The Lazy Evaluation Challenge

The fundamental challenge in porting Hedgehog is that **Haskell's lazy evaluation is the secret sauce** that makes rose tree shrinking so elegant and efficient. Without lazy evaluation, we risk expanding the entire search space at once, making shrinking prohibitively expensive.

### Why Lazy Evaluation Matters

In Haskell, rose trees work beautifully because:
- **Children are thunks**: Only computed when accessed during shrinking
- **Infinite search spaces**: Can represent infinite shrink sequences without memory issues
- **Automatic memoization**: Haskell's runtime handles caching of evaluated expressions
- **No allocation overhead**: Lazy evaluation is a language-level feature with minimal cost

## Insights from Other Language Ports

### F# Hedgehog Port Analysis

The F# port demonstrates the most successful approach to lazy evaluation outside of Haskell:

#### Core Architecture
- **Struct-based generators**: Uses `[<Struct>] type Gen<'a> = Gen of Random<Tree<'a>>` for efficient value types
- **Computation expressions**: Leverages F#'s computation expressions (`gen { }` and `property { }`) for monadic composition
- **Native lazy sequences**: Uses `type Tree<'a> = Node of 'a * seq<Tree<'a>>` with F#'s built-in lazy sequences

#### Lazy Evaluation Strategy
- **F# sequences (`seq<>`)**: Provide native lazy evaluation - children are only evaluated when accessed
- **Seamless integration**: Works transparently with F#'s computation expressions
- **Optimal performance**: Leverages .NET's optimized lazy evaluation infrastructure
- **Memory efficient**: Only materializes nodes as needed

```fsharp
// Children are lazy sequences
let shrinks (Node (_, xs) : Tree<'a>) : seq<Tree<'a>> = xs

// Lazy evaluation via F#'s native mechanisms  
let expand (f : 'a -> seq<'a>) (Node (x, xs) : Tree<'a>) : Tree<'a> =
    let ys = Seq.map (expand f) xs  // Lazy mapping
    let zs = unfoldForest id f x    // Lazy unfolding
    Node (x, Seq.append ys zs)      // Lazy concatenation
```

#### Key Design Patterns
1. **Unified random/tree system**: `Random<Tree<'a>>` combines random generation with shrinking
2. **Explicit tree operations**: `Tree.map`, `Tree.bind`, `Tree.apply` for rose tree manipulation
3. **Lazy evaluation via F#**: Uses F#'s native lazy evaluation for efficient tree operations
4. **Clean separation**: Distinct modules for `Gen`, `Property`, `Tree`, `Random`, `Shrink`

#### Successful Adaptations
- **Computation expressions** provide elegant monadic syntax similar to Haskell's do-notation
- **Struct generators** avoid allocation overhead while maintaining composability
- **Integrated shrinking** preserved through tree-based generators
- **C# interop** via LINQ namespace demonstrates cross-language compatibility

### R Hedgehog Port Analysis

The R port shows a radical but effective approach to lazy evaluation using closures:

#### Core Architecture
- **Closure-based generators**: `gen(function(size) {...})` wraps size-dependent tree creation
- **Closure-based lazy evaluation**: `children = function() { ... }` for on-demand tree expansion
- **Explicit tree sequencing**: `tree.sequence()` for combining independent generators
- **Impure generation**: `gen.impure()` for integrating with R's random functions

#### Lazy Evaluation Strategy
- **Functions as lazy values**: Children are wrapped in functions that are only called when needed
- **Memoization**: Uses `eval.children <<- force(children_)` to cache results and avoid re-computation
- **Dynamic evaluation**: R's dynamic nature allows flexible lazy evaluation patterns
- **Closure overhead**: Some performance cost but manageable in R's context

```r
tree <- function(root, children_ = list()) {
  eval.children <- NULL
  structure(
    list(
      root = root,
      children = function() {
        if (is.null(eval.children))
          eval.children <<- force(children_)  # Memoized evaluation
        eval.children
      }
    ), class = "hedgehog.internal.tree")
}
```

#### Key Design Patterns
1. **Functional composition**: `gen.and_then()`, `gen.map()` for generator combinators
2. **Explicit shrinking**: `gen.shrink(shrinker, generator)` separates shrinking logic
3. **Size-driven generation**: `gen.sized()` for parameterized generators
4. **Monadic loops**: `generate(for (x in gen) {...})` syntax sugar

#### Successful Adaptations
- **Dynamic typing** allows flexible generator composition
- **Closure-based laziness** works well with R's evaluation model
- **Explicit shrinking** makes the shrinking process more transparent
- **Syntax sugar** (`generate()` with for loops) improves ergonomics

### Common Patterns Across Ports

#### Successful Strategies
1. **Preserve core concepts**: All ports maintain integrated shrinking and size-driven generation
2. **Adapt to language strengths**: F# uses computation expressions, R uses closures
3. **Simplify complex types**: Both avoid complex type machinery while preserving functionality
4. **Explicit tree handling**: Make rose tree operations explicit rather than implicit

#### Adaptation Lessons
1. **Embrace native patterns**: Don't force Haskell's monadic style if the language has better alternatives
2. **Explicit over implicit**: Make shrinking and tree operations explicit for better debugging
3. **Language-specific ergonomics**: Use the target language's idioms for better user experience
4. **Modular design**: Separate concerns (generation, shrinking, testing) into distinct modules

### Failed Rust Port Analysis

The previous Rust port demonstrates how **not** to handle lazy evaluation:

#### Attempted Lazy Evaluation Strategy
- **Custom lazy types**: `Lazy<RefCell<T>>` for manual lazy evaluation
- **Shared mutable state**: `Rc<RefCell<T>>` for interior mutability
- **Manual thunks**: Explicit lazy evaluation with closures

#### Fatal Flaws
- **Performance disaster**: `Rc<RefCell<T>>` creates significant overhead
- **Memory bloat**: Reference counting and interior mutability increase memory usage
- **Runtime costs**: Dynamic borrow checking adds performance penalties
- **Allocation pressure**: Excessive cloning due to ownership constraints
- **Architectural misalignment**: Fighting Rust's ownership model instead of embracing it

```rust
// This approach failed - don't do this!
pub struct Tree<'a, A> {
    thunk: Lazy<'a, A>,           // Custom lazy type
    pub children: Vec<Tree<'a, A>>, // Eager children - defeats the purpose!
}
```

#### Key Lesson
The Rust port failed because it tried to **force lazy evaluation patterns** that don't align with Rust's ownership model. Instead of embracing Rust's strengths (zero-cost abstractions, efficient iterators), it created expensive workarounds.

## Lazy Evaluation Alternatives for Rust

Since Rust doesn't have built-in lazy evaluation, we need alternative strategies:

### 1. Iterator-Based Approach
```rust
struct Tree<T> {
    value: T,
    children: Box<dyn Iterator<Item = Tree<T>>>,
}
```
**Pros**: Leverages Rust's efficient iterator infrastructure
**Cons**: Lifetime and trait object complexity

### 2. Eager Pre-computed Shrinks
```rust
struct Tree<T> {
    value: T,
    shrinks: Vec<T>,  // Pre-computed, not lazy
}
```
**Pros**: Simple, no lazy evaluation needed
**Cons**: May compute unnecessary shrinks

### 3. Generator Functions
```rust
struct Tree<T> {
    value: T,
    children: Box<dyn FnOnce() -> Vec<Tree<T>>>,
}
```
**Pros**: Defers computation until needed
**Cons**: Single-use, ownership complexity

### 4. Streaming/Pull-based
```rust
trait ShrinkStream<T> {
    fn next_shrink(&mut self) -> Option<T>;
}
```
**Pros**: Minimal memory usage
**Cons**: Stateful, more complex API

### Implications for Rust Port

#### Validated Approaches from Other Ports
- **F# native sequences**: Shows that language-level lazy evaluation is ideal
- **R closure-based**: Demonstrates that explicit lazy evaluation can work
- **Haskell thunks**: The gold standard for lazy rose trees
- **Rust failure**: Proves that manual lazy evaluation is a dead end

#### Rust-Specific Opportunities
- **Iterator-based shrinking**: Use Rust's iterator trait for lazy shrink sequences
- **Trait-based design**: Define generator behavior via traits rather than closures
- **Zero-cost abstractions**: Leverage Rust's compile-time optimization
- **Memory safety**: Avoid the allocation overhead seen in failed attempts

#### Design Recommendations
1. **Embrace eager evaluation**: Use `Vec<T>` with smart pre-computation
2. **Iterator shrinking**: `impl Iterator<Item = T>` for shrink sequences
3. **Streaming API**: Pull-based shrinking to minimize memory usage
4. **Trait-based generators**: `trait Generator<T>` instead of closure-based approach
5. **Macro-based DSL**: Use procedural macros for ergonomic property syntax

## Conclusion

The key insight is that successful Hedgehog ports don't directly translate Haskell's patterns but instead adapt the core concepts to their target language's strengths. The F# port shows how to maintain elegance with computation expressions, while the R port demonstrates how explicit design can improve usability. 

For Rust, this means embracing traits, iterators, and zero-cost abstractions rather than forcing functional programming patterns. The goal is to create a library that feels native to Rust while preserving Hedgehog's essential innovations in integrated shrinking and size-driven generation.