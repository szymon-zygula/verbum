//! A graph that associates data with both vertices and edges.

use crate::graph::data_graph::DataGraph;
use crate::graph::VertexId;
use std::collections::HashMap;

/// A graph that associates data with both vertices and edges.
/// Wraps a `DataGraph` and adds a layer for edge data.
pub struct EdgeDataGraph<V, E> {
    pub data_graph: DataGraph<V>,
    pub edge_data: HashMap<(VertexId, VertexId), E>,
}

impl<V, E> EdgeDataGraph<V, E> {
    /// Creates a new empty `EdgeDataGraph`.
    pub fn new() -> Self {
        Self {
            data_graph: DataGraph::new(),
            edge_data: HashMap::new(),
        }
    }

    /// Adds a vertex with its associated data and returns the vertex index.
    pub fn add_vertex(&mut self, data: V) -> VertexId {
        self.data_graph.add_vertex(data)
    }

    /// Adds a directed edge with its associated data.
    pub fn add_edge(&mut self, from: VertexId, to: VertexId, data: E) {
        let num_vertices = self.data_graph.graph.num_vertices();
        if from < num_vertices && to < num_vertices {
            self.data_graph.graph.add_edge(from, to);
            self.edge_data.insert((from, to), data);
        }
    }

    /// Returns an iterator over the out-neighbors of a given vertex, with their edge data.
    pub fn out_neighbors_with_data(&self, vertex_idx: VertexId) -> impl Iterator<Item = (VertexId, &E)> {
        self.data_graph
            .out_neighbors(vertex_idx)
            .iter()
            .filter_map(move |&neighbor_idx| {
                self.edge_data
                    .get(&(vertex_idx, neighbor_idx))
                    .map(|data| (neighbor_idx, data))
            })
    }

    /// Returns an iterator over the in-neighbors of a given vertex, with their edge data.
    pub fn in_neighbors_with_data(&self, vertex_idx: VertexId) -> impl Iterator<Item = (VertexId, &E)> {
        self.data_graph
            .in_neighbors(vertex_idx)
            .iter()
            .filter_map(move |&neighbor_idx| {
                self.edge_data
                    .get(&(neighbor_idx, vertex_idx))
                    .map(|data| (neighbor_idx, data))
            })
    }

    /// Returns an iterator over the data of the out-neighbors of a given vertex, with their edge data.
    pub fn out_neighbor_data(&self, vertex_idx: VertexId) -> impl Iterator<Item = (&V, &E)> {
        self.out_neighbors_with_data(vertex_idx)
            .filter_map(move |(idx, edge_data)| {
                self.data_graph.get_data(idx).map(|vertex_data| (vertex_data, edge_data))
            })
    }

    /// Returns an iterator over the data of the in-neighbors of a given vertex, with their edge data.
    pub fn in_neighbor_data(&self, vertex_idx: VertexId) -> impl Iterator<Item = (&V, &E)> {
        self.in_neighbors_with_data(vertex_idx)
            .filter_map(move |(idx, edge_data)| {
                self.data_graph.get_data(idx).map(|vertex_data| (vertex_data, edge_data))
            })
    }

    /// Finds a vertex by a predicate on its data.
    pub fn find_vertex<P>(&self, predicate: P) -> Option<(VertexId, &V)> 
    where 
        P: Fn(&V) -> bool, 
    {
        self.data_graph.find_vertex(predicate)
    }

    /// Returns a reference to the data of a given vertex.
    pub fn get_vertex_data(&self, vertex_idx: VertexId) -> Option<&V> {
        self.data_graph.get_data(vertex_idx)
    }

    /// Returns a reference to the data of a given edge.
    pub fn get_edge_data(&self, from: VertexId, to: VertexId) -> Option<&E> {
        self.edge_data.get(&(from, to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_data_graph_new() {
        let graph = EdgeDataGraph::<(), ()>::new();
        assert_eq!(graph.data_graph.graph.num_vertices(), 0);
        assert!(graph.data_graph.data.is_empty());
        assert!(graph.edge_data.is_empty());
    }

    #[test]
    fn test_edge_data_graph_add_vertex() {
        let mut graph = EdgeDataGraph::<i32, ()>::new();
        let v0 = graph.add_vertex(10);
        assert_eq!(v0, 0);
        assert_eq!(graph.get_vertex_data(v0), Some(&10));
    }

    #[test]
    fn test_edge_data_graph_add_edge() {
        let mut graph = EdgeDataGraph::<(), &str>::new();
        let v0 = graph.add_vertex(());
        let v1 = graph.add_vertex(());
        graph.add_edge(v0, v1, "edge_v0_v1");
        assert_eq!(graph.get_edge_data(v0, v1), Some(&"edge_v0_v1"));
    }

    #[test]
    fn test_edge_data_graph_neighbors() {
        let mut graph = EdgeDataGraph::new();
        let v0 = graph.add_vertex(10);
        let v1 = graph.add_vertex(20);
        let v2 = graph.add_vertex(30);
        graph.add_edge(v0, v1, "v0->v1");
        graph.add_edge(v2, v0, "v2->v0");

        let out: Vec<_> = graph.out_neighbors_with_data(v0).collect();
        assert_eq!(out, vec![(v1, &"v0->v1")]);

        let in_neighbors: Vec<_> = graph.in_neighbors_with_data(v0).collect();
        assert_eq!(in_neighbors, vec![(v2, &"v2->v0")]);
    }

    #[test]
    fn test_edge_data_graph_neighbor_data() {
        let mut graph = EdgeDataGraph::new();
        let v0 = graph.add_vertex(10);
        let v1 = graph.add_vertex(20);
        let v2 = graph.add_vertex(30);
        graph.add_edge(v0, v1, "v0->v1");
        graph.add_edge(v2, v0, "v2->v0");

        let out: Vec<_> = graph.out_neighbor_data(v0).collect();
        assert_eq!(out, vec![(&20, &"v0->v1")]);

        let in_data: Vec<_> = graph.in_neighbor_data(v0).collect();
        assert_eq!(in_data, vec![(&30, &"v2->v0")]);
    }
}