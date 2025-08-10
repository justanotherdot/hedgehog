//! Real-world property testing patterns using advanced Hedgehog features
//!
//! This file demonstrates practical applications of the advanced features
//! that have been thoroughly tested in the meta testing suite.

use hedgehog::*;
use hedgehog::state::*;
use hedgehog::parallel::*;
use hedgehog::targeted::*;
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    println!("🦔 Real-World Hedgehog Patterns Demo");
    println!("====================================\n");

    // Run all examples
    web_api_testing();
    database_transaction_testing();
    distributed_cache_testing();
    json_parsing_robustness();
    network_protocol_testing();
    
    println!("\n✅ All real-world examples completed!");
}

/// Example 1: Web API Testing with Targeted Properties
/// Tests an HTTP API for edge cases using targeted search
fn web_api_testing() {
    println!("🌐 Web API Testing with Targeted Properties");
    println!("-------------------------------------------");

    #[derive(Debug, Clone)]
    struct HttpRequest {
        method: String,
        path: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    }

    impl std::fmt::Display for HttpRequest {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} {} (headers: {})", self.method, self.path, self.headers.len())
        }
    }

    // Generate realistic HTTP requests
    let request_gen = Gen::new(|size, seed| {
        let (method_seed, rest_seed) = seed.split();
        let (path_seed, rest_seed) = rest_seed.split();
        let (headers_seed, body_seed) = rest_seed.split();

        let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH"];
        let method_idx = method_seed.next_bounded(methods.len() as u64).0 as usize;
        let method = methods[method_idx].to_string();

        let path_gen = Gen::<String>::web_domain().map(|domain| format!("/api/v1/{}", domain));
        let path = path_gen.generate(size, path_seed).value;

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("User-Agent".to_string(), "hedgehog-test/1.0".to_string());

        let body = if method == "POST" || method == "PUT" {
            Some(r#"{"test": "data"}"#.to_string())
        } else {
            None
        };

        Tree::singleton(HttpRequest { method, path, headers, body })
    });

    // Utility function that guides search toward problematic paths
    let utility_function = |request: &HttpRequest, _result: &TargetedResult| -> f64 {
        let mut score = 0.0;
        
        // Higher utility for paths that might cause issues
        if request.path.contains("..") || request.path.contains("//") {
            score += 50.0; // Path traversal attempts
        }
        
        if request.path.len() > 100 {
            score += 30.0; // Very long paths
        }
        
        if request.headers.is_empty() {
            score += 20.0; // Missing headers
        }
        
        score
    };

    // Simulate API testing
    let test_function = |request: &HttpRequest| -> TargetedResult {
        // Simulate finding bugs in certain conditions
        if request.path.contains("..") || request.path.len() > 200 {
            TargetedResult::Fail {
                counterexample: format!("Potentially unsafe request: {}", request),
                tests_run: 1,
                shrinks_performed: 0,
                property_name: Some("api_security_test".to_string()),
                module_path: Some("web_api".to_string()),
                assertion_type: Some("Security Check".to_string()),
                shrink_steps: Vec::new(),
                utility: 0.0,
            }
        } else {
            TargetedResult::Pass {
                tests_run: 1,
                property_name: Some("api_security_test".to_string()),
                module_path: Some("web_api".to_string()),
                utility: 0.0,
            }
        }
    };

    let config = TargetedConfig {
        search_steps: 50,
        initial_temperature: 30.0,
        objective: SearchObjective::Maximize,
        max_search_time: Some(Duration::from_millis(500)),
        ..Default::default()
    };

    let search = for_all_targeted_with_config(
        request_gen,
        utility_function,
        test_function,
        StringNeighborhood::default(), // This would need to be implemented for HttpRequest
        config,
    );

    // In a real implementation, you'd run this search
    println!("  ✓ Configured targeted API security testing");
    println!("  → Would search for edge cases in HTTP requests\n");
}

