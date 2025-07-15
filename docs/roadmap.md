# Hedgehog Rust Implementation Roadmap

## Project Vision

Build a Rust port of Hedgehog that preserves its core value proposition:
- **Explicit generators** - No type-directed magic, generators are first-class values
- **Integrated shrinking** - Shrinks built into generators, not separate
- **Compositional** - Rich combinator library for building complex generators
- **Excellent debugging** - First-class failure reporting with minimal counterexamples

## Source Material References

### Original Hedgehog
- **Haskell Hedgehog**: https://github.com/hedgehogqa/haskell-hedgehog
- **Website**: https://hedgehog.qa/
- **Key Talk**: "Gens N' Roses: Appetite for Reduction" by Jacob Stanley at YOW! Lambda Jam 2017
- **Key modules**: `Hedgehog.Internal.Property`, `Hedgehog.Internal.Gen`, `Hedgehog.Internal.Tree`

### Other Language Ports
- **F# Hedgehog**: https://github.com/hedgehogqa/fsharp-hedgehog
- **R Hedgehog**: https://github.com/hedgehogqa/r-hedgehog
- **Previous Rust attempt**: https://github.com/Centril/proptest (different approach)

### Related Libraries
- **QuickCheck (Haskell)**: https://github.com/nick8325/quickcheck
- **QuickCheck (Rust)**: https://github.com/BurntSushi/quickcheck
- **PropTest (Rust)**: https://github.com/proptest-rs/proptest
- **Hypothesis (Python)**: https://github.com/HypothesisWorks/hypothesis

### Research Papers
- **"QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs"** - Claessen & Hughes (2000)
- **"Shrinking and showing functions"** - Claessen (2012)
- **"Integrated Shrinking"** - Stanley (2017)
- **"Generating Good Generators for Inductive Relations"** - Lampropoulos et al. (2017)

### Key Concepts Documentation
- **Rose Trees**: https://en.wikipedia.org/wiki/Rose_tree
- **Property-based testing**: https://hypothesis.works/articles/what-is-property-based-testing/
- **Shrinking in property testing**: https://www.well-typed.com/blog/2019/05/integrated-shrinking/

## Architecture Decisions

Based on analysis of existing ports and Rust's constraints:

### Core Approach
- **Closure-based generators** - `Gen<T>` wraps `Box<dyn Fn(Size, Seed) -> Tree<T>>`
- **Eager trees with lazy shrinking** - Pre-compute some shrinks, lazy evaluation for others
- **Trait-based internals** - Zero-cost abstractions where possible
- **Procedural macros** - Ergonomic property testing syntax

### Key Design Principles
1. **Preserve Hedgehog's explicit nature** - No QuickCheck-style type direction
2. **Embrace Rust idioms** - Use traits, iterators, and zero-cost abstractions
3. **Excellent debugging** - Rich failure reports with source location tracking
4. **Performance focus** - Avoid the `Rc<RefCell<T>>` trap of the previous port

## Phase 1: Foundation (Weeks 1-4)

### Goals
- Establish core abstractions
- Basic generator combinators
- Simple property testing
- Proof of concept

### Deliverables

#### Core Types (`hedgehog-core/src/data.rs`)
- [x] `Size` - Size parameter for generation scaling
- [x] `Seed` - Splittable random seed
- [x] `Config` - Property testing configuration
- [ ] `Journal` - Annotation system for debugging

#### Error Handling (`hedgehog-core/src/error.rs`)
- [x] `HedgehogError` - Main error type
- [x] `TestResult` - Property test outcomes
- [ ] `FailureReport` - Rich failure information
- [ ] `Annotation` - Debug annotations

#### Tree Structure (`hedgehog-core/src/tree.rs`)
- [x] `Tree<T>` - Rose tree for values and shrinks
- [x] Tree combinators (`map`, `bind`, `filter`)
- [x] Lazy shrinking via closures
- [ ] Tree rendering for debugging

#### Generator Framework (`hedgehog-core/src/gen.rs`)
- [x] `Gen<T>` - Core generator type
- [x] Basic combinators (`map`, `bind`, `filter`)
- [x] Primitive generators (`constant`, `bool`, `int_range`)
- [x] Character generators (`ascii_alpha`, `ascii_alphanumeric`, `ascii_printable`)
- [x] String generators with shrinking
- [x] Vector generators with element removal shrinking
- [x] Option generators with None shrinking
- [x] Tuple generators with component-wise shrinking

