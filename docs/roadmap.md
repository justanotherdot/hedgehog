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

## Next Priorities

**Note**: Based on implementation velocity, these features can be completed in 2-3 weeks total.

2. **Major Features** (Week 2: 1 week)
   - Coverage-guided generation - use coverage feedback

3. **Advanced Features** (Week 3: 1-2 weeks)
   - Regression corpus - automatic failure persistence
   - Advanced parallel features - systematic interleaving exploration, concurrent scenario DSL
   - Fault injection - systematic failure testing

4. **Meta Testing** (Week 4: 1 week)
   - Property-based testing of Hedgehog itself
   - Generator invariant testing (size bounds, distribution properties)
   - Shrinking correctness properties (always produces smaller failures)
   - Property combinator correctness (classifications, collections, examples)
   - Statistical distribution validation
   - Performance property testing (generation/shrinking time bounds)

5. **Performance & Benchmarking**
   - Optimize generator performance
   - Benchmark against other property testing libraries

6. **Additional Generators**
   - Date/time generators
   - Network/protocol generators
   - File system generators

7. **Documentation & Examples**
   - Comprehensive user guide
   - Real-world examples and tutorials

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