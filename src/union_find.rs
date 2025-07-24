use std::cell::Cell;

pub type SetId = usize;

// TODO: make two union finds: compressing and not notpressing?
#[derive(Default)]
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
