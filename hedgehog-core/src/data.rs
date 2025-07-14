//! Core data types for Hedgehog property-based testing.

use std::fmt;

/// Size parameter for controlling test data generation.
/// 
/// Size typically ranges from 0 to 100, where larger values
/// generate more complex test data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size(pub usize);

impl Size {
    /// Create a new size value.
    pub fn new(value: usize) -> Self {
        Size(value)
    }
    
    /// Get the inner size value.
    pub fn get(&self) -> usize {
        self.0
    }
}

impl From<usize> for Size {
    fn from(value: usize) -> Self {
        Size(value)
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Size({})", self.0)
    }
}

/// Splittable random seed for deterministic test generation.
///
/// Seeds can be split to create independent random streams,
/// ensuring deterministic and reproducible test runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seed(pub u64, pub u64);

impl Seed {
    /// Create a new seed from a single value.
    pub fn from_u64(value: u64) -> Self {
        Seed(value, value.wrapping_mul(0x9e3779b97f4a7c15))
    }
    
    /// Split a seed into two independent seeds.
    pub fn split(self) -> (Self, Self) {
        let Seed(a, b) = self;
        let c = a.wrapping_add(b);
        let d = b.wrapping_add(c);
        (Seed(a, c), Seed(b, d))
    }
    
    /// Generate a random seed.
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Seed(rng.gen(), rng.gen())
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Seed({}, {})", self.0, self.1)
    }
}

/// Configuration for property testing.
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum number of tests to run.
    pub test_limit: usize,
    
    /// Maximum number of shrinks to attempt.
    pub shrink_limit: usize,
    
    /// Maximum size parameter to use.
    pub size_limit: usize,
    
    /// Maximum number of discards before giving up.
    pub discard_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            test_limit: 100,
            shrink_limit: 1000,
            size_limit: 100,
            discard_limit: 100,
        }
    }
}

impl Config {
    /// Create a new config with the given number of tests.
    pub fn with_tests(mut self, tests: usize) -> Self {
        self.test_limit = tests;
        self
    }
    
    /// Create a new config with the given shrink limit.
    pub fn with_shrinks(mut self, shrinks: usize) -> Self {
        self.shrink_limit = shrinks;
        self
    }
    
    /// Create a new config with the given size limit.
    pub fn with_size_limit(mut self, size: usize) -> Self {
        self.size_limit = size;
        self
    }
}