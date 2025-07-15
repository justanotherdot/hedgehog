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
    /// Distribution shape for generating values within the range.
    pub distribution: Distribution,
}

/// Distribution shapes for value generation within ranges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distribution {
    /// Uniform distribution across the range.
    Uniform,
    /// Linear distribution favoring smaller values.
    Linear,
    /// Exponential distribution strongly favoring smaller values.
    Exponential,
    /// Constant distribution (always generates the same value).
    Constant,
}

impl<T> Range<T>
where
    T: Copy + PartialOrd,
{
    /// Create a new range with the given bounds and uniform distribution.
    pub fn new(min: T, max: T) -> Self {
        Range {
            min,
            max,
            origin: None,
            distribution: Distribution::Uniform,
        }
    }

    /// Create a linear range that favors smaller values.
    pub fn linear(min: T, max: T) -> Self {
        Range {
            min,
            max,
            origin: None,
            distribution: Distribution::Linear,
        }
    }

    /// Create an exponential range that strongly favors smaller values.
    pub fn exponential(min: T, max: T) -> Self {
        Range {
            min,
            max,
            origin: None,
            distribution: Distribution::Exponential,
        }
    }

    /// Create a constant range that always generates the same value.
    pub fn constant(value: T) -> Self {
        Range {
            min: value,
            max: value,
            origin: Some(value),
            distribution: Distribution::Constant,
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

    /// Get the distribution shape for this range.
    pub fn distribution(&self) -> Distribution {
        self.distribution
    }
}

impl Range<i32> {
    /// Create a positive range [1, i32::MAX] with linear distribution.
    pub fn positive() -> Self {
        Range::linear(1, i32::MAX).with_origin(1)
    }

    /// Create a natural range [0, i32::MAX] with linear distribution.
    pub fn natural() -> Self {
        Range::linear(0, i32::MAX).with_origin(0)
    }

    /// Create a small positive range [1, 100] with uniform distribution.
    pub fn small_positive() -> Self {
        Range::new(1, 100).with_origin(1)
    }
}

impl Range<i64> {
    /// Create a positive range [1, i64::MAX] with linear distribution.
    pub fn positive() -> Self {
        Range::linear(1, i64::MAX).with_origin(1)
    }

    /// Create a natural range [0, i64::MAX] with linear distribution.
    pub fn natural() -> Self {
        Range::linear(0, i64::MAX).with_origin(0)
    }
}

impl Range<u32> {
    /// Create a positive range [1, u32::MAX] with linear distribution.
    pub fn positive() -> Self {
        Range::linear(1, u32::MAX).with_origin(1)
    }

    /// Create a natural range [0, u32::MAX] with linear distribution.
    pub fn natural() -> Self {
        Range::linear(0, u32::MAX).with_origin(0)
    }
}

impl Range<f64> {
    /// Create a unit range [0.0, 1.0] with uniform distribution.
    pub fn unit() -> Self {
        Range::new(0.0, 1.0).with_origin(0.0)
    }

    /// Create a positive range [f64::EPSILON, f64::MAX] with exponential distribution.
    pub fn positive() -> Self {
        Range::exponential(f64::EPSILON, f64::MAX).with_origin(f64::EPSILON)
    }

    /// Create a natural range [0.0, f64::MAX] with linear distribution.
    pub fn natural() -> Self {
        Range::linear(0.0, f64::MAX).with_origin(0.0)
    }

    /// Create a standard normal-like range [-3.0, 3.0] with uniform distribution.
    pub fn normal() -> Self {
        Range::new(-3.0, 3.0).with_origin(0.0)
    }
}

/// Helper functions for distribution sampling within ranges.
impl Distribution {
    /// Sample a value from the distribution within the given range.
    pub fn sample_u64(&self, seed: Seed, range_size: u64) -> (u64, Seed) {
        match self {
            Distribution::Uniform => seed.next_bounded(range_size),
            Distribution::Linear => {
                // Linear distribution: higher probability for smaller values
                // Use triangular distribution where P(x) = 2*(range_size - x) / (range_size^2)
                let (u1, new_seed) = seed.next_bounded(range_size);
                let (u2, final_seed) = new_seed.next_bounded(range_size);
                (u1.min(u2), final_seed)
            }
            Distribution::Exponential => {
                // Exponential distribution: much higher probability for smaller values
                // Use exponential decay: sample from geometric distribution
                let (uniform, new_seed) = seed.next_bounded(1000);
                let exponential = if uniform < 500 {
                    0
                } else if uniform < 750 {
                    1
                } else if uniform < 875 {
                    2
                } else if uniform < 937 {
                    3
                } else {
                    4.min(range_size.saturating_sub(1))
                };
                (exponential.min(range_size.saturating_sub(1)), new_seed)
            }
            Distribution::Constant => {
                // Always return 0 (will be adjusted by caller to the constant value)
                (0, seed)
            }
        }
    }

    /// Sample a float value from the distribution within [0.0, 1.0].
    pub fn sample_f64(&self, seed: Seed) -> (f64, Seed) {
        match self {
            Distribution::Uniform => {
                let (value, new_seed) = seed.next_u64();
                // Convert to [0.0, 1.0]
                let float_val = (value as f64) / (u64::MAX as f64);
                (float_val, new_seed)
            }
            Distribution::Linear => {
                // Linear distribution favoring smaller values
                let (u1, seed2) = seed.next_u64();
                let (u2, final_seed) = seed2.next_u64();
                let val1 = (u1 as f64) / (u64::MAX as f64);
                let val2 = (u2 as f64) / (u64::MAX as f64);
                (val1.min(val2), final_seed)
            }
            Distribution::Exponential => {
                // Exponential distribution strongly favoring smaller values
                let (uniform, new_seed) = seed.next_u64();
                let normalized = (uniform as f64) / (u64::MAX as f64);
                // Use exponential decay: -ln(1-u) / λ, with λ=2 for reasonable spread
                let exponential = if normalized >= 1.0 {
                    0.0
                } else {
                    -((1.0 - normalized).ln()) / 2.0
                };
                (exponential.min(1.0), new_seed)
            }
            Distribution::Constant => (0.0, seed),
        }
    }
}