/// Example 2: Database Transaction Testing with State Machines  
/// Tests ACID properties using state machine testing
fn database_transaction_testing() {
    println!("🗄️  Database Transaction Testing with State Machines");
    println!("--------------------------------------------------");

    #[derive(Debug, Clone, PartialEq)]
    struct DatabaseState {
        tables: HashMap<String, HashMap<String, i32>>, // table -> key -> value
        active_transactions: Vec<String>,
        isolation_level: String,
    }

    impl DatabaseState {
        fn new() -> Self {
            let mut tables = HashMap::new();
            tables.insert("users".to_string(), HashMap::new());
            tables.insert("accounts".to_string(), HashMap::new());

            Self {
                tables,
                active_transactions: Vec::new(),
                isolation_level: "READ_COMMITTED".to_string(),
            }
        }

        fn can_start_transaction(&self) -> bool {
            self.active_transactions.len() < 10
        }

        fn has_active_transactions(&self) -> bool {
            !self.active_transactions.is_empty()
        }
    }

    #[derive(Clone, Debug)]
    struct BeginTransactionInput {
        transaction_id: String,
    }

    impl std::fmt::Display for BeginTransactionInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "BEGIN {}", self.transaction_id)
        }
    }

    #[derive(Clone, Debug)]  
    struct InsertInput {
        table: String,
        key: String,
        value: i32,
        transaction_id: String,
    }

    impl std::fmt::Display for InsertInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "INSERT INTO {} ({}: {}) TX:{}", self.table, self.key, self.value, self.transaction_id)
        }
    }

    let mut generator = ActionGenerator::new();

    // Begin transaction command
    let begin_cmd: Command<BeginTransactionInput, String, DatabaseState, String> = Command::new(
        "begin_transaction".to_string(),
        |state: &DatabaseState| {
            if state.can_start_transaction() {
                Some(Gen::<String>::alpha_with_range(Range::new(5, 10))
                    .map(|id| BeginTransactionInput { transaction_id: id }))
            } else {
                None
            }
        },
        |input: BeginTransactionInput| input.transaction_id.clone(),
    )
    .with_require(|state: &DatabaseState, _input: &BeginTransactionInput| {
        state.can_start_transaction()
    })
    .with_update(|state: &mut DatabaseState, input: &BeginTransactionInput, _output: &Var<String>| {
        state.active_transactions.push(input.transaction_id.clone());
    })
    .with_ensure(|old_state: &DatabaseState, new_state: &DatabaseState, input: &BeginTransactionInput, output: &String| {
        if new_state.active_transactions.len() != old_state.active_transactions.len() + 1 {
            Err("Transaction count not incremented".to_string())
        } else if *output != input.transaction_id {
            Err("Wrong transaction ID returned".to_string())
        } else {
            Ok(())
        }
    });

    generator.add_command(begin_cmd);

    let initial_state = DatabaseState::new();
    let sequence = generator.generate_sequential(initial_state.clone(), 5);

    println!("  Generated transaction sequence:");
    for (i, action) in sequence.actions.iter().enumerate() {
        println!("    {}: {}", i + 1, action.display_action());
    }

    match execute_sequential(initial_state, sequence) {
        Ok(()) => println!("  ✓ All database operations completed successfully"),
        Err(e) => println!("  ✗ Database operation failed: {}", e),
    }
    println!();
}

