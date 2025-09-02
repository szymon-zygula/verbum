use serde::{Deserialize, Serialize};

use super::AnyExpression;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct OwnedPath(pub Vec<usize>);

impl OwnedPath {
    pub fn as_ref(&self) -> Path<'_> {
        Path(&self.0)
    }

    /// Returns the child path of this path
    pub fn child(&self) -> Path<'_> {
        self.as_ref().child()
    }

    /// Returns the first element on the path, or `None` if the path is empty
    pub fn head(&self) -> Option<usize> {
        self.0.first().copied()
    }

    /// Adds a new position at the end of the path
    pub fn push(&mut self, location: usize) {
        self.0.push(location)
    }

    /// Removes the last position from the path
    pub fn pop(&mut self) -> Option<usize> {
        self.0.pop()
    }

    /// Returns a `Path` referencing `self`
    pub fn as_path(&self) -> Path<'_> {
        Path(&self.0)
    }
}

/// Path to a subexpression in an expression
#[derive(Clone, Debug, PartialEq)]
pub struct Path<'p>(&'p [usize]);

impl<'p> Path<'p> {
    /// Returns the child path of this path
    pub fn child(&self) -> Self {
        Path(&self.0[1..])
    }

    /// Returns the first element on the path, or `None` if the path is empty
    pub fn head(&self) -> Option<usize> {
        self.0.first().copied()
    }
}

#[derive(Clone, Debug)]
pub struct SubexpressionPathIterator<'e, E: AnyExpression> {
    expression: &'e E,
    current_path: Option<OwnedPath>,
}

impl<'e, E: AnyExpression> SubexpressionPathIterator<'e, E> {
    pub fn new(expression: &'e E) -> Self {
        Self {
            expression,
            current_path: None,
        }
    }
}

impl<'e, E: AnyExpression> Iterator for SubexpressionPathIterator<'e, E> {
    type Item = OwnedPath;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(mut path) = self.current_path.take() else {
            // First iteration, return the root path
            let root_path = OwnedPath::default();
            self.current_path = Some(root_path.clone());
            return Some(root_path);
        };

        // Try to go deeper
        let subexpression = self.expression.subexpression(path.as_path()).unwrap();

        if let Some(children) = subexpression.children() {
            if !children.is_empty() {
                path.push(0);
                self.current_path = Some(path.clone());
                return Some(path);
            }
        }

        // Backtrack to find the next sibling
        loop {
            if path.as_ref().0.is_empty() {
                // We are at the root, and we have explored all children
                return None;
            }

            let last_index = path.pop().unwrap();
            let parent_path = path.as_path();
            let parent_expression = self.expression.subexpression(parent_path).unwrap();

            let parent_children = parent_expression.children().unwrap();
            let next_sibling_index = last_index + 1;

            if next_sibling_index < parent_children.len() {
                path.push(next_sibling_index);
                self.current_path = Some(path.clone());
                return Some(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;

    #[test]
    fn iterate_over_leaf() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("1").unwrap();
        let mut iterator = SubexpressionPathIterator::new(&expr);
        assert_eq!(iterator.next(), Some(OwnedPath(vec![])));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn iterate_over_simple_expression() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("(+ 1 2)").unwrap();
        let paths: Vec<_> = SubexpressionPathIterator::new(&expr).collect();
        assert_eq!(
            paths,
            vec![OwnedPath(vec![]), OwnedPath(vec![0]), OwnedPath(vec![1]),]
        );
    }

    #[test]
    fn iterate_over_nested_expression() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("(+ 1 (* 2 3) 4)").unwrap();
        let paths: Vec<_> = SubexpressionPathIterator::new(&expr).collect();
        assert_eq!(
            paths,
            vec![
                OwnedPath(vec![]),
                OwnedPath(vec![0]),
                OwnedPath(vec![1]),
                OwnedPath(vec![1, 0]),
                OwnedPath(vec![1, 1]),
                OwnedPath(vec![2]),
            ]
        );
    }
}
