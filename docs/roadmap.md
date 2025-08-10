# Roadmap

## Current Status

✅ **Core Features Complete**
- Distribution shaping and range system
- Variable name tracking in failure reporting  
- Derive macros for custom types
- Comprehensive property testing API
- State machine testing utilities
- Function generators for higher-order function testing
- Example integration with configurable strategies
- Dictionary support - domain-specific token injection (web domains, HTTP codes, SQL keywords, etc.)
- Parallel testing - multi-threaded property execution with race condition detection
- Targeted property-based testing - search-guided generation using simulated annealing

✅ **Meta Testing & Quality Assurance Complete**
- **Meta Testing Suite** - Property-based testing of Hedgehog itself with 28+ comprehensive tests covering:
  - Generator invariant testing (size bounds, distribution properties)
  - Shrinking correctness properties (always produces smaller failures) 
  - Property combinator correctness (classifications, collections, examples)
  - Statistical distribution validation
  - Performance property testing (generation/shrinking time bounds)
  - Combinator law verification (map, bind, filter composition)
  - Edge case handling (empty ranges, extreme values, integer overflow)
  - Result/Option generator meta-properties
  - String generation properties (length, character sets, unicode handling)
  - Targeted testing properties (convergence, search effectiveness)
  - Parallel testing properties (thread safety, deadlock detection)
  - State machine testing properties (command generation, execution consistency)
  - Integration workflow testing (complex multi-step scenarios)
- **Curated Test Data Collections** - Ported realistic test data from haskell-hedgehog:
  - Muppet characters, animals, colors, fruits, vegetables for domain-specific testing
  - Unicode Glass Suite with 150+ "I can eat glass" phrases in different languages/scripts
  - Metasyntactic variables, weather conditions, bodies of water
  - Practical usage examples for web validation, i18n testing, game systems
  - Generator functions for convenient access to curated data
- **Flakiness Prevention** - Robust probabilistic test assertions that prevent CI failures
- **100% Test Coverage** - All major Hedgehog features comprehensively tested

## Next Priorities

1. **Enhanced Distribution Support** (Priority: High)
   - Normal/Gaussian distribution - bell curve around mean
   - Beta distribution - flexible bounded distributions for modeling percentages  
   - Gamma distribution - right-skewed continuous for modeling wait times
   - Poisson distribution - discrete event modeling (request rates, failures)
   - Binomial distribution - success/failure trials (A/B testing, reliability)
   - Pareto distribution - power law/"80-20 rule" (file sizes, wealth distribution)
   - Zipf distribution - frequency ranking (text analysis, web traffic, social media)
   - Weibull distribution - reliability/survival analysis (failure rates, lifetimes)
   - Custom distribution sampling framework

2. **Regression Corpus System** (Priority: High)
   - Automatic failure case persistence - save failing inputs to corpus files
   - Configurable corpus replay - test saved cases first on subsequent runs
   - Corpus management - pruning, merging, organizing regression cases
   - Integration with CI/CD - persistent corpus across test runs
   - Similar to proptest's `.proptest-regressions/` or AFL's corpus directories

3. **Advanced Features** (Priority: Medium)
   - Coverage-guided generation - use coverage feedback to explore code paths
   - Fault injection - systematic failure testing

4. **Performance & Benchmarking** (Priority: Medium)
   - Optimize generator performance
   - Benchmark against other property testing libraries
   - Memory usage optimization for large test suites

5. **Additional Generators** (Priority: Low)
   - Date/time generators with timezone support
   - Network/protocol generators (IP addresses, URLs, HTTP headers)
   - File system generators (paths, permissions, content types)
   - JSON/XML structure generators
   - Database schema generators

6. **Documentation & Ecosystem** (Priority: Medium)
   - Comprehensive user guide with advanced patterns
   - Property testing best practices guide
   - Integration examples with popular Rust frameworks (tokio, actix, axum)
   - Cookbook for common testing scenarios

## Future Considerations

- **Attribute Customization** for derive macros
- **Generic Type Support** in derive macros
- **State Machine Testing** utilities
- **Regression Corpus** - save failing cases to a corpus file that gets tried first on subsequent runs (similar to proptest's approach)
- **Custom RNG Support**

## Non-Goals

- Type-directed generation (we prefer explicit generators)
- Replacing existing test frameworks (we integrate with them)
- Complex DSLs (we prefer simple, composable APIs)

This roadmap focuses on practical features that enhance the developer experience while maintaining Hedgehog's core philosophy of explicit, composable generators.