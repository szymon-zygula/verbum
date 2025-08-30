use std::{collections::HashMap, fmt::Write, fs::File, path::Path};

use crate::language::Language;

use super::{Analysis, ClassId, EGraph, Node, NodeId, class::DynClass};

// This file contains only debugging code for drawing egraphs using `dot`.
// It was written by ChatGPT, as I don't know this language.
// Have fun debugging if it turns out it doesn't work.

const NODE_SHAPE: &str = "box";
const GRAPH_SPLINES: &str = "true";
const GRAPH_NODESEP: f32 = 1.0;
const GRAPH_RANKSEP: f32 = 1.2;

impl<A: Analysis> EGraph<A> {
    pub fn dot(&self, language: &Language) -> String {
        let mut out = String::new();
        writeln!(&mut out, "digraph egraph {{").unwrap();
        writeln!(&mut out, "  compound=true;").unwrap();
        writeln!(&mut out, "  node [shape={NODE_SHAPE:?}];").unwrap();
        writeln!(
            &mut out,
            "  graph [splines={GRAPH_SPLINES}, nodesep={GRAPH_NODESEP}, ranksep={GRAPH_RANKSEP}];"
        )
        .unwrap();

        let class_repr = self.write_clusters(&mut out, language);
        self.write_edges(&mut out, &class_repr);

        writeln!(&mut out, "}}\n").unwrap();
        out
    }

    fn write_clusters(&self, out: &mut String, language: &Language) -> HashMap<ClassId, NodeId> {
        let mut class_repr = HashMap::new();

        for (class_id, class) in &self.classes {
            writeln!(out, "  subgraph cluster_{class_id:?} {{").unwrap();
            let mut label = format!("Class {class_id:?}");
            if let Some(analysis_str) = class.analysis().to_string() {
                write!(label, " ({analysis_str})").unwrap();
            }
            writeln!(out, "    label = \"{label}\";").unwrap();

            for node_id in class.nodes_ids() {
                class_repr.entry(*class_id).or_insert(*node_id);

                if let Some(node) = self.nodes.get(node_id) {
                    let label = match node {
                        Node::Literal(lit) => format!("{lit:?}"),
                        Node::Symbol(sym) => language.get_symbol(sym.id).to_string(),
                    };
                    writeln!(out, "    {node_id:?} [label=\"{label}\"];").unwrap();
                }
            }
            writeln!(out, "  }}").unwrap();
        }

        class_repr
    }

    fn write_edges(&self, out: &mut String, class_repr: &HashMap<ClassId, NodeId>) {
        for class in self.classes.values() {
            for node_id in class.nodes_ids() {
                if let Some(Node::Symbol(sym)) = self.nodes.get(node_id) {
                    for (i, child_class) in sym.children.iter().enumerate() {
                        let canonical = self.union_find.find(*child_class);
                        if let Some(target_node) = class_repr.get(&canonical) {
                            writeln!(
                                out,
                                "  {node_id:?} -> {target_node:?} [lhead=cluster_{canonical:?}, taillabel=\"{i}\"];"
                            )
                            .unwrap();
                        }
                    }
                }
            }
        }
    }

    pub fn save_dot<P: AsRef<Path>>(&self, language: &Language, path: P) -> std::io::Result<()> {
        let dot_content = self.dot(language);
        let mut file = File::create(path)?;
        std::io::Write::write_all(&mut file, dot_content.as_bytes())
    }
}