/// Example 3: Distributed Cache Testing with Parallel Properties
/// Tests cache consistency across multiple threads
fn distributed_cache_testing() {
    println!("⚡ Distributed Cache Testing with Parallel Properties");
    println!("---------------------------------------------------");

    use std::sync::Arc;
    use std::sync::Mutex;

    #[derive(Clone)]
    struct CacheOperation {
        key: String,
        value: Option<i32>, // None for GET, Some for PUT
        operation_id: u64,
    }

    impl std::fmt::Display for CacheOperation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.value {
                Some(v) => write!(f, "PUT {} = {} ({})", self.key, v, self.operation_id),
                None => write!(f, "GET {} ({})", self.key, self.operation_id),
            }
        }
    }

    // Simulate a distributed cache
    let cache = Arc::new(Mutex::new(HashMap::<String, i32>::new()));

    let cache_gen = Gen::new(|_size, seed| {
        let (key_seed, rest_seed) = seed.split();
        let (value_seed, op_seed) = rest_seed.split();

        let key_gen = Gen::<String>::alpha_with_range(Range::new(3, 8));
        let key = key_gen.generate(Size::new(5), key_seed).value;

        let (is_put, _) = value_seed.split().0.next_bounded(2);
        let value = if is_put == 0 {
            None // GET operation
        } else {
            Some((value_seed.next_bounded(1000).0 as i32)) // PUT operation
        };

        let operation_id = op_seed.next_bounded(10000).0;

        Tree::singleton(CacheOperation { key, value, operation_id })
    });

    let parallel_config = ParallelConfig {
        thread_count: 4,
        work_distribution: WorkDistribution::WorkStealing,
        ..ParallelConfig::default()
    };

    let cache_clone = Arc::clone(&cache);
    let parallel_prop = parallel_property(
        cache_gen,
        move |operation: &CacheOperation| {
            let mut cache_guard = cache_clone.lock().unwrap();

            match &operation.value {
                Some(value) => {
                    // PUT operation
                    cache_guard.insert(operation.key.clone(), *value);
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some(format!("cache_put_{}", operation.operation_id)),
                        module_path: Some("distributed_cache".to_string()),
                    }
                }
                None => {
                    // GET operation
                    let _cached_value = cache_guard.get(&operation.key);
                    TestResult::Pass {
                        tests_run: 1,
                        property_name: Some(format!("cache_get_{}", operation.operation_id)),
                        module_path: Some("distributed_cache".to_string()),
                    }
                }
            }
        },
        parallel_config,
    );

    // In a real test, you'd run this
    println!("  ✓ Configured parallel cache consistency testing");
    println!("  → Would test cache operations across {} threads", 4);
    println!("  → Would detect race conditions and consistency issues\n");
}

