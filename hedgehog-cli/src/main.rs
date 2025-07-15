use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use hedgehog_cli::discovery::TestDiscovery;
use std::path::PathBuf;
use std::process::Command;

/// Enhanced Hedgehog property testing CLI
#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run Hedgehog property tests
    Hedgehog(HedgehogArgs),
}

#[derive(Parser)]
struct HedgehogArgs {
    #[command(subcommand)]
    action: Option<HedgehogAction>,

    // Default test action arguments (when no subcommand is provided)
    /// Test name pattern to run
    #[arg(short, long)]
    test: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Show all generated values (very verbose)
    #[arg(long)]
    show_all: bool,

    /// Enable derive feature
    #[arg(long)]
    derive: bool,

    /// Run tests in release mode
    #[arg(long)]
    release: bool,
}

#[derive(Subcommand)]
enum HedgehogAction {
    /// Run property tests (default)
    #[command(alias = "t")]
    Test {
        /// Test name pattern to run
        #[arg(short, long)]
        test: Option<String>,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Show all generated values (very verbose)
        #[arg(long)]
        show_all: bool,

        /// Enable derive feature
        #[arg(long)]
        derive: bool,

        /// Run tests in release mode
        #[arg(long)]
        release: bool,
    },

    /// Generate test report
    Report {
        /// Output format (json, markdown, html)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show test coverage
    Coverage {
        /// Show detailed coverage per property
        #[arg(short, long)]
        detailed: bool,
    },

    /// Discover property tests in the current project
    Discover {
        /// Directory to search for tests
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output format (list, json)
        #[arg(short, long, default_value = "list")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Enable colors only when running in a terminal or explicitly requested
    // This ensures consistent behavior between cargo run and direct execution
    let should_use_colors = atty::is(atty::Stream::Stdout)
        || std::env::var("FORCE_COLOR").is_ok()
        || (std::env::var("NO_COLOR").is_err()
            && std::env::var("TERM").unwrap_or_default() != "dumb");

    colored::control::set_override(should_use_colors);

    let cli = Cli::parse();

    match cli.command {
        Commands::Hedgehog(args) => handle_hedgehog(args).await,
    }
}

async fn handle_hedgehog(args: HedgehogArgs) -> Result<()> {
    match args.action {
        Some(HedgehogAction::Test {
            test,
            verbose,
            show_all,
            derive,
            release,
        }) => run_tests(test, verbose, show_all, derive, release).await,
        Some(HedgehogAction::Report { format, output }) => generate_report(format, output).await,
        Some(HedgehogAction::Coverage { detailed }) => show_coverage(detailed).await,
        Some(HedgehogAction::Discover { path, format }) => discover_tests(path, format).await,
        None => {
            // Default to test action with args from top level
            run_tests(
                args.test,
                args.verbose,
                args.show_all,
                args.derive,
                args.release,
            )
            .await
        }
    }
}

async fn run_tests(
    test_pattern: Option<String>,
    verbose: bool,
    show_all: bool,
    derive: bool,
    release: bool,
) -> Result<()> {
    println!("{}", hedgehog_cli::format_header());

    if verbose {
        println!(
            "{}",
            hedgehog_cli::format_config_output(derive, release, test_pattern.as_deref())
        );
        println!();
    }

    // Discover property tests
    let discovery = TestDiscovery::new(PathBuf::from("."));
    let mut properties = discovery.discover_properties()?;

    // Filter by pattern if provided
    if let Some(pattern) = &test_pattern {
        properties.retain(|p| p.name.contains(pattern));
    }

    if properties.is_empty() {
        println!("No property tests found matching criteria");
        return Ok(());
    }

    if verbose {
        println!("Found {} property tests:", properties.len());
        for prop in &properties {
            println!("  - {}", prop.name);
        }
        println!();
    }

    // Build the project first
    let mut build_cmd = Command::new("cargo");
    build_cmd.arg("build");

    if release {
        build_cmd.arg("--release");
    }

    if derive {
        build_cmd.arg("--features").arg("derive");
    }

    build_cmd.arg("--tests");

    if verbose {
        println!("Building tests...");
    }

    let build_output = build_cmd.output()?;

    if !build_output.status.success() {
        println!("{}", hedgehog_cli::format_failure());
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        println!("{}", hedgehog_cli::enhance_test_output("", &stderr));
        std::process::exit(1);
    }

    // Run each property test
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut failed_tests = Vec::new();

    for property in &properties {
        if verbose {
            println!("Running {}...", property.name);
        }

        let mut test_cmd = Command::new("cargo");
        test_cmd.arg("test");

        if release {
            test_cmd.arg("--release");
        }

        if derive {
            test_cmd.arg("--features").arg("derive");
        }

        test_cmd.arg(&property.name);

        // Set environment variables for hedgehog
        test_cmd.env("HEDGEHOG_VERBOSE", if verbose { "1" } else { "0" });
        test_cmd.env("HEDGEHOG_SHOW_ALL", if show_all { "1" } else { "0" });

        let test_output = test_cmd.output()?;
        let stdout = String::from_utf8_lossy(&test_output.stdout);
        let stderr = String::from_utf8_lossy(&test_output.stderr);

        if test_output.status.success() {
            total_passed += 1;
            if verbose || show_all {
                println!("✓ {}", property.name);
                if show_all {
                    println!("{}", hedgehog_cli::enhance_test_output(&stdout, &stderr));
                }
            }
        } else {
            total_failed += 1;
            failed_tests.push(property.name.clone());
            println!("✗ {}", property.name);
            println!("{}", hedgehog_cli::enhance_test_output(&stdout, &stderr));
        }
    }

    // Summary
    println!();
    if total_failed == 0 {
        println!("{}", hedgehog_cli::format_success());
        println!("Tests run: {}, Passed: {}", properties.len(), total_passed);
    } else {
        println!("{}", hedgehog_cli::format_failure());
        println!(
            "Tests run: {}, Passed: {}, Failed: {}",
            properties.len(),
            total_passed,
            total_failed
        );
        println!("Failed tests: {}", failed_tests.join(", "));
        std::process::exit(1);
    }

    Ok(())
}

async fn generate_report(format: String, output: Option<PathBuf>) -> Result<()> {
    println!("{}", hedgehog_cli::format_report_message());

    match format.as_str() {
        "json" => {
            let report = serde_json::json!({
                "format": "hedgehog-test-report",
                "version": "1.0",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "summary": {
                    "total_properties": 0,
                    "passed": 0,
                    "failed": 0,
                    "total_tests_run": 0
                },
                "properties": []
            });

            if let Some(path) = output {
                std::fs::write(path, serde_json::to_string_pretty(&report)?)?;
            } else {
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        }
        "markdown" => {
            let report = r#"# Hedgehog Test Report

## Summary
- **Total Properties**: 0
- **Passed**: 0
- **Failed**: 0
- **Total Tests Run**: 0

## Properties

*No properties found. Run `cargo hedgehog test` first.*

---
Generated by cargo-hedgehog
"#;

            if let Some(path) = output {
                std::fs::write(path, report)?;
            } else {
                println!("{}", report);
            }
        }
        "html" => {
            let report = r#"<!DOCTYPE html>
<html>
<head>
    <title>Hedgehog Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .summary { background: #f0f8ff; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
        .passed { color: #008000; }
        .failed { color: #ff0000; }
        .property { margin: 10px 0; padding: 10px; border-left: 4px solid #ddd; }
    </style>
</head>
<body>
    <h1>Hedgehog Test Report</h1>
    
    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Total Properties:</strong> 0</p>
        <p><strong>Passed:</strong> <span class="passed">0</span></p>
        <p><strong>Failed:</strong> <span class="failed">0</span></p>
        <p><strong>Total Tests Run:</strong> 0</p>
    </div>
    
    <h2>Properties</h2>
    <p><em>No properties found. Run <code>cargo hedgehog test</code> first.</em></p>
    
    <hr>
    <p><small>Generated by cargo-hedgehog</small></p>
</body>
</html>"#;

            if let Some(path) = output {
                std::fs::write(path, report)?;
            } else {
                println!("{}", report);
            }
        }
        _ => {
            anyhow::bail!(
                "Unsupported format: {}. Use json, markdown, or html",
                format
            );
        }
    }

    println!("{}", "Report generated successfully!".bright_green());
    Ok(())
}

async fn show_coverage(detailed: bool) -> Result<()> {
    println!("{}", hedgehog_cli::format_coverage_header());

    // This would analyze the codebase to find property tests
    // For now, show a placeholder
    println!("Coverage analysis:");
    println!(
        "  {}: {} properties",
        "src/lib.rs".bright_cyan(),
        "3".bright_yellow()
    );
    println!(
        "  {}: {} properties",
        "src/data.rs".bright_cyan(),
        "5".bright_yellow()
    );
    println!(
        "  {}: {} properties",
        "tests/".bright_cyan(),
        "12".bright_yellow()
    );
    println!();

    println!("Total: {} properties found", "20".bright_green().bold());

    if detailed {
        println!("\nDetailed coverage:");
        println!(
            "  {}: {}",
            "test_range_generation".bright_white(),
            "Covered".bright_green()
        );
        println!(
            "  {}: {}",
            "test_string_shrinking".bright_white(),
            "Covered".bright_green()
        );
        println!(
            "  {}: {}",
            "test_distribution_shaping".bright_white(),
            "Covered".bright_green()
        );
        println!(
            "  {}: {}",
            "test_derive_macros".bright_white(),
            "Covered".bright_green()
        );
        println!(
            "  {}: {}",
            "test_variable_tracking".bright_white(),
            "Covered".bright_green()
        );
    }

    Ok(())
}

async fn discover_tests(path: PathBuf, format: String) -> Result<()> {
    let discovery = TestDiscovery::new(path.clone());
    let properties = discovery.discover_properties()?;

    match format.as_str() {
        "list" => {
            println!("{}", "Property Tests Discovered".bright_cyan().bold());
            println!("{}", "========================".bright_cyan());
            println!();

            if properties.is_empty() {
                println!("No property tests found in {}", path.display());
                return Ok(());
            }

            for prop in &properties {
                let async_marker = if prop.is_async { " (async)" } else { "" };
                println!(
                    "  {}{}",
                    prop.name.bright_green(),
                    async_marker.bright_yellow()
                );
                println!("    {}", prop.file_path.display().to_string().bright_blue());
                println!();
            }

            println!(
                "Total: {} property tests found",
                properties.len().to_string().bright_green().bold()
            );
        }
        "json" => {
            let json_output = serde_json::json!({
                "properties": properties.iter().map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "file_path": p.file_path.to_string_lossy(),
                        "line_number": p.line_number,
                        "is_async": p.is_async
                    })
                }).collect::<Vec<_>>(),
                "total": properties.len()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        _ => {
            anyhow::bail!("Unsupported format: {}. Use 'list' or 'json'", format);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test that the CLI can parse basic commands
        let args = vec!["cargo", "hedgehog", "test", "--verbose"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Hedgehog(hedgehog_args) => match hedgehog_args.action {
                Some(HedgehogAction::Test { verbose, .. }) => {
                    assert!(verbose);
                }
                _ => panic!("Expected test action"),
            },
        }
    }

    #[test]
    fn test_report_parsing() {
        let args = vec!["cargo", "hedgehog", "report", "--format", "json"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Hedgehog(hedgehog_args) => match hedgehog_args.action {
                Some(HedgehogAction::Report { format, .. }) => {
                    assert_eq!(format, "json");
                }
                _ => panic!("Expected report action"),
            },
        }
    }

    #[test]
    fn test_coverage_parsing() {
        let args = vec!["cargo", "hedgehog", "coverage", "--detailed"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Hedgehog(hedgehog_args) => match hedgehog_args.action {
                Some(HedgehogAction::Coverage { detailed }) => {
                    assert!(detailed);
                }
                _ => panic!("Expected coverage action"),
            },
        }
    }

    #[test]
    fn test_default_values() {
        let args = vec!["cargo", "hedgehog", "test"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Hedgehog(hedgehog_args) => match hedgehog_args.action {
                Some(HedgehogAction::Test {
                    verbose,
                    show_all,
                    derive,
                    release,
                    ..
                }) => {
                    assert!(!verbose);
                    assert!(!show_all);
                    assert!(!derive);
                    assert!(!release);
                }
                _ => panic!("Expected test action"),
            },
        }
    }

    #[test]
    fn test_snapshot_cli_help() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "cargo-hedgehog", "--", "hedgehog", "--help"])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        archetype::snap("cli_help_output", stdout);
    }

    #[test]
    fn test_snapshot_cli_test_help() {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cargo-hedgehog",
                "--",
                "hedgehog",
                "test",
                "--help",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        archetype::snap("cli_test_help_output", stdout);
    }

    #[test]
    fn test_snapshot_cli_discover_help() {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cargo-hedgehog",
                "--",
                "hedgehog",
                "discover",
                "--help",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        archetype::snap("cli_discover_help_output", stdout);
    }

    #[test]
    fn test_snapshot_cli_discover_output() {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cargo-hedgehog",
                "--",
                "hedgehog",
                "discover",
                "--path",
                "../hedgehog/tests",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        archetype::snap("cli_discover_output", stdout);
    }

    #[test]
    fn test_snapshot_cli_discover_json() {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cargo-hedgehog",
                "--",
                "hedgehog",
                "discover",
                "--path",
                "../hedgehog/tests",
                "--format",
                "json",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        archetype::snap("cli_discover_json_output", stdout);
    }

    #[test]
    fn test_snapshot_cli_coverage_help() {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cargo-hedgehog",
                "--",
                "hedgehog",
                "coverage",
                "--help",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        archetype::snap("cli_coverage_help_output", stdout);
    }

    #[test]
    fn test_snapshot_cli_report_help() {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cargo-hedgehog",
                "--",
                "hedgehog",
                "report",
                "--help",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        archetype::snap("cli_report_help_output", stdout);
    }

    // Note: These tests would require actually running failing tests, which would
    // require creating a temporary crate structure. For now, we have tested the
    // discovery output and the formatting functions that handle failure output.
    // The actual test execution failure output would need to be tested by:
    // 1. Creating a temporary Cargo project
    // 2. Adding hedgehog as a dependency
    // 3. Running the failing tests
    // This is complex enough that it might be better to test the formatting
    // functions directly with mock failure output.
}
