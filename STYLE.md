# Hedgehog Style Guide

This document outlines coding and naming conventions for the Hedgehog project.

## File Naming

**Always use kebab-case for file names, type tags, and unique identifiers.**

✅ **Good:**
- `example-integration.rs`
- `quickstart-example.rs` 
- `function-generators.rs`
- `derive-macro-test.rs`

❌ **Avoid:**
- `example_integration.rs` (snake_case)
- `quickstart_example.rs` (snake_case)
- `ExampleIntegration.rs` (PascalCase)

This applies to all files: source files, test files, example files, and documentation.

## Terminal Output

**Never use ANSI color codes in command-line tools. Always start without color.**

✅ **Enforced:**
- `.cargo/config.toml` sets `color = "never"`
- All scripts in `bin/` use `--color never`
- No ANSI escape sequences in source code

❌ **Avoid:**
- Color output in any terminal tools
- ANSI escape codes (`\x1b[`, `\033[`, etc.)
- Dependencies that default to colored output

This ensures consistent, clean output across all environments and follows the project's preference for minimal, functional interfaces.

## Code Identifiers

Use noun-verb pattern for code identifiers:
- `userCreate`, `workspaceDelete`, `dataProcess`
- Reduces cognitive load and clarifies the range of possible verbs

## Documentation

- Put documentation closest to source when possible
- Use `docs/` for comprehensive documentation  
- `README.md`, `LICENSE`, `STYLE.md`, and `CONTRIBUTING.md` go at top level

## Scripts

- Scripts go in `bin/` at project top level
- All executable binaries have no file suffix (use shebangs)

## Error Handling

- Always use structured errors unless for throwaway code
- Use errors as return values rather than exceptions

For complete style guidelines, see the project's CLAUDE.md configuration.