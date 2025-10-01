//! Real-world state machine testing examples.
//!
//! This module demonstrates practical applications of state machine testing
//! for various common scenarios that developers encounter.

use hedgehog_core::gen::Gen;
use hedgehog_core::state::*;
use hedgehog_core::tree::Tree;
use std::collections::HashMap;

/// Example 1: File System Operations
///
/// This example tests file system operations like creating directories,
/// writing files, reading files, and deleting them. Common bugs this
/// can catch include:
/// - Race conditions in file operations
/// - Incorrect state tracking after operations
/// - Resource leaks (files not properly closed)
/// - Permission handling errors
pub mod filesystem {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub struct FileSystem {
        pub directories: std::collections::HashSet<String>,
        pub files: HashMap<String, String>, // path -> content
        pub current_dir: String,
    }

    impl Default for FileSystem {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FileSystem {
        pub fn new() -> Self {
            let mut dirs = std::collections::HashSet::new();
            dirs.insert("/".to_string());
            Self {
                directories: dirs,
                files: HashMap::new(),
                current_dir: "/".to_string(),
            }
        }

        pub fn path_exists(&self, path: &str) -> bool {
            self.directories.contains(path) || self.files.contains_key(path)
        }
    }

    #[derive(Clone, Debug)]
    pub struct CreateDirInput {
        pub path: String,
    }

    impl std::fmt::Display for CreateDirInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "\"{}\"", self.path)
        }
    }

    #[derive(Clone, Debug)]
    pub struct WriteFileInput {
        pub path: String,
        pub content: String,
    }

    impl std::fmt::Display for WriteFileInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "\"{}\" with \"{}\"",
                self.path,
                self.content.chars().take(10).collect::<String>()
            )
        }
    }

    pub fn filesystem_commands() -> Vec<()> {
        let mut generator = ActionGenerator::new();

        // Create directory command
        let mkdir_cmd: Command<CreateDirInput, bool, FileSystem, bool> = Command::new(
            "mkdir".to_string(),
            |_state: &FileSystem| {
                Some(Gen::new(|_size, seed| {
                    let paths = ["/tmp", "/home", "/var", "/etc", "/usr"];
                    let (idx, _) = seed.next_bounded(paths.len() as u64);
                    Tree::singleton(CreateDirInput {
                        path: paths[idx as usize].to_string(),
                    })
                }))
            },
            |input: CreateDirInput| !input.path.is_empty(),
        )
        .with_require(|state: &FileSystem, input: &CreateDirInput| {
            !input.path.is_empty() && !state.path_exists(&input.path)
        })
        .with_update(
            |state: &mut FileSystem, input: &CreateDirInput, _output: &Var<bool>| {
                state.directories.insert(input.path.clone());
            },
        )
        .with_ensure(
            |_old_state: &FileSystem,
             new_state: &FileSystem,
             input: &CreateDirInput,
             output: &bool| {
                if !new_state.directories.contains(&input.path) {
                    Err(format!("Directory {} was not created", input.path))
                } else if !output {
                    Err("mkdir should return true on success".to_string())
                } else {
                    Ok(())
                }
            },
        );

        // Write file command
        let write_cmd: Command<WriteFileInput, usize, FileSystem, usize> = Command::new(
            "write_file".to_string(),
            |_state: &FileSystem| {
                Some(Gen::new(|_size, seed| {
                    let (path_idx, seed2) = seed.next_bounded(5);
                    let (content_len, _) = seed2.next_bounded(20);
                    let path = format!("/file{path_idx}.txt");
                    let content = "x".repeat(content_len as usize + 1);
                    Tree::singleton(WriteFileInput { path, content })
                }))
            },
            |input: WriteFileInput| input.content.len(),
        )
        .with_require(|_state: &FileSystem, input: &WriteFileInput| {
            !input.path.is_empty() && !input.content.is_empty()
        })
        .with_update(
            |state: &mut FileSystem, input: &WriteFileInput, _output: &Var<usize>| {
                state
                    .files
                    .insert(input.path.clone(), input.content.clone());
            },
        )
        .with_ensure(
            |_old_state: &FileSystem,
             new_state: &FileSystem,
             input: &WriteFileInput,
             output: &usize| {
                if let Some(stored_content) = new_state.files.get(&input.path) {
                    if stored_content != &input.content {
                        Err("File content mismatch".to_string())
                    } else if *output != input.content.len() {
                        Err("Incorrect bytes written count".to_string())
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("File {} was not created", input.path))
                }
            },
        );

        generator.add_command(mkdir_cmd);
        generator.add_command(write_cmd);

        // This is a bit of a hack to return the internal commands
        // In a real implementation, we'd want a better API design
        vec![] // Placeholder - this example shows the pattern
    }
}

