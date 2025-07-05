use std::collections::HashMap;

use crate::{
    language::{Language, SymbolId},
    union_find::UnionFind,
};

pub type ClassId = usize;
pub type NodeId = usize;

struct Node {
    symbol: SymbolId,
    inputs: Vec<ClassId>,
}

struct EGraph {
    language: Language,
    classes: UnionFind,
    nodes: HashMap<Node, ClassId>,
}

impl EGraph {}
