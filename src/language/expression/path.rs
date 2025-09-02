use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OwnedPath(Vec<usize>);

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