#### Property Testing (`hedgehog-core/src/property.rs`)
- [x] `Property<T>` - Property test wrapper
- [x] Basic property runner
- [x] Simple shrinking logic
- [x] Test result reporting

### Success Criteria
- Can write: `Gen::constant(42).check(|x| x == 42)`
- Basic shrinking works for simple cases
- Clear error messages for failures
- No performance regressions vs manual testing

## Phase 2: Core Functionality (Weeks 5-8)

### Goals
- Complete generator combinator library
- Robust shrinking system
- Range-based generation
- Basic debugging support

### Deliverables

#### Random Generation (`hedgehog-core/src/random.rs`)
- [ ] `Random<T>` - Pure random value generation
- [ ] Seed splitting and management
- [ ] Integration with `rand` crate
- [ ] Deterministic generation

#### Range System (`hedgehog-core/src/range.rs`)
- [ ] `Range<T>` - Bounds and scaling for generation
- [ ] Range combinators (`linear`, `exponential`, `constant`)
- [ ] Size-dependent scaling
- [ ] Origin-based shrinking

#### Shrinking (`hedgehog-core/src/shrink.rs`)
- [ ] Shrinking strategies for primitive types
- [ ] Collection shrinking (lists, vectors)
- [ ] Integrated shrinking in generators
- [ ] Shrink path optimization

#### Extended Generators
- [ ] Numeric generators with ranges
- [ ] String and character generators
- [ ] Recursive generator support
- [ ] Frequency-weighted choice

#### Basic Debugging
- [ ] Source location tracking
- [ ] Counterexample formatting
- [ ] Basic annotation system
- [ ] Minimal failure reports

### Success Criteria
- Can generate complex nested structures
- Shrinking produces minimal counterexamples
- Range-based generation works correctly
- Basic debugging output is helpful

## Phase 3: Advanced Features (Weeks 9-12)

### Goals
- Rich debugging experience
- Advanced generator patterns
- Performance optimization
- Integration testing

### Deliverables

#### Rich Debugging (`hedgehog-core/src/debug.rs`)
- [ ] Diff visualization for failures
- [ ] Shrink path visualization
- [ ] Rich annotation system
- [ ] Source location integration

#### Advanced Generators
- [ ] Dependent generation patterns
- [ ] Recursive structure generation
- [ ] State machine testing support
- [ ] Custom generator traits

#### Performance Optimization
- [ ] Benchmark suite
- [ ] Memory usage optimization
- [ ] Shrinking performance improvements
- [ ] Zero-cost abstraction verification

#### Integration
- [ ] `cargo test` integration
- [ ] Test framework compatibility
- [ ] CI/CD friendly output
- [ ] Documentation and examples

### Success Criteria
- Debugging experience matches Haskell Hedgehog
- Performance competitive with QuickCheck
- Easy integration with existing test suites
- Comprehensive documentation

## Phase 4: Ecosystem Integration (Weeks 13-16)

### Goals
- Procedural macros for ergonomics
- Ecosystem compatibility
- Production readiness
- Community adoption

### Deliverables

#### Procedural Macros (`hedgehog-derive/src/lib.rs`)
- [ ] `#[derive(Arbitrary)]` for automatic generators
- [ ] `property!` macro for ergonomic property testing
- [ ] `quickcheck!` macro for QuickCheck compatibility
- [ ] Source location tracking in macros

#### Main Crate (`hedgehog/src/lib.rs`)
- [ ] Clean public API
- [ ] Re-exports and prelude
- [ ] Documentation and examples
- [ ] Migration guide from QuickCheck

#### Ecosystem Integration
- [ ] `serde` support for serializable types
- [ ] `proptest` compatibility layer
- [ ] `quickcheck` migration tools
- [ ] IDE integration support

#### Production Readiness
- [ ] Comprehensive test suite
- [ ] Security audit
- [ ] Performance benchmarks
- [ ] Stability guarantees

### Success Criteria
- Ergonomic API competitive with other libraries
- Seamless integration with Rust ecosystem
- Clear migration path from existing libraries
- Ready for production use

## Phase 5: Advanced Features (Weeks 17-20)

