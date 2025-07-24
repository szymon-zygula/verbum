use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
};

type SetId = usize;
type DataId = usize;

pub trait UnionData: Debug {
    fn merge(&mut self, other: Self);
}

#[derive(Debug)]
enum DataUnionFindNode {
    Data(DataId),
    Pointer(Cell<SetId>),
}

#[derive(Debug, Default)]
pub struct DataUnionFind<D: UnionData> {
    data: HashMap<DataId, D>,
    contents: Vec<DataUnionFindNode>,
}

impl<D: UnionData> DataUnionFind<D> {
    pub fn with_size(size: usize, mut generator: impl FnMut(usize) -> D) -> Self {
        let mut data = HashMap::with_capacity(size);
        let mut contents = Vec::with_capacity(size);

        for i in 0..size {
            data.insert(i, generator(i));
            contents.push(DataUnionFindNode::Data(i));
        }

        Self { data, contents }
    }

    pub fn size(&self) -> usize {
        self.contents.len()
    }

    fn add(&mut self, data: D) -> SetId {
        let new_id = self.size();
        self.data.insert(new_id, data);
        self.contents.push(DataUnionFindNode::Data(new_id));
        new_id
    }

    fn parent(&self, id: SetId) -> SetId {
        match &self.contents[id] {
            DataUnionFindNode::Data(_) => id,
            DataUnionFindNode::Pointer(cell) => cell.get(),
        }
    }

    /// Merges set with ID `id_1` with set with ID `id_2`. The new canonical ID is `id_2`.
    fn union(&mut self, id_1: SetId, id_2: SetId) {
        let (id_1_canon, data_1_id) = self.find(id_1);
        let (id_2_canon, data_2_id) = self.find(id_2);

        if id_1_canon == id_2_canon {
            // Nothing to do, both ids point to the same data
            return;
        }

        let data_1 = self.data.remove(&data_1_id).unwrap();
        self.data.get_mut(&data_2_id).unwrap().merge(data_1);

        self.contents[id_1_canon] = DataUnionFindNode::Pointer(Cell::new(id_2_canon));
    }

    fn find(&self, id: SetId) -> (SetId, DataId) {
        match &self.contents[id] {
            DataUnionFindNode::Data(data_id) => (id, *data_id),
            DataUnionFindNode::Pointer(parent) => {
                let grandparent = self.parent(parent.get());
                parent.set(grandparent);

                self.find(parent.get())
            }
        }
    }
}

/// A reference to a Union-Find set which always returns the canonical version of its set and
/// can be used to interact with other sets.
#[derive(Clone, Debug)]
pub struct Set<D: UnionData> {
    set_id: Cell<SetId>,
    union_find: Rc<RefCell<DataUnionFind<D>>>,
}

impl<D: UnionData> Set<D> {
    pub fn new(union_find: &Rc<RefCell<DataUnionFind<D>>>, data: D) -> Self {
        let set_id = union_find.borrow_mut().add(data);

        Self {
            set_id: Cell::new(set_id),
            union_find: Rc::clone(union_find),
        }
    }

    fn get(&self) -> SetId {
        // Update the cached id with the canonical one before returning it
        let (canonical_id, _) = self.union_find.borrow().find(self.set_id.get());
        self.set_id.set(canonical_id);
        canonical_id
    }

    pub fn union(self, other: Set<D>) -> Set<D> {
        assert!(
            std::ptr::eq(self.union_find.as_ref(), other.union_find.as_ref()),
            "Sets on which a union is performed have to belong to the same union-find."
        );

        self.union_find
            .borrow_mut()
            .union(self.set_id.get(), other.set_id.get());

        self
    }

    pub fn data_intro<R>(&self, fun: impl FnOnce(&D) -> R) -> R {
        fun(&self.union_find.borrow().data[&self.union_find.borrow().find(self.set_id.get()).0])
    }
}

impl<D: UnionData> PartialEq for Set<D> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<D: UnionData> Eq for Set<D> {}

impl UnionData for () {
    fn merge(&mut self, _: Self) {}
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::UnionData;

    #[test]
    fn creation_with_addition() {
        let mut uf = super::DataUnionFind::<()>::default();
        uf.add(());
        uf.add(());
        uf.add(());

        assert_eq!(uf.size(), 3)
    }

    #[test]
    fn creation_with_size() {
        let uf = super::DataUnionFind::with_size(10, |_| ());

        assert_eq!(uf.size(), 10)
    }

    #[test]
    fn union_and_find() {
        let mut uf = super::DataUnionFind::with_size(4, |_| ());
        uf.union(0, 2);
        uf.union(3, 1);
        uf.union(1, 0);

        assert_eq!(uf.find(0), uf.find(1));
        assert_eq!(uf.find(0), uf.find(2));
        assert_eq!(uf.find(0), uf.find(3));
    }

    #[test]
    fn add_set() {
        let mut uf = super::DataUnionFind::with_size(1, |_| ());
        let new_id = uf.add(());

        assert_eq!(new_id, 1);
        assert_eq!(uf.size(), 2);
    }

    #[test]
    fn compression() {
        let mut uf = super::DataUnionFind::with_size(3, |_| ());
        uf.union(0, 1);
        uf.union(1, 2);

        match &uf.contents[0] {
            crate::data_union_find::DataUnionFindNode::Data(_) => panic!("Variant not expected"),
            crate::data_union_find::DataUnionFindNode::Pointer(cell) => assert_eq!(cell.get(), 1),
        }

        uf.find(0);

        match &uf.contents[0] {
            crate::data_union_find::DataUnionFindNode::Data(_) => panic!("Variant not expected"),
            crate::data_union_find::DataUnionFindNode::Pointer(cell) => assert_eq!(cell.get(), 2),
        }
    }

    #[test]
    fn union_find_user() {
        let uf = Rc::new(RefCell::new(super::DataUnionFind::default()));
        let set_0 = super::Set::new(&uf, ());
        let set_1 = super::Set::new(&uf, ());
        let set_2 = super::Set::new(&uf, ());

        let set_01 = set_0.clone().union(set_1.clone());

        assert_eq!(set_01, set_0);
        assert_eq!(set_01, set_1);
        assert_eq!(set_0, set_1);

        assert_ne!(set_0, set_2);
        assert_ne!(set_1, set_2);

        let set_3 = super::Set::new(&uf, ());

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

    #[derive(Clone, Debug, Default)]
    struct TestData(i32);

    impl UnionData for TestData {
        fn merge(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    #[test]
    fn merge_with_data() {
        let uf = Rc::new(RefCell::new(super::DataUnionFind::default()));
        let set_0 = super::Set::new(&uf, TestData(2));
        let set_1 = super::Set::new(&uf, TestData(5));
        let set_2 = super::Set::new(&uf, TestData(9));

        let set_01 = set_0.clone().union(set_1.clone());

        assert_eq!(set_0.data_intro(|data| data.0), 7);
        assert_eq!(set_1.data_intro(|data| data.0), 7);
        assert_eq!(set_01.data_intro(|data| data.0), 7);
        assert_eq!(set_2.data_intro(|data| data.0), 9);

        let set_21 = set_2.clone().union(set_1.clone());

        assert_eq!(set_0.data_intro(|data| data.0), 16);
        assert_eq!(set_1.data_intro(|data| data.0), 16);
        assert_eq!(set_01.data_intro(|data| data.0), 16);
        assert_eq!(set_2.data_intro(|data| data.0), 16);
        assert_eq!(set_21.data_intro(|data| data.0), 16);
    }
}
