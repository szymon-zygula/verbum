//! A module for representing directed graphs.

pub type VertexId = usize;

/// A simple sparse directed graph using an adjacency list.
#[derive(PartialEq, Debug)]
pub struct Graph {
    pub adjacency: Vec<Vec<VertexId>>,
    pub rev_adjacency: Vec<Vec<VertexId>>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    /// Creates a new empty graph.
    pub fn new() -> Self {
        Self {
            adjacency: Vec::new(),
            rev_adjacency: Vec::new(),
        }
    }

    /// Adds a new vertex to the graph and returns its index.
    pub fn add_vertex(&mut self) -> VertexId {
        let id = self.adjacency.len();
        self.adjacency.push(Vec::new());
        self.rev_adjacency.push(Vec::new());
        id
    }

    /// Adds a directed edge from `from` to `to`.
    ///
    /// # Panics
    ///
    /// Panics if `from` or `to` are out of bounds.
    pub fn add_edge(&mut self, from: VertexId, to: VertexId) {
        if from < self.adjacency.len() && to < self.adjacency.len() {
            self.adjacency[from].push(to);
            self.rev_adjacency[to].push(from);
        }
    }

    /// Returns the number of vertices in the graph.
    pub fn num_vertices(&self) -> usize {
        self.adjacency.len()
    }

    /// Returns a slice of out-neighbors for a given vertex.
    pub fn out_neighbors(&self, vertex: VertexId) -> &[VertexId] {
        self.adjacency.get(vertex).map_or(&[], |v| v.as_slice())
    }

        /// Returns a slice of in-neighbors for a given vertex.
    pub fn in_neighbors(&self, vertex: VertexId) -> &[VertexId] {
        self.rev_adjacency.get(vertex).map_or(&[], |v| v.as_slice())
    }

    /// Returns a string in DOT format representing the graph.
    pub fn dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph G {
");
        for i in 0..self.num_vertices() {
            dot.push_str(&format!("    {i};\n"));
        }
        for i in 0..self.num_vertices() {
            for &neighbor in self.out_neighbors(i) {
                dot.push_str(&format!("    {i} -> {neighbor};\n"));
            }
        }
        dot.push('}');
        dot
    }
}


pub mod data_graph;
pub mod edge_data_graph;

pub use self::data_graph::DataGraph;
pub use self::edge_data_graph::EdgeDataGraph;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_new() {
        let graph = Graph::new();
        assert_eq!(graph.num_vertices(), 0);
    }

    #[test]
    fn test_graph_add_vertex() {
        let mut graph = Graph::new();
        let v0 = graph.add_vertex();
        let v1 = graph.add_vertex();
        assert_eq!(v0, 0);
        assert_eq!(v1, 1);
        assert_eq!(graph.num_vertices(), 2);
    }

    #[test]
    fn test_graph_add_edge() {
        let mut graph = Graph::new();
        let v0 = graph.add_vertex();
        let v1 = graph.add_vertex();
        graph.add_edge(v0, v1);
        assert_eq!(graph.out_neighbors(v0), &[v1]);
        assert_eq!(graph.in_neighbors(v1), &[v0]);
        assert_eq!(graph.out_neighbors(v1), &[] as &[VertexId]);
        assert_eq!(graph.in_neighbors(v0), &[] as &[VertexId]);
    }
}