/// Example 2: HTTP Client Session Management
///
/// This example tests HTTP client behavior including connection pooling,
/// authentication, request/response cycles, and cleanup. Bugs this can catch:
/// - Connection leaks
/// - Authentication state issues  
/// - Request/response mismatches
/// - Timeout handling problems
pub mod http_client {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub struct HttpClient {
        pub connections: HashMap<String, bool>, // host -> is_connected
        pub auth_tokens: HashMap<String, String>, // host -> token
        pub request_count: usize,
        pub pool_size: usize,
        pub max_connections: usize,
    }

    impl Default for HttpClient {
        fn default() -> Self {
            Self::new()
        }
    }

    impl HttpClient {
        pub fn new() -> Self {
            Self {
                connections: HashMap::new(),
                auth_tokens: HashMap::new(),
                request_count: 0,
                pool_size: 0,
                max_connections: 10,
            }
        }

        pub fn is_connected(&self, host: &str) -> bool {
            self.connections.get(host).copied().unwrap_or(false)
        }
    }

    #[derive(Clone, Debug)]
    pub struct ConnectInput {
        pub host: String,
    }

    impl std::fmt::Display for ConnectInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.host)
        }
    }

    #[derive(Clone, Debug)]
    pub struct RequestInput {
        pub host: String,
        pub path: String,
        pub method: String,
    }

    impl std::fmt::Display for RequestInput {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} {}{}", self.method, self.host, self.path)
        }
    }

    // Example command implementations would go here...
    // Similar pattern to filesystem example
}

/// Example 3: Cache Management
///
/// This example tests cache operations including setting values, getting values,
/// expiration, eviction policies, and memory management. Bugs this can catch:
/// - Memory leaks from items not being evicted
/// - Incorrect expiration handling
/// - Race conditions in cache access
/// - Inconsistent state after eviction
pub mod cache {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone, PartialEq)]
    pub struct CacheEntry {
        pub value: String,
        pub expires_at: Option<u64>, // Unix timestamp
        pub access_count: usize,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Cache {
        pub entries: HashMap<String, CacheEntry>,
        pub max_size: usize,
        pub current_time: u64, // Simulated time for testing
    }

    impl Cache {
        pub fn new(max_size: usize) -> Self {
            Self {
                entries: HashMap::new(),
                max_size,
                current_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }
        }

        pub fn is_expired(&self, key: &str) -> bool {
            if let Some(entry) = self.entries.get(key) {
                if let Some(expires_at) = entry.expires_at {
                    return self.current_time >= expires_at;
                }
            }
            false
        }

        pub fn should_evict(&self) -> bool {
            self.entries.len() >= self.max_size
        }
    }

    // Command implementations would follow similar patterns...
}

/// Example 4: Database Transaction Management
///
/// This example tests database transaction behavior including begin/commit/rollback,
/// isolation levels, concurrent access, and data consistency. Bugs this can catch:
/// - Transaction isolation violations  
/// - Deadlocks and race conditions
/// - Incorrect rollback behavior
/// - Data corruption from incomplete transactions
pub mod database {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub enum TransactionState {
        None,
        Active,
        Committed,
        RolledBack,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Database {
        pub tables: HashMap<String, HashMap<String, String>>, // table -> key -> value
        pub transaction_state: TransactionState,
        pub transaction_log: Vec<String>, // Operations in current transaction
        pub connection_count: usize,
    }

    impl Default for Database {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Database {
        pub fn new() -> Self {
            Self {
                tables: HashMap::new(),
                transaction_state: TransactionState::None,
                transaction_log: Vec::new(),
                connection_count: 0,
            }
        }

        pub fn in_transaction(&self) -> bool {
            matches!(self.transaction_state, TransactionState::Active)
        }
    }

    // Transaction and query command implementations...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_example() {
        // This demonstrates how to use the filesystem state machine
        let initial_state = filesystem::FileSystem::new();

        // In a real test, we'd set up the commands and run them
        assert_eq!(initial_state.current_dir, "/");
        assert!(initial_state.directories.contains("/"));

        println!("✓ Filesystem state machine example structure created");
    }

    #[test]
    fn test_cache_example() {
        let cache = cache::Cache::new(100);

        assert_eq!(cache.max_size, 100);
        assert!(cache.entries.is_empty());

        println!("✓ Cache state machine example structure created");
    }

    #[test]
    fn test_http_client_example() {
        let client = http_client::HttpClient::new();

        assert_eq!(client.max_connections, 10);
        assert_eq!(client.request_count, 0);

        println!("✓ HTTP client state machine example structure created");
    }

    #[test]
    fn test_database_example() {
        let db = database::Database::new();

        assert_eq!(db.transaction_state, database::TransactionState::None);
        assert_eq!(db.connection_count, 0);

        println!("✓ Database state machine example structure created");
    }
}

fn main() {
    println!("State machine testing examples");
    println!("Run with: cargo test --example state-machine-examples");
}