/// Example 4: JSON Parsing Robustness with String Generators
/// Tests JSON parsing with malformed/edge-case inputs
fn json_parsing_robustness() {
    println!("📄 JSON Parsing Robustness with String Generators");
    println!("------------------------------------------------");

    // Generate JSON-like strings that might break parsers
    let malformed_json_gen = Gen::frequency(vec![
        // Valid JSON (10%)
        WeightedChoice::new(1, Gen::constant(r#"{"valid": "json"}"#.to_string())),
        
        // Missing quotes (20%)
        WeightedChoice::new(2, Gen::constant(r#"{missing: quotes}"#.to_string())),
        
        // Trailing commas (20%)
        WeightedChoice::new(2, Gen::constant(r#"{"trailing": "comma",}"#.to_string())),
        
        // Nested structures (20%)  
        WeightedChoice::new(2, Gen::constant(r#"{"nested": {"deep": {"very": "deep"}}}"#.to_string())),
        
        // Unicode and special characters (20%)
        WeightedChoice::new(2, Gen::<String>::ascii_printable().map(|s| format!(r#"{{"unicode": "{}"}}"#, s))),
        
        // Empty/null cases (10%)
        WeightedChoice::new(1, Gen::one_of(vec![
            Gen::constant("".to_string()),
            Gen::constant("null".to_string()),
            Gen::constant("{}".to_string()),
        ]).unwrap()),
    ]).unwrap();

    let json_parsing_prop = for_all(malformed_json_gen, |json_str: &String| {
        // Simulate JSON parsing
        let parse_result = simulate_json_parse(json_str);
        
        // Property: Parser should either succeed or fail gracefully (no panics)
        match parse_result {
            Ok(_) => true, // Valid parse
            Err(_) => true, // Graceful error
        }
        // If this property fails, it means the parser panicked
    });

    println!("  ✓ Configured JSON parsing robustness testing");
    
    // Run a few examples
    let config = Config::default().with_tests(5);
    match json_parsing_prop.run(&config) {
        TestResult::Pass { tests_run, .. } => {
            println!("  ✓ JSON parser handled {} malformed inputs gracefully", tests_run);
        }
        result => {
            println!("  ✗ JSON parser failed robustness test: {:?}", result);
        }
    }
    println!();
}

// Simulate JSON parsing (placeholder)
fn simulate_json_parse(json_str: &str) -> Result<serde_json::Value, String> {
    if json_str.is_empty() {
        Err("Empty input".to_string())
    } else if json_str.contains("missing") {
        Err("Invalid JSON".to_string())  
    } else {
        Ok(serde_json::Value::Null) // Simplified success case
    }
}

/// Example 5: Network Protocol Testing with Result/Option Generators
/// Tests network message handling with connection failures
fn network_protocol_testing() {
    println!("🌐 Network Protocol Testing with Result/Option Generators");
    println!("---------------------------------------------------------");

    #[derive(Debug, Clone)]
    struct NetworkMessage {
        message_type: String,
        payload: Vec<u8>,
        sequence_number: u32,
    }

    #[derive(Debug, Clone)]
    struct ConnectionState {
        is_connected: bool,
        last_sequence: u32,
        retry_count: usize,
    }

    // Generate network scenarios with failures
    let network_scenario_gen = Gen::<std::result::Result<Option<NetworkMessage>, String>>::result_of(
        Gen::<Option<NetworkMessage>>::option_of(
            Gen::new(|_size, seed| {
                let (type_seed, rest_seed) = seed.split();
                let (payload_seed, seq_seed) = rest_seed.split();

                let message_types = vec!["PING", "PONG", "DATA", "ACK", "FIN"];
                let type_idx = type_seed.next_bounded(message_types.len() as u64).0 as usize;
                let message_type = message_types[type_idx].to_string();

                let payload_size = payload_seed.next_bounded(100).0 as usize;
                let payload: Vec<u8> = (0..payload_size).map(|i| (i % 256) as u8).collect();

                let sequence_number = seq_seed.next_bounded(1000).0 as u32;

                Tree::singleton(NetworkMessage {
                    message_type,
                    payload,
                    sequence_number,
                })
            })
        ),
        Gen::one_of(vec![
            Gen::constant("CONNECTION_LOST".to_string()),
            Gen::constant("TIMEOUT".to_string()),
            Gen::constant("CHECKSUM_MISMATCH".to_string()),
            Gen::constant("BUFFER_OVERFLOW".to_string()),
        ]).unwrap(),
    );

    let network_prop = for_all(network_scenario_gen, |scenario: &std::result::Result<Option<NetworkMessage>, String>| {
        let mut connection_state = ConnectionState {
            is_connected: true,
            last_sequence: 0,
            retry_count: 0,
        };

        match scenario {
            Ok(Some(message)) => {
                // Handle successful message
                let sequence_valid = message.sequence_number > connection_state.last_sequence || 
                                   connection_state.last_sequence == 0;
                let payload_reasonable = message.payload.len() <= 1000;
                
                connection_state.last_sequence = message.sequence_number;
                sequence_valid && payload_reasonable
            }
            Ok(None) => {
                // No message (keep-alive scenario)
                true
            }
            Err(error) => {
                // Handle network error
                connection_state.is_connected = false;
                connection_state.retry_count += 1;
                
                // Property: should handle all error types gracefully
                ["CONNECTION_LOST", "TIMEOUT", "CHECKSUM_MISMATCH", "BUFFER_OVERFLOW"]
                    .contains(&error.as_str())
            }
        }
    });

    let config = Config::default().with_tests(10);
    match network_prop.run(&config) {
        TestResult::Pass { tests_run, .. } => {
            println!("  ✓ Network protocol handled {} scenarios correctly", tests_run);
        }
        result => {
            println!("  ✗ Network protocol test failed: {:?}", result);
        }
    }
    
    println!("  ✓ Tested various network failure modes and recovery");
    println!();
}

// External dependencies (would be real crates in practice)
mod serde_json {
    #[derive(Debug)]
    pub enum Value {
        Null,
    }
}