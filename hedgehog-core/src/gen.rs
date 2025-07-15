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

impl Gen<char> {
    /// Generate ASCII alphabetic characters (a-z, A-Z).
    pub fn ascii_alpha() -> Self {
        Gen::new(|_size, seed| {
            let (value, _new_seed) = seed.next_bounded(52);
            let ch = if value < 26 {
                (b'a' + value as u8) as char
            } else {
                (b'A' + (value - 26) as u8) as char
            };
            Tree::singleton(ch)
        })
    }

    /// Generate ASCII alphanumeric characters (a-z, A-Z, 0-9).
    pub fn ascii_alphanumeric() -> Self {
        Gen::new(|_size, seed| {
            let (value, _new_seed) = seed.next_bounded(62);
            let ch = if value < 26 {
                (b'a' + value as u8) as char
            } else if value < 52 {
                (b'A' + (value - 26) as u8) as char
            } else {
                (b'0' + (value - 52) as u8) as char
            };
            Tree::singleton(ch)
        })
    }

    /// Generate printable ASCII characters (space through tilde).
    pub fn ascii_printable() -> Self {
        Gen::new(|_size, seed| {
            let (value, _new_seed) = seed.next_bounded(95);
            let ch = (b' ' + value as u8) as char;
            Tree::singleton(ch)
        })
    }
}

impl Gen<String> {
    /// Generate strings using the given character generator.
    pub fn string_of(char_gen: Gen<char>) -> Self {
        Gen::new(move |size, seed| {
            let (len_seed, chars_seed) = seed.split();
            let (length, _) = len_seed.next_bounded(size.get() as u64 + 1);

            let mut current_seed = chars_seed;
            let mut chars = Vec::new();
            let mut char_trees = Vec::new();

            for _ in 0..length {
                let (char_seed, next_seed) = current_seed.split();
                current_seed = next_seed;

                let char_tree = char_gen.generate(size, char_seed);
                chars.push(char_tree.value);
                char_trees.push(char_tree);
            }

            let string_value: String = chars.iter().collect();

            // Generate shrinks by removing characters and shrinking individual chars
            let mut shrinks = Vec::new();

            // Shrink by removing characters
            if !chars.is_empty() {
                // Remove last character
                let shorter: String = chars[..chars.len() - 1].iter().collect();
                shrinks.push(Tree::singleton(shorter));

                // Remove first character
                if chars.len() > 1 {
                    let shorter: String = chars[1..].iter().collect();
                    shrinks.push(Tree::singleton(shorter));
                }

                // Try empty string
                if chars.len() > 2 {
                    shrinks.push(Tree::singleton(String::new()));
                }
            }

            Tree::with_children(string_value, shrinks)
        })
    }

    /// Generate ASCII alphabetic strings.
    pub fn ascii_alpha() -> Self {
        Self::string_of(Gen::<char>::ascii_alpha())
    }

    /// Generate ASCII alphanumeric strings.
    pub fn ascii_alphanumeric() -> Self {
        Self::string_of(Gen::<char>::ascii_alphanumeric())
    }

    /// Generate printable ASCII strings.
    pub fn ascii_printable() -> Self {
        Self::string_of(Gen::<char>::ascii_printable())
    }
}

impl<T> Gen<Vec<T>>
where
    T: 'static + Clone,
{
    /// Generate vectors using the given element generator.
    pub fn vec_of(element_gen: Gen<T>) -> Self {
        Gen::new(move |size, seed| {
            let (len_seed, elements_seed) = seed.split();
            let (length, _) = len_seed.next_bounded(size.get() as u64 + 1);

            let mut current_seed = elements_seed;
            let mut elements = Vec::new();
            let mut element_trees = Vec::new();

            for _ in 0..length {
                let (element_seed, next_seed) = current_seed.split();
                current_seed = next_seed;

                let element_tree = element_gen.generate(size, element_seed);
                elements.push(element_tree.value.clone());
                element_trees.push(element_tree);
            }

            // Generate shrinks by removing elements
            let mut shrinks = Vec::new();

            // Shrink by removing elements
            if !elements.is_empty() {
                // Remove last element
                let mut shorter = elements.clone();
                shorter.pop();
                shrinks.push(Tree::singleton(shorter));

                // Remove first element
                if elements.len() > 1 {
                    let shorter = elements[1..].to_vec();
                    shrinks.push(Tree::singleton(shorter));
                }

                // Try empty vector
                if elements.len() > 2 {
                    shrinks.push(Tree::singleton(Vec::new()));
                }

                // Try removing middle elements for larger vectors
                if elements.len() > 4 {
                    let mid = elements.len() / 2;
                    let shorter = [&elements[..mid], &elements[mid + 1..]].concat();
                    shrinks.push(Tree::singleton(shorter));
                }
            }

            Tree::with_children(elements, shrinks)
        })
    }
}

impl Gen<Vec<i32>> {
    /// Generate vectors of integers.
    pub fn vec_int() -> Self {
        Self::vec_of(Gen::int_range(-100, 100))
    }
}

impl Gen<Vec<bool>> {
    /// Generate vectors of booleans.
    pub fn vec_bool() -> Self {
        Self::vec_of(Gen::bool())
    }
}

impl<T> Gen<Option<T>>
where
    T: 'static + Clone,
{
    /// Generate optional values using the given generator.
    pub fn option_of(inner_gen: Gen<T>) -> Self {
        Gen::new(move |size, seed| {
            let (choice_seed, value_seed) = seed.split();
            let (choice, _) = choice_seed.next_bounded(4);

            if choice == 0 {
                // Generate None (25% chance)
                Tree::singleton(None)
            } else {
                // Generate Some(value) (75% chance)
                let value_tree = inner_gen.generate(size, value_seed);
                let some_value = Some(value_tree.value.clone());

                // Shrink to None and shrink the inner value
                let mut shrinks = vec![Tree::singleton(None)];

                // Add shrinks of the inner value wrapped in Some
                for shrink in value_tree.shrinks() {
                    shrinks.push(Tree::singleton(Some(shrink.clone())));
                }

                Tree::with_children(some_value, shrinks)
            }
        })
    }
}

impl<T, U> Gen<(T, U)>
where
    T: 'static + Clone,
    U: 'static + Clone,
{
    /// Generate tuples using the given generators.
    pub fn tuple_of(first_gen: Gen<T>, second_gen: Gen<U>) -> Self {
        Gen::new(move |size, seed| {
            let (first_seed, second_seed) = seed.split();

            let first_tree = first_gen.generate(size, first_seed);
            let second_tree = second_gen.generate(size, second_seed);

            let tuple_value = (first_tree.value.clone(), second_tree.value.clone());

            // Generate shrinks by shrinking each component
            let mut shrinks = Vec::new();

            // Shrink first component, keep second
            for first_shrink in first_tree.shrinks() {
                let shrunk_tuple = (first_shrink.clone(), second_tree.value.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            // Shrink second component, keep first
            for second_shrink in second_tree.shrinks() {
                let shrunk_tuple = (first_tree.value.clone(), second_shrink.clone());
                shrinks.push(Tree::singleton(shrunk_tuple));
            }

            Tree::with_children(tuple_value, shrinks)
        })
    }
}
