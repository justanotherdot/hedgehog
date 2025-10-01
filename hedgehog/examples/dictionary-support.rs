//! Dictionary support demonstration
//!
//! This example shows how to use dictionary-based generators to inject
//! domain-specific realistic values into property-based tests.

use hedgehog::*;

fn main() {
    println!("=== Dictionary Support Demonstration ===\n");

    // Example 1: Basic element selection
    example_basic_elements();

    // Example 2: Dictionary mixing with random generation
    example_dictionary_mixing();

    // Example 3: Web domain generation
    example_web_domains();

    // Example 4: HTTP status code testing
    example_http_testing();

    // Example 5: Database testing with SQL identifiers
    example_database_testing();

    // Example 6: Network service testing
    example_network_testing();
}

/// Example 1: Basic element selection from predefined lists
fn example_basic_elements() {
    println!("1. Basic element selection from predefined lists");

    // Test a function that validates user roles
    fn is_valid_role(role: &str) -> bool {
        matches!(role, "admin" | "user" | "guest" | "moderator")
    }

    let valid_roles = vec![
        "admin".to_string(),
        "user".to_string(),
        "guest".to_string(),
        "moderator".to_string(),
    ];

    let prop = for_all(Gen::from_elements(valid_roles).unwrap(), |role| {
        is_valid_role(role)
    });

    match prop.run(&Config::default().with_tests(20)) {
        TestResult::Pass { tests_run, .. } => {
            println!("   ✓ All {tests_run} generated roles were valid");
        }
        TestResult::Fail { counterexample, .. } => {
            println!("   ✗ Invalid role found: {counterexample}");
        }
        _ => {}
    }
    println!();
}

/// Example 2: Dictionary mixing with random generation
fn example_dictionary_mixing() {
    println!("2. Dictionary mixing with random generation");

    // Test a port validation function
    fn categorize_port(port: u16) -> &'static str {
        match port {
            1..=1023 => "well-known",
            1024..=49151 => "registered",
            49152..=65535 => "dynamic",
            _ => "invalid",
        }
    }

    let well_known_ports = vec![22, 80, 443, 53, 25, 110, 143, 993];

    // Mix well-known ports (70%) with random ports (30%)
    let port_gen = Gen::from_dictionary(
        well_known_ports,
        Gen::int_range(1024, 65535).map(|i| i as u16),
        70, // 70% well-known ports
        30, // 30% random ports
    )
    .unwrap();

    let mut well_known_count = 0;
    let mut registered_count = 0;
    let mut dynamic_count = 0;

    for _ in 0..100 {
        let tree = port_gen.generate(Size::new(10), Seed::random());
        match categorize_port(tree.value) {
            "well-known" => well_known_count += 1,
            "registered" => registered_count += 1,
            "dynamic" => dynamic_count += 1,
            _ => {}
        }
    }

    println!("   Port distribution over 100 generations:");
    println!("     Well-known: {well_known_count} (expected ~70%)");
    println!("     Registered: {registered_count} (expected ~20-25%)");
    println!("     Dynamic: {dynamic_count} (expected ~5-10%)");
    println!();
}

/// Example 3: Web domain generation for URL testing
fn example_web_domains() {
    println!("3. Web domain generation for URL testing");

    // Test a URL validation function
    fn is_plausible_url(domain: &str) -> bool {
        domain.contains('.')
            && domain.len() > 4
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && domain
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.')
    }

    let prop = for_all(Gen::<String>::web_domain(), |domain| {
        is_plausible_url(domain)
    });

    match prop.run(&Config::default().with_tests(30)) {
        TestResult::Pass { tests_run, .. } => {
            println!("   ✓ All {tests_run} generated domains passed validation");

            // Show some example domains
            println!("   Example domains generated:");
            for _ in 0..5 {
                let tree = Gen::<String>::web_domain().generate(Size::new(10), Seed::random());
                println!("     - {}", tree.value);
            }
        }
        TestResult::Fail { counterexample, .. } => {
            println!("   ✗ Invalid domain: {counterexample}");
        }
        _ => {}
    }
    println!();
}

/// Example 4: HTTP status code testing for web services
fn example_http_testing() {
    println!("4. HTTP status code testing for web services");

    // Test a function that categorizes HTTP responses
    fn response_category(status: u16) -> &'static str {
        match status {
            100..=199 => "informational",
            200..=299 => "success",
            300..=399 => "redirection",
            400..=499 => "client_error",
            500..=599 => "server_error",
            _ => "invalid",
        }
    }

    let prop = for_all(Gen::<u16>::http_status_code(), |&status| {
        let category = response_category(status);
        category != "invalid"
    })
    .classify("2xx_success", |&status| (200..=299).contains(&status))
    .classify("4xx_client_error", |&status| (400..=499).contains(&status))
    .classify("5xx_server_error", |&status| (500..=599).contains(&status))
    .collect("status_code", |&status| status as f64);

    match prop.run(&Config::default().with_tests(100)) {
        TestResult::PassWithStatistics {
            tests_run,
            statistics,
            ..
        } => {
            println!("   ✓ All {tests_run} status codes were valid HTTP codes");
            println!("   Status code distribution:");

            for (category, count) in &statistics.classifications {
                let percentage = (*count as f64 / tests_run as f64) * 100.0;
                println!("     {category}: {percentage:.1}% ({count} codes)");
            }

            if let Some(codes) = statistics.collections.get("status_code") {
                let avg = codes.iter().sum::<f64>() / codes.len() as f64;
                let min = codes.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = codes.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                println!("     Range: {min:.0}-{max:.0}, Average: {avg:.1}");
            }
        }
        TestResult::Fail { counterexample, .. } => {
            println!("   ✗ Invalid status code: {counterexample}");
        }
        _ => {}
    }
    println!();
}

