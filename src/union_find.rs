use std::cell::{Cell, RefCell};

type SetId = usize;

// TODO: make two union finds: compressing and not notpressing?
#[derive(Debug, Default)]
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

    fn add(&mut self) -> SetId {
        let new_id = self.size();
        self.parents.push(Cell::new(new_id));
        new_id
    }

    fn parent(&self, id: SetId) -> SetId {
        self.parents[id].get()
    }

    /// Merges set with ID `id_1` with set with ID `id_2`. The new canonical ID is `id_2`.
    fn union(&mut self, id_1: SetId, id_2: SetId) {
        let id_1_canon = self.find(id_1);
        let id_2_canon = self.find(id_2);
        self.parents[id_1_canon] = Cell::new(id_2_canon);
    }

    fn find_no_compress(&self, id: SetId) -> SetId {
        let parent = self.parent(id);
        if parent == id {
            return id;
        }

        self.find_no_compress(parent)
    }

    fn find(&self, id: SetId) -> SetId {
        let parent = self.parent(id);
        if parent == id {
            return id;
        }

        let grandparent = self.parent(parent);
        self.parents[id].set(grandparent);

        self.find(parent)
    }
}

/// A reference to a Union-Find set which always returns the canonical version of its set and
/// can be used to interact with other sets.
#[derive(Clone, Debug)]
struct Set<'u> {
    set_id: Cell<SetId>,
    union_find: &'u RefCell<UnionFind>,
}

impl<'u> Set<'u> {
    pub fn new(union_find: &'u RefCell<UnionFind>) -> Self {
        let set_id = union_find.borrow_mut().add();

        Self {
            set_id: Cell::new(set_id),
            union_find,
        }
    }

    fn get(&self) -> SetId {
        // Update the cached id with the canonical one before returning it
        let canonical_id = self.union_find.borrow().find(self.set_id.get());
        self.set_id.set(canonical_id);
        canonical_id
    }

    pub fn union(self, other: Set<'u>) -> Set<'u> {
        assert!(
            std::ptr::eq(self.union_find, other.union_find),
            "Sets on which a union is performed have to belong to the same union-find."
        );

        self.union_find
            .borrow_mut()
            .union(self.set_id.get(), other.set_id.get());

        self
    }
}

impl<'u> PartialEq for Set<'u> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<'u> Eq for Set<'u> {}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

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

    #[test]
    fn union_find_user() {
        let uf = RefCell::new(super::UnionFind::with_size(3));
        let set_0 = super::Set::new(&uf);
        let set_1 = super::Set::new(&uf);
        let set_2 = super::Set::new(&uf);

        let set_01 = set_0.clone().union(set_1.clone());

        assert_eq!(set_01, set_0);
        assert_eq!(set_01, set_1);
        assert_eq!(set_0, set_1);

        assert_ne!(set_0, set_2);
        assert_ne!(set_1, set_2);

        let set_3 = super::Set::new(&uf);

        let set_31 = set_3.clone().union(set_1.clone());

        // Old hold
        assert_eq!(set_01, set_0);
        assert_eq!(set_01, set_1);
        assert_eq!(set_0, set_1);

        assert_ne!(set_0, set_2);
        assert_ne!(set_1, set_2);

        // New hold
        assert_eq!(set_0, set_3);
        assert_eq!(set_01, set_31);
        assert_eq!(set_1, set_3);
        assert_ne!(set_2, set_3);
        assert_eq!(set_1, set_31);
        assert_ne!(set_2, set_31);
    }
}
