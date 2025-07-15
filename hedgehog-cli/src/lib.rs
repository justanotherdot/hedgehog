use colored::*;

pub mod discovery;

/// Enhanced test output formatting for better readability
pub fn enhance_test_output(stdout: &str, stderr: &str) -> String {
    let mut enhanced = String::new();

    // Process stdout for property test failures
    for line in stdout.lines() {
        if line.contains("forAll") {
            enhanced.push_str(&format!("SEARCH: {}\n", line.bright_yellow()));
        } else if line.contains("Property failed") {
            enhanced.push_str(&format!("FAIL: {}\n", line.bright_red()));
        } else if line.contains("Counterexample") {
            enhanced.push_str(&format!("COUNTER: {}\n", line.bright_cyan()));
        } else if line.contains("Shrinks performed") {
            enhanced.push_str(&format!("SHRINK: {}\n", line.bright_magenta()));
        } else if line.contains("✓") {
            enhanced.push_str(&format!("PASS: {}\n", line.bright_green()));
        } else {
            enhanced.push_str(&format!("{}\n", line));
        }
    }

    // Add stderr if present
    if !stderr.is_empty() {
        enhanced.push_str("\nCompilation errors:\n");
        enhanced.push_str(&stderr.bright_red().to_string());
    }

    enhanced
}

/// Format test configuration output
pub fn format_config_output(derive: bool, release: bool, pattern: Option<&str>) -> String {
    let mut output = String::new();
    output.push_str("Configuration:\n");
    output.push_str(&format!(
        "  Derive feature: {}\n",
        if derive {
            "enabled".bright_green()
        } else {
            "disabled".bright_red()
        }
    ));
    output.push_str(&format!(
        "  Release mode: {}\n",
        if release {
            "enabled".bright_green()
        } else {
            "disabled".bright_red()
        }
    ));
    if let Some(pattern) = pattern {
        output.push_str(&format!("  Test pattern: {}\n", pattern.bright_yellow()));
    }
    output
}

/// Format header output
pub fn format_header() -> String {
    format!(
        "{}\n{}\n\n",
        "Hedgehog Property Testing".bright_green().bold(),
        "=========================".bright_green()
    )
}

/// Format success message
pub fn format_success() -> String {
    format!("{}\n", "All property tests passed!".bright_green().bold())
}

/// Format failure message
pub fn format_failure() -> String {
    format!("{}\n", "Some property tests failed!".bright_red().bold())
}

/// Format report generation message
pub fn format_report_message() -> String {
    format!("{}\n", "Generating test report...".bright_blue().bold())
}

