# Hedgehog CLI

Enhanced command-line interface for Hedgehog property-based testing.

## Installation

From the hedgehog workspace directory:

```sh
cargo install --path hedgehog-cli
```

## Usage

### Run property tests

```sh
# Basic usage
cargo hedgehog test

# With options
cargo hedgehog test --verbose --count 500 --derive --release

# Run specific test pattern
cargo hedgehog test --test integration_test --verbose
```

**Options:**
- `-t, --test <TEST>` - Test name pattern to run
- `-c, --count <COUNT>` - Number of tests to run for each property (default: 100)
- `-v, --verbose` - Enable verbose output
- `--show-all` - Show all generated values (very verbose)
- `--derive` - Enable derive feature
- `--release` - Run tests in release mode

### Generate test reports

```sh
# Generate JSON report
cargo hedgehog report --format json --output report.json

# Generate HTML report
cargo hedgehog report --format html --output report.html

# Generate markdown report (default)
cargo hedgehog report --format markdown
```

**Options:**
- `-f, --format <FORMAT>` - Output format: json, markdown, html (default: markdown)
- `-o, --output <OUTPUT>` - Output file (default: stdout)

### Show test coverage

```sh
# Basic coverage
cargo hedgehog coverage

# Detailed coverage
cargo hedgehog coverage --detailed
```

**Options:**
- `-d, --detailed` - Show detailed coverage per property

## Output formatting

The CLI provides enhanced output formatting with color-coded prefixes:

- `SEARCH:` - Property test search operations
- `FAIL:` - Property test failures
- `COUNTER:` - Counterexamples found
- `SHRINK:` - Shrinking operations
- `PASS:` - Successful property tests

## Examples

### Basic property test run

```sh
cargo hedgehog test --verbose
```

Output:
```
Hedgehog Property Testing
=========================

Configuration:
  Test count: 100
  Derive feature: disabled
  Release mode: disabled

Running: cargo test

All property tests passed!
```

### Generate JSON report

```sh
cargo hedgehog report --format json --output test-report.json
```

Creates a structured JSON report with test results and metadata.

### Check test coverage

```sh
cargo hedgehog coverage --detailed
```

Shows which properties are covered by tests and their status.

## Integration with CI/CD

The CLI is designed to work well in continuous integration environments:

- Exit codes indicate test success/failure
- Supports both colored and plain text output
- JSON reports for automated processing
- Verbose mode for debugging

## Environment variables

The CLI respects these environment variables:

- `HEDGEHOG_TEST_COUNT` - Default test count
- `HEDGEHOG_VERBOSE` - Enable verbose output
- `HEDGEHOG_SHOW_ALL` - Show all generated values
- `NO_COLOR` - Disable colored output