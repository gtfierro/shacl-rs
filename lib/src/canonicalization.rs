use oxigraph::model::{Graph, NamedNode, Term};
use petgraph::algo::is_isomorphic_matching;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Converts an `oxigraph::model::Graph` to a `petgraph::graph::DiGraph`.
///
/// Each unique subject and object in the oxigraph graph becomes a node in the petgraph graph.
/// Each triple becomes a directed edge from the subject node to the object node, with the
/// predicate as the edge weight.
pub fn oxigraph_to_petgraph(ox_graph: &Graph) -> DiGraph<Term, NamedNode> {
    let mut pg_graph = DiGraph::<Term, NamedNode>::new();
    let mut node_map = HashMap::<Term, NodeIndex>::new();

    for triple_ref in ox_graph.iter() {
        let subject_term = Term::from(triple_ref.subject.into_owned());
        let object_term = triple_ref.object.into_owned();
        let predicate = triple_ref.predicate.into_owned();

        let s_node = *node_map
            .entry(subject_term.clone())
            .or_insert_with(|| pg_graph.add_node(subject_term));
        let o_node = *node_map
            .entry(object_term.clone())
            .or_insert_with(|| pg_graph.add_node(object_term));

        pg_graph.add_edge(s_node, o_node, predicate);
    }

    pg_graph
}

/// Checks if two `oxigraph::model::Graph`s are isomorphic.
///
/// This is done by converting both graphs to `petgraph` directed graphs and then
/// using `petgraph::algo::is_isomorphic_matching` to check for isomorphism.
pub fn are_isomorphic(g1: &Graph, g2: &Graph) -> bool {
    let pg1 = oxigraph_to_petgraph(g1);
    let pg2 = oxigraph_to_petgraph(g2);

    is_isomorphic_matching(&pg1, &pg2, |n1, n2| n1 == n2, |e1, e2| e1 == e2)
}
