#![allow(dead_code)]

mod language;
mod union_find;

use std::collections::HashMap;

use language::{Language, SymbolId};
use union_find::UnionFind;

pub type ClassId = usize;
pub type NodeId = usize;

struct RewriteRule {}

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

fn main() {}
