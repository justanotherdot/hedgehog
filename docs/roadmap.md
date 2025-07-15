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
- [ ] `Tree<T>` - Rose tree for values and shrinks
- [ ] Tree combinators (`map`, `bind`, `filter`)
- [ ] Lazy shrinking via closures
- [ ] Tree rendering for debugging

#### Generator Framework (`hedgehog-core/src/gen.rs`)
- [ ] `Gen<T>` - Core generator type
- [ ] Basic combinators (`map`, `bind`, `filter`)
- [ ] Primitive generators (`constant`, `choice`)
- [ ] Collection generators (`vec`, `option`)

#### Property Testing (`hedgehog-core/src/property.rs`)
- [ ] `Property<T>` - Property test wrapper
- [ ] Basic property runner
- [ ] Simple shrinking logic
- [ ] Test result reporting

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

## Future Considerations

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