### Goals
- State machine testing
- Advanced shrinking strategies
- Custom generator ecosystem
- Performance leadership

### Deliverables

#### State Machine Testing
- [ ] State machine testing framework
- [ ] Action generation and execution
- [ ] State invariant checking
- [ ] Parallel execution testing

#### Advanced Shrinking
- [ ] Smart shrinking strategies
- [ ] Shrink combination optimization
- [ ] Custom shrinking functions
- [ ] Shrink tree visualization

#### Extensibility
- [ ] Plugin system for custom generators
- [ ] Custom property types
- [ ] Extension traits for third-party types
- [ ] Generator composition patterns

#### Performance Leadership
- [ ] Parallel property testing
- [ ] Incremental shrinking
- [ ] Memory-efficient tree structures
- [ ] Benchmark against all competitors

### Success Criteria
- Best-in-class performance
- Most comprehensive feature set
- Extensible architecture
- Industry adoption

## Success Metrics

### Technical Metrics
- **Performance**: Faster than QuickCheck for equivalent tests
- **Memory**: Lower memory usage than failed Rust port
- **Ergonomics**: Fewer lines of code than QuickCheck for same tests
- **Debugging**: Richer failure reports than any existing library

### Adoption Metrics
- **Community**: Active contributor base
- **Usage**: Adoption by major Rust projects
- **Ecosystem**: Third-party extensions and integrations
- **Documentation**: Comprehensive guides and examples

## Risk Mitigation

### Technical Risks
- **Lazy evaluation**: Mitigated by closure-based approach
- **Performance**: Continuous benchmarking and optimization
- **Complexity**: Phased delivery with clear milestones
- **Ecosystem fit**: Early integration testing

### Project Risks
- **Scope creep**: Strict phase boundaries
- **Perfectionism**: MVP approach with iterative improvement
- **Compatibility**: Extensive testing with existing codebases
- **Maintenance**: Clear documentation and contributor guidelines

## Known Issues and Improvements

### Tree Rendering and Debugging Output

**Current limitation**: Our tree rendering differs significantly from Haskell Hedgehog's debugging experience:

**Haskell Hedgehog shows the shrinking *process***:
```
✗ reverse involutive failed at size 2 after 1 test and 2 shrinks.

      ┏━━ test/Test.hs ━━━
   20 ┃ prop_reverse_involutive :: Property
   21 ┃ prop_reverse_involutive = property $ do
   22 ┃   xs <- forAll $ Gen.list (Range.linear 0 100) Gen.alpha
   23 ┃   reverse (reverse xs) === xs
      ┃   │ forAll 0 = "ab"
      ┃   │ forAll 1 = "a"    -- shrink step 1
      ┃   │ forAll 2 = ""     -- shrink step 2 (minimal)
```

**Our implementation shows shrinking *possibilities***:
- Static tree visualization showing all potential shrinks
- No integration with property failure output
- Missing step-by-step progression and source location info

**Missing features**:
1. **Integrated failure reporting** - `TestResult::Fail` should show shrinking steps
2. **Step functionality** - Ability to walk through shrinks one by one like `--hedgehog-shrinks N`
3. **Rich annotations** - Show variable names and progression 
4. **Better tree traversal** - Show the path taken during shrinking, not just all possibilities
5. **Display trait limitations** - Tree rendering only works for `T: Display`, not `T: Debug`

**Priority**: High - This significantly impacts the debugging experience

### String Generator Issues

**Current limitation**: String generator can produce unexpected results due to size-dependent length:

```rust
let (length, _) = len_seed.next_bounded(size.get() as u64 + 1);
```

With `Size::new(4)`, possible lengths are 0-4. Certain seeds produce empty strings even when non-empty strings are expected.

**Issues**:
1. **Unpredictable length distribution** - Size-dependent randomness may not match user expectations
2. **Empty string frequency** - Higher than expected for small sizes
3. **No minimum length control** - Cannot specify "at least N characters"

**Root cause**: We conflate Size (generation complexity) with Range (value bounds), unlike Haskell Hedgehog which separates these concerns.

**Haskell Hedgehog approach**:
```haskell
Gen.string (Range.linear 0 100) Gen.alpha     -- Uniform 0-100 length
Gen.string (Range.exponential 1 100) Gen.alpha -- Bias towards shorter
Gen.string (Range.constant 5) Gen.alpha        -- Always length 5
```