/// Format coverage header
pub fn format_coverage_header() -> String {
    format!(
        "{}\n{}\n\n",
        "Property Test Coverage".bright_blue().bold(),
        "======================".bright_blue()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhance_test_output_empty() {
        let result = enhance_test_output("", "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_enhance_test_output_forall() {
        let stdout = "forAll 0 = 42";
        let result = enhance_test_output(stdout, "");
        assert!(result.contains("SEARCH:"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_enhance_test_output_property_failed() {
        let stdout = "Property failed with counterexample";
        let result = enhance_test_output(stdout, "");
        assert!(result.contains("FAIL:"));
        assert!(result.contains("Property failed"));
    }

    #[test]
    fn test_enhance_test_output_counterexample() {
        let stdout = "Counterexample: 42";
        let result = enhance_test_output(stdout, "");
        assert!(result.contains("COUNTER:"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_enhance_test_output_shrinks() {
        let stdout = "Shrinks performed: 5";
        let result = enhance_test_output(stdout, "");
        assert!(result.contains("SHRINK:"));
        assert!(result.contains("5"));
    }

    #[test]
    fn test_enhance_test_output_success() {
        let stdout = "✓ Property passed";
        let result = enhance_test_output(stdout, "");
        assert!(result.contains("PASS:"));
        assert!(result.contains("Property passed"));
    }

    #[test]
    fn test_enhance_test_output_with_stderr() {
        let stdout = "test output";
        let stderr = "compilation error";
        let result = enhance_test_output(stdout, stderr);
        assert!(result.contains("test output"));
        assert!(result.contains("Compilation errors:"));
        assert!(result.contains("compilation error"));
    }

    #[test]
    fn test_format_config_output_basic() {
        let result = format_config_output(false, false, None);
        assert!(result.contains("Configuration:"));
        assert!(result.contains("disabled")); // Check for the text, not the colored version
    }

    #[test]
    fn test_format_config_output_with_pattern() {
        let result = format_config_output(true, true, Some("test_pattern"));
        assert!(result.contains("enabled")); // Check for the text, not the colored version
        assert!(result.contains("test_pattern")); // Check for the text, not the colored version
    }

    #[test]
    fn test_format_header() {
        let result = format_header();
        assert!(result.contains("Hedgehog Property Testing"));
        assert!(result.contains("========================="));
    }

    #[test]
    fn test_format_success() {
        let result = format_success();
        assert!(result.contains("All property tests passed!"));
    }

    #[test]
    fn test_format_failure() {
        let result = format_failure();
        assert!(result.contains("Some property tests failed!"));
    }

    #[test]
    fn test_format_report_message() {
        let result = format_report_message();
        assert!(result.contains("Generating test report..."));
    }

    #[test]
    fn test_format_coverage_header() {
        let result = format_coverage_header();
        assert!(result.contains("Property Test Coverage"));
        assert!(result.contains("======================"));
    }

    // Snapshot tests for CLI output formatting
    #[test]
    fn test_snapshot_enhance_test_output_complex() {
        let stdout = "forAll 0 = 42\nProperty failed with counterexample\nCounterexample: 42\nShrinks performed: 3\n✓ Property passed";
        let stderr = "warning: unused variable";
        let result = enhance_test_output(stdout, stderr);
        archetype::snap("cli_enhance_test_output_complex", result);
    }

    #[test]
    fn test_snapshot_format_config_output_full() {
        let result = format_config_output(true, true, Some("integration_test"));
        archetype::snap("cli_format_config_output_full", result);
    }

    #[test]
    fn test_snapshot_format_config_output_minimal() {
        let result = format_config_output(false, false, None);
        archetype::snap("cli_format_config_output_minimal", result);
    }

    #[test]
    fn test_snapshot_format_header() {
        let result = format_header();
        archetype::snap("cli_format_header", result);
    }

    #[test]
    fn test_snapshot_format_success() {
        let result = format_success();
        archetype::snap("cli_format_success", result);
    }

    #[test]
    fn test_snapshot_format_failure() {
        let result = format_failure();
        archetype::snap("cli_format_failure", result);
    }

    #[test]
    fn test_snapshot_format_report_message() {
        let result = format_report_message();
        archetype::snap("cli_format_report_message", result);
    }

    #[test]
    fn test_snapshot_format_coverage_header() {
        let result = format_coverage_header();
        archetype::snap("cli_format_coverage_header", result);
    }

    #[test]
    fn test_snapshot_enhance_test_output_property_failure() {
        let stdout = "Property failed with counterexample\nCounterexample: (0, \"test\")\nShrinks performed: 15\nMinimal counterexample found";
        let result = enhance_test_output(stdout, "");
        archetype::snap("cli_enhance_test_output_property_failure", result);
    }

    #[test]
    fn test_snapshot_enhance_test_output_compilation_error() {
        let stdout = "running 1 test";
        let stderr = "error[E0425]: cannot find value `undefined_var` in this scope\n  --> src/lib.rs:10:5\n   |\n10 |     undefined_var\n   |     ^^^^^^^^^^^^^ not found in this scope";
        let result = enhance_test_output(stdout, stderr);
        archetype::snap("cli_enhance_test_output_compilation_error", result);
    }

    #[test]
    fn test_snapshot_enhance_test_output_mixed_results() {
        let stdout = "✓ Property passed\nforAll 1 = 100\nProperty failed with counterexample\n✓ Another property passed\nCounterexample: -5\nShrinks performed: 8";
        let result = enhance_test_output(stdout, "");
        archetype::snap("cli_enhance_test_output_mixed_results", result);
    }
}
