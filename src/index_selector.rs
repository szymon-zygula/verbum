//! Multi-dimensional index iteration utilities.
//!
//! This module provides the [`IndexSelector`] for iterating over all possible
//! combinations of indices within given bounds, useful for cartesian product operations.

/// An iterator that generates all possible index combinations within given bounds.
///
/// This is useful for iterating over multi-dimensional spaces, similar to nested
/// loops but with dynamic dimensionality.
///
/// # Examples
///
/// ```no_run
/// # use verbum::index_selector::IndexSelector;
/// let selector = IndexSelector::new(vec![2, 3]);
/// // Will iterate over: [0,0], [0,1], [0,2], [1,0], [1,1], [1,2]
/// for indices in selector {
///     println!("{:?}", indices);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct IndexSelector {
    current: Vec<usize>,
    bounds: Vec<usize>,
    end: bool,
}

impl IndexSelector {
    /// Creates a new index selector with the given bounds.
    ///
    /// # Arguments
    ///
    /// * `bounds` - A vector specifying the upper bound (exclusive) for each dimension
    ///
    /// # Returns
    ///
    /// Returns an `IndexSelector` that will iterate over all index combinations
    pub fn new(bounds: Vec<usize>) -> Self {
        Self {
            current: vec![0; bounds.len()],
            end: bounds.iter().product::<usize>() == 0,
            bounds,
        }
    }
}

impl Iterator for IndexSelector {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        let old_current = self.current.clone();

        for i in 0..self.current.len() {
            if self.current[i] < self.bounds[i] - 1 {
                self.current[i] += 1;
                return Some(old_current);
            }

            self.current[i] = 0;
        }

        self.end = true;
        Some(old_current)
    }
}

#[cfg(test)]
mod tests {
    use super::IndexSelector;

    #[test]
    fn empty() {
        assert!(IndexSelector::new(vec![2, 4, 1, 0, 4, 5]).next().is_none());
    }

    fn something() {
        assert_eq!(IndexSelector::new(vec![2, 4, 5, 8, 2]).count(), 640);
    }
}
