//! Rose tree implementation for shrinking test values.

use std::collections::VecDeque;

pub mod render;

/// A rose tree containing a value and its shrink possibilities.
///
/// Trees are used to represent generated values along with their
/// possible shrinks, enabling automatic shrinking of failing test cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree<T> {
    pub value: T,
    pub children: Vec<Tree<T>>,
}

impl<T> Tree<T> {
    /// Create a new tree with the given value and no children.
    pub fn singleton(value: T) -> Self {
        Tree {
            value,
            children: Vec::new(),
        }
    }

    /// Create a new tree with the given value and children.
    pub fn with_children(value: T, children: Vec<Tree<T>>) -> Self {
        Tree { value, children }
    }

    /// Map a function over the tree values.
    pub fn map<U, F>(self, f: F) -> Tree<U>
    where
        F: Fn(T) -> U + Clone,
    {
        Tree {
            value: f(self.value),
            children: self
                .children
                .into_iter()
                .map(|child| child.map(f.clone()))
                .collect(),
        }
    }

    /// Apply a function to the tree value and collect all results.
    pub fn bind<U, F>(self, f: F) -> Tree<U>
    where
        F: Fn(T) -> Tree<U> + Clone,
    {
        let Tree {
            value: new_value,
            children: new_children,
        } = f(self.value);

        let mapped_children: Vec<Tree<U>> = self
            .children
            .into_iter()
            .map(|child| child.bind(f.clone()))
            .collect();

        Tree {
            value: new_value,
            children: {
                let mut result = new_children;
                result.extend(mapped_children);
                result
            },
        }
    }

    /// Get all possible shrink values in breadth-first order.
    pub fn shrinks(&self) -> Vec<&T> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        for child in &self.children {
            queue.push_back(child);
        }

        while let Some(tree) = queue.pop_front() {
            result.push(&tree.value);
            for child in &tree.children {
                queue.push_back(child);
            }
        }

        result
    }

    /// Expand the tree to a given depth, collecting all values.
    pub fn expand(&self, max_depth: usize) -> Vec<&T> {
        let mut result = vec![&self.value];
        self.expand_recursive(&mut result, max_depth, 0);
        result
    }

    fn expand_recursive<'a>(
        &'a self,
        result: &mut Vec<&'a T>,
        max_depth: usize,
        current_depth: usize,
    ) {
        if current_depth >= max_depth {
            return;
        }

        for child in &self.children {
            result.push(&child.value);
            child.expand_recursive(result, max_depth, current_depth + 1);
        }
    }

    /// Filter the tree, keeping only values that satisfy the predicate.
    pub fn filter<F>(self, predicate: F) -> Option<Tree<T>>
    where
        F: Fn(&T) -> bool + Clone,
    {
        if !predicate(&self.value) {
            return None;
        }

        let filtered_children: Vec<Tree<T>> = self
            .children
            .into_iter()
            .filter_map(|child| child.filter(predicate.clone()))
            .collect();

        Some(Tree {
            value: self.value,
            children: filtered_children,
        })
    }

    /// Get the value from the tree.
    pub fn outcome(&self) -> &T {
        &self.value
    }

    /// Check if the tree has any children (shrinks).
    pub fn has_shrinks(&self) -> bool {
        !self.children.is_empty()
    }

    /// Count the total number of nodes in the tree.
    pub fn count_nodes(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|child| child.count_nodes())
            .sum::<usize>()
    }

    /// Get the depth of the tree.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self
                .children
                .iter()
                .map(|child| child.depth())
                .max()
                .unwrap_or(0)
        }
    }
}

impl<T> From<T> for Tree<T> {
    fn from(value: T) -> Self {
        Tree::singleton(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton_tree() {
        let tree = Tree::singleton(42);
        assert_eq!(tree.value, 42);
        assert!(tree.children.is_empty());
        assert!(!tree.has_shrinks());
    }

    #[test]
    fn test_tree_with_children() {
        let tree = Tree::with_children(10, vec![Tree::singleton(5), Tree::singleton(0)]);
        assert_eq!(tree.value, 10);
        assert_eq!(tree.children.len(), 2);
        assert!(tree.has_shrinks());
    }

    #[test]
    fn test_tree_map() {
        let tree = Tree::with_children(10, vec![Tree::singleton(5), Tree::singleton(0)]);
        let mapped = tree.map(|x| x * 2);
        assert_eq!(mapped.value, 20);
        assert_eq!(mapped.children[0].value, 10);
        assert_eq!(mapped.children[1].value, 0);
    }

    #[test]
    fn test_shrinks() {
        let tree = Tree::with_children(
            10,
            vec![
                Tree::with_children(5, vec![Tree::singleton(2)]),
                Tree::singleton(0),
            ],
        );
        let shrinks = tree.shrinks();
        assert_eq!(shrinks, vec![&5, &0, &2]);
    }

    #[test]
    fn test_tree_metrics() {
        let tree = Tree::with_children(
            10,
            vec![
                Tree::with_children(5, vec![Tree::singleton(2)]),
                Tree::singleton(0),
            ],
        );

        assert_eq!(tree.count_nodes(), 4); // 10, 5, 2, 0
        assert_eq!(tree.depth(), 3); // 10 -> 5 -> 2

        let singleton = Tree::singleton(42);
        assert_eq!(singleton.count_nodes(), 1);
        assert_eq!(singleton.depth(), 1);
    }
}
