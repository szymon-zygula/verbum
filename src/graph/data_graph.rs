//! A graph that associates data with each vertex.

use crate::graph::{Graph, VertexId};

/// A graph that associates data with each vertex.
pub struct DataGraph<T> {
    pub graph: Graph,
    pub data: Vec<T>,
}

impl<T> DataGraph<T> {
    /// Creates a new empty `DataGraph`.
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            data: Vec::new(),
        }
    }

    /// Adds a vertex with its associated data and returns the vertex index.
    pub fn add_vertex(&mut self, data: T) -> VertexId {
        let id = self.graph.add_vertex();
        self.data.push(data);
        id
    }

    /// Adds a directed edge from `from` to `to`.
    pub fn add_edge(&mut self, from: VertexId, to: VertexId) {
        self.graph.add_edge(from, to);
    }

    /// Returns a slice of out-neighbors for a given vertex.
    pub fn out_neighbors(&self, vertex_idx: VertexId) -> &[VertexId] {
        self.graph.out_neighbors(vertex_idx)
    }

    /// Returns a slice of in-neighbors for a given vertex.
    pub fn in_neighbors(&self, vertex_idx: VertexId) -> &[VertexId] {
        self.graph.in_neighbors(vertex_idx)
    }

    /// Returns an iterator over the data of the out-neighbors of a given vertex.
    pub fn out_neighbor_data(&self, vertex_idx: VertexId) -> impl Iterator<Item = &T> {
        self.out_neighbors(vertex_idx)
            .iter()
            .filter_map(move |&idx| self.data.get(idx))
    }

    /// Returns an iterator over the data of the in-neighbors of a given vertex.
    pub fn in_neighbor_data(&self, vertex_idx: VertexId) -> impl Iterator<Item = &T> {
        self.in_neighbors(vertex_idx)
            .iter()
            .filter_map(move |&idx| self.data.get(idx))
    }

    /// Finds a vertex by a predicate on its data.
    pub fn find_vertex<P>(&self, predicate: P) -> Option<(VertexId, &T)> 
    where 
        P: Fn(&T) -> bool, 
    {
        self.data
            .iter()
            .enumerate()
            .find(|(_, data)| predicate(data))
    }

    /// Returns a reference to the data of a given vertex.
    pub fn get_data(&self, vertex_idx: VertexId) -> Option<&T> {
        self.data.get(vertex_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_graph_new() {
        let graph = DataGraph::<i32>::new();
        assert_eq!(graph.graph.num_vertices(), 0);
        assert!(graph.data.is_empty());
    }

    #[test]
    fn test_data_graph_add_vertex() {
        let mut graph = DataGraph::new();
        let v0 = graph.add_vertex(10);
        let v1 = graph.add_vertex(20);
        assert_eq!(v0, 0);
        assert_eq!(v1, 1);
        assert_eq!(graph.get_data(v0), Some(&10));
        assert_eq!(graph.get_data(v1), Some(&20));
    }

    #[test]
    fn test_data_graph_add_edge() {
        let mut graph = DataGraph::<()>::new();
        let v0 = graph.add_vertex(());
        let v1 = graph.add_vertex(());
        graph.add_edge(v0, v1);
        assert_eq!(graph.out_neighbors(v0), &[v1]);
        assert_eq!(graph.in_neighbors(v1), &[v0]);
    }

    #[test]
    fn test_data_graph_neighbor_data() {
        let mut graph = DataGraph::new();
        let v0 = graph.add_vertex(10);
        let v1 = graph.add_vertex(20);
        let v2 = graph.add_vertex(30);
        graph.add_edge(v0, v1);
        graph.add_edge(v2, v0);

        let out: Vec<_> = graph.out_neighbor_data(v0).collect();
        assert_eq!(out, vec![&20]);

        let in_data: Vec<_> = graph.in_neighbor_data(v0).collect();
        assert_eq!(in_data, vec![&30]);
    }

    #[test]
    fn test_data_graph_find_vertex() {
        let mut graph = DataGraph::new();
        graph.add_vertex(10);
        graph.add_vertex(20);
        let (idx, &data) = graph.find_vertex(|d| *d == 20).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(data, 20);
    }
}