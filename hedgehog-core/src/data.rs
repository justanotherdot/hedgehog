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

    /// Scale size by a factor.
    pub fn scale(&self, factor: f64) -> Self {
        Size((self.0 as f64 * factor) as usize)
    }

    /// Clamp size to a maximum value.
    pub fn clamp(&self, max: usize) -> Self {
        Size(self.0.min(max))
    }

    /// Golden ratio progression for size scaling.
    pub fn golden(&self) -> Self {
        Size((self.0 as f64 * 0.61803398875) as usize)
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
        let state = splitmix64_mix(value);
        let gamma = mix_gamma(state);
        Seed(state, gamma)
    }

    /// Split a seed into two independent seeds.
    /// Uses SplitMix64 splitting strategy for independence.
    pub fn split(self) -> (Self, Self) {
        let Seed(state, gamma) = self;
        let new_state = state.wrapping_add(gamma);
        let output = splitmix64_mix(new_state);
        let new_gamma = mix_gamma(output);

        (Seed(new_state, gamma), Seed(output, new_gamma))
    }

    /// Generate the next random value and advance the seed.
    /// Uses SplitMix64 algorithm for high-quality randomness.
    pub fn next_u64(self) -> (u64, Self) {
        let Seed(state, gamma) = self;
        let new_state = state.wrapping_add(gamma);
        let output = splitmix64_mix(new_state);
        (output, Seed(new_state, gamma))
    }

    /// Generate a bounded random value [0, bound).
    pub fn next_bounded(self, bound: u64) -> (u64, Self) {
        let (value, new_seed) = self.next_u64();
        ((value as u128 * bound as u128 >> 64) as u64, new_seed)
    }

    /// Generate a random bool.
    pub fn next_bool(self) -> (bool, Self) {
        let (value, new_seed) = self.next_u64();
        (value & 1 == 1, new_seed)
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

/// SplitMix64 mixing function for high-quality output.
fn splitmix64_mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// Generate a good gamma value for SplitMix64 splitting.
fn mix_gamma(mut z: u64) -> u64 {
    z = splitmix64_mix(z);
    // Ensure gamma is odd for maximal period
    (z | 1).wrapping_mul(0x9e3779b97f4a7c15)
}

/// A range for generating numeric values with enhanced shrinking.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range<T> {
    /// Lower bound (inclusive).
    pub min: T,
    /// Upper bound (inclusive).
    pub max: T,
    /// Origin point for shrinking (usually zero or closest valid value).
    pub origin: Option<T>,
}

impl<T> Range<T>
where
    T: Copy + PartialOrd,
{
    /// Create a new range with the given bounds.
    pub fn new(min: T, max: T) -> Self {
        Range {
            min,
            max,
            origin: None,
        }
    }

    /// Set the origin point for shrinking.
    pub fn with_origin(mut self, origin: T) -> Self {
        self.origin = Some(origin);
        self
    }

    /// Check if a value is within this range.
    pub fn contains(&self, value: &T) -> bool {
        value >= &self.min && value <= &self.max
    }
}

impl Range<i32> {
    /// Create a positive range [1, i32::MAX].
    pub fn positive() -> Self {
        Range::new(1, i32::MAX).with_origin(1)
    }

    /// Create a natural range [0, i32::MAX].
    pub fn natural() -> Self {
        Range::new(0, i32::MAX).with_origin(0)
    }

    /// Create a small positive range [1, 100].
    pub fn small_positive() -> Self {
        Range::new(1, 100).with_origin(1)
    }
}

impl Range<i64> {
    /// Create a positive range [1, i64::MAX].
    pub fn positive() -> Self {
        Range::new(1, i64::MAX).with_origin(1)
    }

    /// Create a natural range [0, i64::MAX].
    pub fn natural() -> Self {
        Range::new(0, i64::MAX).with_origin(0)
    }
}

impl Range<u32> {
    /// Create a positive range [1, u32::MAX].
    pub fn positive() -> Self {
        Range::new(1, u32::MAX).with_origin(1)
    }

    /// Create a natural range [0, u32::MAX].
    pub fn natural() -> Self {
        Range::new(0, u32::MAX).with_origin(0)
    }
}

impl Range<f64> {
    /// Create a unit range [0.0, 1.0].
    pub fn unit() -> Self {
        Range::new(0.0, 1.0).with_origin(0.0)
    }

    /// Create a positive range [f64::EPSILON, f64::MAX].
    pub fn positive() -> Self {
        Range::new(f64::EPSILON, f64::MAX).with_origin(f64::EPSILON)
    }

    /// Create a natural range [0.0, f64::MAX].
    pub fn natural() -> Self {
        Range::new(0.0, f64::MAX).with_origin(0.0)
    }

    /// Create a standard normal-like range [-3.0, 3.0].
    pub fn normal() -> Self {
        Range::new(-3.0, 3.0).with_origin(0.0)
    }
}
