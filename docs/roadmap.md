# Roadmap

## Current Status

✅ **Core Features Complete**
- Distribution shaping and range system
- Variable name tracking in failure reporting  
- Derive macros for custom types
- Comprehensive property testing API

## Next Priorities

1. **Custom CLI Tool** (cargo-hedgehog)
   - Property test runner with better output
   - Integration with existing test frameworks

2. **Performance & Benchmarking**
   - Optimize generator performance
   - Benchmark against other property testing libraries

3. **Additional Generators**
   - Date/time generators
   - Network/protocol generators
   - File system generators

## Future Considerations

- **Attribute Customization** for derive macros
- **Generic Type Support** in derive macros
- **State Machine Testing** utilities
- **Parallel Property Testing**
- **Custom RNG Support**

## Non-Goals

- Type-directed generation (we prefer explicit generators)
- Replacing existing test frameworks (we integrate with them)
- Complex DSLs (we prefer simple, composable APIs)

This roadmap focuses on practical features that enhance the developer experience while maintaining Hedgehog's core philosophy of explicit, composable generators.