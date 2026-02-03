use std::{fs::File, io::Write};
use verbum::{
    graph::{DataGraph, EdgeDataGraph, Graph},
    language, macros,
    rewriting::{
        egraph::{
            class::simple_math_local_cost::SimpleMathLocalCost,
            matching::bottom_up::BottomUpMatcher,
            saturation::{Saturator, SaturationConfig, SimpleSaturator},
            EGraph,
        },
        rule::Rule,
    },
};

fn main() {
    println!("Running dot examples...");
    save_graph_dot();
    save_data_graph_dot();
    save_edge_data_graph_dot();
    test_saturation_and_dot_output();
    println!("Dot examples finished. Files were saved to the root of the project.");
}

fn save_graph_dot() {
    let mut graph = Graph::new();
    let v0 = graph.add_vertex();
    let v1 = graph.add_vertex();
    let v2 = graph.add_vertex();
    graph.add_edge(v0, v1);
    graph.add_edge(v1, v2);
    graph.add_edge(v2, v0);

    let dot_output = graph.dot();
    let mut file = File::create("graph.dot").expect("Unable to create file");
    file.write_all(dot_output.as_bytes())
        .expect("Unable to write data");
    println!("Saved graph.dot");
}

fn save_data_graph_dot() {
    let mut graph = DataGraph::new();
    let v0 = graph.add_vertex("A");
    let v1 = graph.add_vertex("B");
    let v2 = graph.add_vertex("C");
    graph.add_edge(v0, v1);
    graph.add_edge(v1, v2);
    graph.add_edge(v2, v0);

    let dot_output = graph.dot();
    let mut file = File::create("data_graph.dot").expect("Unable to create file");
    file.write_all(dot_output.as_bytes())
        .expect("Unable to write data");
    println!("Saved data_graph.dot");
}

fn save_edge_data_graph_dot() {
    let mut graph = EdgeDataGraph::new();
    let v0 = graph.add_vertex("Node A");
    let v1 = graph.add_vertex("Node B");
    let v2 = graph.add_vertex("Node C");
    graph.add_edge(v0, v1, "Edge 1");
    graph.add_edge(v1, v2, "Edge 2");
    graph.add_edge(v2, v0, "Edge 3");

    let dot_output = graph.dot();
    let mut file = File::create("edge_data_graph.dot").expect("Unable to create file");
    file.write_all(dot_output.as_bytes())
        .expect("Unable to write data");
    println!("Saved edge_data_graph.dot");
}

fn test_saturation_and_dot_output() {
    let lang = language::Language::simple_math();
    let rules: Vec<Rule> = macros::rules!(lang;
        "(* $0 2)" => "(<< $0 1)",
        "(* $0 1)" => "$0",
        "(/ (* $0 $1) $2)" => "(* $0 (/ $1 $2))",
        "(/ $0 $0)" => "1",
    );

    let expression = lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap();
    let mut egraph = EGraph::<SimpleMathLocalCost>::from_expression(expression);

    let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
    let config = SaturationConfig::default();
    saturator.saturate(&mut egraph, &rules, &config);

    egraph.save_dot(&lang, "output.dot").unwrap();
}
