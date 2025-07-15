use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "cargo-hedgehog", "--", "hedgehog", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Run Hedgehog property tests"));
    assert!(stdout.contains("test"));
    assert!(stdout.contains("report"));
    assert!(stdout.contains("coverage"));
}

#[test]
fn test_cli_test_help() {
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
    assert!(stdout.contains("Run property tests"));
    assert!(stdout.contains("--verbose"));
    assert!(stdout.contains("--derive"));
    assert!(stdout.contains("--release"));
}

#[test]
fn test_cli_report_help() {
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
    assert!(stdout.contains("Generate test report"));
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("json"));
    assert!(stdout.contains("markdown"));
    assert!(stdout.contains("html"));
}

#[test]
fn test_cli_coverage_help() {
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
    assert!(stdout.contains("Show test coverage"));
    assert!(stdout.contains("--detailed"));
}

#[test]
fn test_cli_report_json() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cargo-hedgehog",
            "--",
            "hedgehog",
            "report",
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generating test report"));
    assert!(stdout.contains("hedgehog-test-report"));
    assert!(stdout.contains("timestamp"));
    assert!(stdout.contains("properties"));
    assert!(stdout.contains("Report generated successfully"));
}

#[test]
fn test_cli_coverage_detailed() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "cargo-hedgehog",
            "--",
            "hedgehog",
            "coverage",
            "--detailed",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Property Test Coverage"));
    assert!(stdout.contains("Coverage analysis"));
    assert!(stdout.contains("properties found"));
    assert!(stdout.contains("Detailed coverage"));
    assert!(stdout.contains("Covered"));
}