**Erlang PropEr distribution shaping**:
```erlang
?LET(N, weighted_union([{10, 0}, {90, range(1, 100)}]),
     vector(N, char()))  % 10% empty, 90% non-empty
```

**Potential solutions**:
1. **Range-based string generation** - `Gen::<String>::with_range(Range::linear(1, 10), Gen::ascii_alpha())`
2. **Distribution shaping** - `Gen::<String>::weighted_length([{10, 0}, {90, range(1, 20)}])`
3. **Separate size and range** - Size affects recursion depth, Range controls length distribution
4. **Convenience methods** - `Gen::<String>::non_empty_alpha()`, `Gen::<String>::short_alpha()`

**Priority**: Medium - Affects usability but has workarounds. Should implement proper Range system first.

### Display vs Debug Rendering

**Current limitation**: Tree rendering methods require `T: std::fmt::Display` but many types only implement `Debug`:

```rust
impl<T> Tree<T> where T: std::fmt::Display {
    pub fn render(&self) -> String { ... }
}
```

This prevents rendering for `Vec<T>`, `Option<T>`, and most custom types.

**Potential solutions**:
1. **Debug-based rendering methods** - Additional methods using `Debug` formatting
2. **Generic formatting trait** - Accept either `Display` or `Debug`  
3. **Separate rendering traits** - More flexible approach

**Priority**: Medium - Good for developer experience

### Distribution Shaping and Weighted Generation

**Current limitation**: We lack sophisticated distribution control found in mature property testing libraries.

**Haskell Hedgehog has Range combinators**:
```haskell
Range.linear 0 100        -- Uniform distribution
Range.exponential 1 1000  -- Exponential bias towards smaller values  
Range.singleton 42        -- Always generates 42
Range.constantFrom 10 5 20 -- Around 10, between 5-20
```

**Erlang PropEr has weighted unions**:
```erlang
weighted_union([{1, atom()}, {9, integer()}])  % 10% atoms, 90% integers
frequency([{10, small_int()}, {1, large_int()}]) % Bias towards small ints
```

**QuickCheck has frequency and oneof**:
```haskell
frequency [(1, return 0), (9, choose (1, 100))]  -- 10% zeros, 90% positive
```

**Missing in our implementation**:
1. **Weighted choice generators** - `Gen::frequency([(weight, gen), ...])`
2. **Range combinators** - Linear, exponential, constant distributions  
3. **Biased generation** - Easy ways to create realistic distributions
4. **Size-sensitive scaling** - Ranges that grow with test complexity

**Potential API design**:
```rust
// Weighted choice
Gen::frequency([
    (1, Gen::constant(0)),           // 10% zeros
    (9, Gen::int_range(1, 100))      // 90% positive
])

// Range-based generation  
Gen::int_with_range(Range::exponential(1, 1000))
Gen::string_with_range(Range::linear(0, 50), Gen::ascii_alpha())

// Size-sensitive ranges
Gen::list_with_range(Range::linear(0, size.get()), Gen::int_range(1, 100))
```

**Benefits**:
- **Realistic test data** - Model real-world distributions
- **Edge case control** - Tune frequency of boundary conditions
- **Performance** - Avoid generating overly large test cases
- **User expectations** - Match intuitive behavior (e.g., mostly non-empty strings)

**Priority**: High - Essential for production-quality property testing

### Custom Test Runner CLI

**Current limitation**: Our enhanced test reporting with box drawing characters and proper formatting is only visible when tests fail or when explicitly displayed in examples. For passing tests, users only see standard cargo output:

```
test my_property_test ... ok
```

Instead of our beautiful formatted output:
```
━━━ my_module ━━━
  ✓ my_property passed 100 tests.
```

**Root cause**: Cargo's built-in test harness doesn't provide hooks for custom output formatting. This is a known limitation that affects many testing libraries.

**Solution**: Build a `cargo-hedgehog` CLI tool following the pattern of `cargo-nextest`, `cargo-criterion`, etc.:

```bash
cargo install cargo-hedgehog
cargo hedgehog test  # Shows enhanced reporting for all tests
```

**Benefits**:
- **Familiar pattern** - Users already know similar tools
- **Non-intrusive** - Doesn't require changing existing test code  
- **Optional** - Users can still use standard `cargo test`
- **Full control** - Complete formatting control over output
- **Future-proof** - Can migrate when cargo adds custom test runner support

