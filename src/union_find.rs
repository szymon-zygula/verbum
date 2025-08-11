//! A `UnionFind` implementation (also known as Disjoint-Set Union), used to efficiently manage equivalence classes and perform union and find operations on sets.
use std::cell::Cell;

pub type SetId = usize;

/// A `UnionFind` data structure, also known as a Disjoint-Set Union (DSU).
///
/// The `UnionFind` struct is used to efficiently manage and query the connected components
/// of a set of elements. It supports two primary operations:
///
/// 1. **Union**: Connects two elements, merging their respective components.
/// 2. **Find**: Identifies the representative or "root" of the component to which an element belongs.
///    This is useful for checking if two elements are in the same component.
///
/// # Fields
/// - `parents`: A vector of `Cell<SetId>` representing the parent of each element in the disjoint-set structure.
///   Each element either points to its parent or, in the case of a root node, points to itself.
///   `Cell` is used for interior mutability.
///
/// # Derives
/// - `Clone`: Allows the `UnionFind` structure to be cloned, creating a deep copy of the internal state.
/// - `Default`: Provides a default empty initialization for the `UnionFind` structure.
///
/// # Example
/// ```
/// use your_crate::UnionFind;
///
/// let mut uf = UnionFind::default();
/// // Example usage would involve initializing the structure with elements,
/// // performing union and find operations, etc.
/// ```
#[derive(Clone, Default)]
pub struct UnionFind {
    parents: Vec<Cell<SetId>>,
}

impl UnionFind {
    pub fn with_size(size: usize) -> Self {
        let mut vec = Vec::with_capacity(size);

        for i in 0..size {
            vec.push(Cell::new(i));
        }

        Self { parents: vec }
    }

    pub fn size(&self) -> usize {
        self.parents.len()
    }

    pub fn add(&mut self) -> SetId {
        let new_id = self.size();
        self.parents.push(Cell::new(new_id));
        new_id
    }

    pub fn parent(&self, id: SetId) -> SetId {
        self.parents[id].get()
    }

    /// After the union, `id_2` becomes the canonical version of the set represented by id_1`
    pub fn union(&mut self, id_1: SetId, id_2: SetId) {
        let id_1 = self.find(id_1);
        let id_2 = self.find(id_2);
        self.parents[id_1] = Cell::new(id_2);
    }

    pub fn find_no_compress(&self, id: SetId) -> SetId {
        let parent = self.parent(id);
        if parent == id {
            return id;
        }

        self.find_no_compress(parent)
    }

    pub fn find(&self, id: SetId) -> SetId {
        let parent = self.parent(id);
        if parent == id {
            return id;
        }

        let grandparent = self.parent(parent);
        self.parents[id].set(grandparent);

        self.find(parent)
    }
}

impl PartialEq for UnionFind {
    fn eq(&self, other: &Self) -> bool {
        if self.parents.len() != other.parents.len() {
            return false;
        }

        for id in 0..self.parents.len() {
            if self.find(id) != other.find(id) {
                return false;
            }
        }

        true
    }
}

impl Eq for UnionFind {}

#[cfg(test)]
mod tests {
    #[test]
    fn creation() {
        let uf = super::UnionFind::with_size(10);

        assert_eq!(uf.size(), 10)
    }

    #[test]
    fn union_and_find() {
        let mut uf = super::UnionFind::with_size(4);
        uf.union(0, 2);
        uf.union(3, 1);
        uf.union(1, 0);

        assert_eq!(uf.find(0), uf.find(1));
        assert_eq!(uf.find(0), uf.find(2));
        assert_eq!(uf.find(0), uf.find(3));
    }

    #[test]
    fn add_set() {
        let mut uf = super::UnionFind::with_size(1);
        let new_id = uf.add();

        assert_eq!(new_id, 1);
        assert_eq!(uf.size(), 2);
    }

    #[test]
    fn compression() {
        let mut uf = super::UnionFind::with_size(3);
        uf.union(0, 1);
        uf.union(1, 2);

        assert_eq!(uf.parents[0].get(), 1);
        uf.find(0);
        assert_eq!(uf.parents[0].get(), 2);
    }
}