/// Example 5: Database testing with SQL identifiers
fn example_database_testing() {
    println!("5. Database testing with SQL identifiers");

    // Test a SQL query builder that should handle identifiers safely
    fn build_select_query(table: &str, column: &str) -> std::result::Result<String, &'static str> {
        if table.is_empty() || column.is_empty() {
            return Err("Empty identifiers not allowed");
        }

        // Simple check - in reality you'd have more sophisticated validation
        if table.len() > 100 || column.len() > 100 {
            return Err("Identifier too long");
        }

        Ok(format!("SELECT {column} FROM {table}"))
    }

    let table_gen = Gen::<String>::sql_identifier(false); // Safe identifiers only
    let column_gen = Gen::<String>::sql_identifier(false);

    let prop = for_all(
        Gen::<(String, String)>::tuple_of(table_gen, column_gen),
        |(table, column)| {
            match build_select_query(table, column) {
                Ok(query) => {
                    // Query should contain both identifiers
                    query.contains(table) && query.contains(column)
                }
                Err(_) => false, // Should not error with safe identifiers
            }
        },
    );

    match prop.run(&Config::default().with_tests(50)) {
        TestResult::Pass { tests_run, .. } => {
            println!("   ✓ All {tests_run} SQL queries built successfully");

            // Show example queries
            println!("   Example queries generated:");
            for _ in 0..3 {
                let table_tree =
                    Gen::<String>::sql_identifier(false).generate(Size::new(10), Seed::random());
                let column_tree =
                    Gen::<String>::sql_identifier(false).generate(Size::new(10), Seed::random());
                let query = build_select_query(&table_tree.value, &column_tree.value).unwrap();
                println!("     {query}");
            }
        }
        TestResult::Fail { counterexample, .. } => {
            println!("   ✗ Query building failed for: {counterexample}");
        }
        _ => {}
    }
    println!();
}

/// Example 6: Network service testing
fn example_network_testing() {
    println!("6. Network service testing with realistic ports");

    // Test a service configuration validator
    fn validate_service_config(name: &str, port: u16) -> std::result::Result<String, &'static str> {
        if name.is_empty() {
            return Err("Service name cannot be empty");
        }

        if port == 0 {
            return Err("Port cannot be zero");
        }

        // Check for common conflicts
        match port {
            22 => {
                if name != "ssh" {
                    return Err("Port 22 reserved for SSH");
                }
            }
            80 => {
                if name != "http" {
                    return Err("Port 80 reserved for HTTP");
                }
            }
            443 => {
                if name != "https" {
                    return Err("Port 443 reserved for HTTPS");
                }
            }
            _ => {}
        }

        Ok(format!("{name}:{port}"))
    }

    let service_names = vec![
        "ssh".to_string(),
        "http".to_string(),
        "https".to_string(),
        "api".to_string(),
        "web".to_string(),
        "app".to_string(),
    ];

    let name_gen = Gen::from_elements(service_names).unwrap();
    let port_gen = Gen::<u16>::network_port();

    let prop = for_all(
        Gen::<(String, u16)>::tuple_of(name_gen, port_gen),
        |(name, port)| {
            match validate_service_config(name, *port) {
                Ok(config) => {
                    // Should contain the service name and port
                    config.contains(name) && config.contains(&port.to_string())
                }
                Err(_) => {
                    // Some combinations are expected to fail (like wrong service on reserved port)
                    true
                }
            }
        },
    )
    .classify("well_known_port", |(_, port)| *port <= 1023)
    .classify("registered_port", |(_, port)| {
        *port >= 1024 && *port <= 49151
    })
    .classify("dynamic_port", |(_, port)| *port >= 49152);

    match prop.run(&Config::default().with_tests(100)) {
        TestResult::PassWithStatistics {
            tests_run,
            statistics,
            ..
        } => {
            println!("   ✓ All {tests_run} service configurations handled correctly");
            println!("   Port type distribution:");

            for (port_type, count) in &statistics.classifications {
                let percentage = (*count as f64 / tests_run as f64) * 100.0;
                println!("     {port_type}: {percentage:.1}% ({count} configs)");
            }

            // Show some example valid configurations
            println!("   Example valid configurations:");
            for _ in 0..3 {
                let name = "api".to_string();
                let port_tree = Gen::<u16>::network_port().generate(Size::new(10), Seed::random());
                match validate_service_config(&name, port_tree.value) {
                    Ok(config) => println!("     ✓ {config}"),
                    Err(reason) => println!("     ✗ {}:{} - {}", name, port_tree.value, reason),
                }
            }
        }
        TestResult::Fail { counterexample, .. } => {
            println!("   ✗ Service validation failed for: {counterexample}");
        }
        _ => {}
    }

    println!("\n=== Dictionary Support Complete ===");
    println!("Dictionary support enables realistic domain-specific testing by mixing");
    println!("predefined meaningful values with random generation for comprehensive coverage.");
}