**Features to implement**:
- Enhanced test reporting with box drawing
- Parallel test execution
- Filtering and selection
- Progress indicators
- Test result aggregation
- JSON output for CI/CD integration

**Priority**: High - Essential for showcasing our enhanced reporting capabilities

## Implementation Priority Order

Based on current analysis and user feedback, the next development priorities are:

1. **Distribution Shaping and Range System** (High Priority)
   - Implement `Range` combinators (linear, exponential, constant)
   - Add `Gen::frequency()` for weighted choice
   - Better control over string lengths and numeric distributions
   - Addresses fundamental usability gaps in current implementation

2. **Input Variable Name Tracking** (Medium Priority)
   - Enhance failure reporting to show variable names in shrinking progression
   - Make output more like Haskell Hedgehog with named inputs
   - Improve debugging experience with contextual information

3. **Derive Macros** (Medium Priority)
   - Add `#[derive(Generate)]` for custom types
   - Improve ergonomics for user-defined types
   - Reduce boilerplate for common generator patterns

4. **Custom CLI Tool** (Medium Priority)
   - Build `cargo-hedgehog` to showcase enhanced reporting
   - Provide proper display for success cases with box drawing
   - Enable full control over test output formatting

## Future Considerations

### Date/Time Generators (Maybe)

Following the Haskell Hedgehog approach, date/time generators are not part of core but could be a separate extension crate.

**Potential approach for `hedgehog-time` crate:**
- Unix epoch-based shrinking (towards 1970-01-01 instead of 0)
- Support for `SystemTime`, `Duration`, and potentially `chrono` types
- Meaningful temporal origins (e.g., year 2000, current date)
- Timezone-aware shrinking strategies

**Benefits of epoch-based shrinking:**
- Avoids invalid dates from shrinking towards 0
- Provides realistic minimal counterexamples
- Maintains temporal meaning in failed test cases

**Implementation considerations:**
- Keep separate from core to avoid dependencies
- Allow custom temporal origins for domain-specific testing
- Support both absolute times and durations
- Consider leap years, timezone complexities

### Configurable Random Number Generators

Currently, Hedgehog uses a hardcoded SplitMix64 PRNG, which provides excellent quality for property testing and matches the choice made by Haskell Hedgehog. However, users may eventually want to configure the RNG algorithm.

**Current limitation**: Users can control the seed but not the algorithm:
```rust
let seed = Seed::from_u64(12345);  // Deterministic seed
let seed = Seed::random();         // System randomness
// But always uses SplitMix64
```

**Potential approaches for configurability**:

#### Option A: RNG Selection in Config
```rust
let config = Config::default()
    .with_rng(RngType::SplitMix64)  // or ChaCha20, Xoshiro, etc.
    .with_tests(100);
```
- **Pros**: Simple API, backward compatible
- **Cons**: Runtime dispatch overhead, limited to predefined algorithms

#### Option B: Generic Seed Type
```rust
struct Property<T, R: SplittableRng> {
    generator: Gen<T, R>,
    // ...
}
```
- **Pros**: Zero-cost abstraction, compile-time selection
- **Cons**: Major API complexity, significant refactoring required

#### Option C: Trait-Based Runtime Dispatch
```rust
trait SplittableRng {
    fn split(self) -> (Self, Self);
    fn next_u64(self) -> (u64, Self);
}
```
- **Pros**: Flexible, allows custom RNGs
- **Cons**: Runtime overhead, more complex implementation

**Recommendation**: Defer until there's concrete user demand. SplitMix64 is proven for property testing and adding configurability would complicate the API without clear benefit. Most users care about test quality, not RNG algorithm choice.

**Requirements for any RNG**: Must be splittable for deterministic property testing. This rules out most standard PRNGs that aren't designed for splitting.

## Conclusion

This roadmap provides a clear path from foundation to production-ready library. Each phase builds incrementally on previous work, with clear success criteria and deliverables. The emphasis on preserving Hedgehog's core value proposition while embracing Rust's strengths should result in a library that's both powerful and ergonomic.

The phased approach allows for early validation of core concepts while building toward a comprehensive property testing solution that could become the standard for Rust property-based testing.