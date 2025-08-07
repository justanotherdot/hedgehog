# Roadmap

## Current Status

✅ **Core Features Complete**
- Distribution shaping and range system
- Variable name tracking in failure reporting  
- Derive macros for custom types
- Comprehensive property testing API
- State machine testing utilities
- Function generators for higher-order function testing

## Next Priorities

**Note**: Based on implementation velocity, these features can be completed in 2-3 weeks total.

1. **Core Extensions** (Week 1: 3-4 days)
   - Example integration - mix explicit examples with generated tests
   - Property classification - see distribution of test data
   - Dictionary support - domain-specific token injection

2. **Major Features** (Week 2: 1 week)
   - Coverage-guided generation - use coverage feedback

3. **Advanced Features** (Week 3: 1-2 weeks)
   - Regression corpus - automatic failure persistence
   - Parallel testing - find race conditions
   - Fault injection - systematic failure testing

4. **Performance & Benchmarking**
   - Optimize generator performance
   - Benchmark against other property testing libraries

5. **Additional Generators**
   - Date/time generators
   - Network/protocol generators
   - File system generators

6. **Documentation & Examples**
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