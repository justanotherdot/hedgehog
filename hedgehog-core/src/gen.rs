//! Generator combinators for property-based testing.

use crate::{data::*, tree::*};

/// A generator for test data of type `T`.
///
/// Generators are explicit, first-class values that can be composed
/// using combinator functions. This is a key difference from
/// type-directed approaches like QuickCheck.
pub struct Gen<T> {
    generator: Box<dyn Fn(Size, Seed) -> Tree<T>>,
}

impl<T> Gen<T> {
    /// Create a new generator from a function.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Size, Seed) -> Tree<T> + 'static,
    {
        Gen {
            generator: Box::new(f),
        }
    }

    /// Generate a value using the given size and seed.
    pub fn generate(&self, size: Size, seed: Seed) -> Tree<T> {
        (self.generator)(size, seed)
    }

    /// Create a generator that always produces the same value.
    pub fn constant(value: T) -> Self
    where
        T: Clone + 'static,
    {
        Gen::new(move |_size, _seed| Tree::singleton(value.clone()))
    }
}

impl<T> Gen<T>
where
    T: 'static,
{
    /// Map a function over the generated values.
    pub fn map<U, F>(self, f: F) -> Gen<U>
    where
        F: Fn(T) -> U + 'static + Clone,
        U: 'static,
    {
        Gen::new(move |size, seed| {
            let tree = self.generate(size, seed);
            tree.map(f.clone())
        })
    }

    /// Bind/flatmap for dependent generation.
    pub fn bind<U, F>(self, f: F) -> Gen<U>
    where
        F: Fn(T) -> Gen<U> + 'static,
        U: 'static,
        T: Clone,
    {
        Gen::new(move |size, seed| {
            let (seed1, seed2) = seed.split();
            let tree = self.generate(size, seed1);
            tree.bind(|value| f(value.clone()).generate(size, seed2))
        })
    }

    /// Filter generated values by a predicate.
    pub fn filter<F>(self, predicate: F) -> Gen<T>
    where
        F: Fn(&T) -> bool + 'static,
        T: Clone,
    {
        Gen::new(move |size, seed| {
            let tree = self.generate(size, seed);
            let value = tree.value.clone();
            tree.filter(&predicate)
                .unwrap_or_else(|| Tree::singleton(value))
        })
    }
}

/// Primitive generators.
impl Gen<bool> {
    /// Generate a random boolean.
    pub fn bool() -> Self {
        Gen::new(|_size, seed| {
            let (value, _new_seed) = seed.next_bool();
            Tree::singleton(value)
        })
    }
}

impl Gen<i32> {
    /// Generate an integer in the given range.
    pub fn int_range(min: i32, max: i32) -> Self {
        Gen::new(move |_size, seed| {
            let range = (max - min + 1) as u64;
            let (value, _new_seed) = seed.next_bounded(range);
            let result = min + value as i32;

            // Create shrinks towards zero
            let mut shrinks = Vec::new();
            let mut current = result;
            while current != 0 && current != min {
                current = if current > 0 {
                    current / 2
                } else {
                    current / 2
                };
                if current >= min && current <= max && current != result {
                    shrinks.push(Tree::singleton(current));
                }
            }

            Tree::with_children(result, shrinks)
        })
    }

    /// Generate a positive integer.
    pub fn positive() -> Self {
        Self::int_range(1, i32::MAX)
    }

    /// Generate a natural number (including zero).
    pub fn natural() -> Self {
        Self::int_range(0, i32::MAX)
    }
}
