use crate::context::{Context, TraceItem, ValidationContext};
use crate::types::Path;
use oxigraph::model::{BlankNode, Graph, Literal, NamedOrBlankNode, Subject, Term, Triple};
use oxigraph::vocab::{rdf, sh};
use std::collections::HashMap; // For using Term as a HashMap key

pub struct ValidationReportBuilder {
    pub(crate) results: Vec<(Context, String)>, // Made pub(crate)
}

impl ValidationReportBuilder {
    pub fn new() -> Self {
        ValidationReportBuilder {
            results: Vec::new(),
        }
    }

    pub fn add_error(&mut self, context: &Context, error: String) {
        // Store the context by cloning it, as the original context might have a shorter lifetime.
        // The error string is moved.
        self.results.push((context.clone(), error));
        // The println! macro is removed as per the request to track errors instead of printing.
    }

    pub fn results(&self) -> &[(Context, String)] {
        &self.results
    }

    pub fn to_graph(&self, validation_context: &ValidationContext) -> Graph {
        let mut graph = Graph::new();
        let report_node: Subject = BlankNode::default().into();

        graph
            .insert(&Triple::new(
                report_node.clone(),
                rdf::TYPE,
                sh::VALIDATION_REPORT.into(),
            ))
            .unwrap();

        let conforms = self.results.is_empty();
        graph
            .insert(&Triple::new(
                report_node.clone(),
                sh::CONFORMS,
                Literal::from(conforms).into(),
            ))
            .unwrap();

        if !conforms {
            for (context, error_message) in &self.results {
                let result_node: Subject = BlankNode::default().into();
                graph
                    .insert(&Triple::new(
                        report_node.clone(),
                        sh::RESULT,
                        result_node.clone().into(),
                    ))
                    .unwrap();

                graph
                    .insert(&Triple::new(
                        result_node.clone(),
                        rdf::TYPE,
                        sh::VALIDATION_RESULT.into(),
                    ))
                    .unwrap();

                // sh:focusNode
                graph
                    .insert(&Triple::new(
                        result_node.clone(),
                        sh::FOCUS_NODE,
                        context.focus_node().clone(),
                    ))
                    .unwrap();

                // sh:resultMessage
                graph
                    .insert(&Triple::new(
                        result_node.clone(),
                        sh::RESULT_MESSAGE,
                        Literal::new_simple_literal(error_message).into(),
                    ))
                    .unwrap();

                // Extract info from trace
                let mut source_shape_term = None;
                let mut result_path_term = None;
                let mut source_constraint_component_term = None;

                for item in context.execution_trace().iter().rev() {
                    match item {
                        TraceItem::NodeShape(id) => {
                            if source_shape_term.is_none() {
                                source_shape_term = validation_context
                                    .nodeshape_id_lookup()
                                    .borrow()
                                    .get_term(*id)
                                    .cloned();
                            }
                        }
                        TraceItem::PropertyShape(id) => {
                            if source_shape_term.is_none() {
                                source_shape_term = validation_context
                                    .propshape_id_lookup()
                                    .borrow()
                                    .get_term(*id)
                                    .cloned();
                                if let Some(shape) = validation_context.get_prop_shape_by_id(id) {
                                    if result_path_term.is_none() {
                                        result_path_term = Some(path_to_rdf(shape.path(), &mut graph));
                                    }
                                }
                            }
                        }
                        TraceItem::Component(id) => {
                            if source_constraint_component_term.is_none() {
                                source_constraint_component_term = validation_context
                                    .component_id_lookup()
                                    .borrow()
                                    .get_term(*id)
                                    .cloned();
                            }
                        }
                    }
                }

                if let Some(term) = source_shape_term {
                    graph
                        .insert(&Triple::new(
                            result_node.clone(),
                            sh::SOURCE_SHAPE,
                            term,
                        ))
                        .unwrap();
                }

                if let Some(term) = result_path_term {
                    graph
                        .insert(&Triple::new(
                            result_node.clone(),
                            sh::RESULT_PATH,
                            term,
                        ))
                        .unwrap();
                }

                graph
                    .insert(&Triple::new(
                        result_node.clone(),
                        sh::RESULT_SEVERITY,
                        sh::VIOLATION.into(),
                    ))
                    .unwrap();

                if let Some(term) = source_constraint_component_term {
                    graph
                        .insert(&Triple::new(
                            result_node.clone(),
                            sh::SOURCE_CONSTRAINT_COMPONENT,
                            term,
                        ))
                        .unwrap();
                }
            }
        }

        graph
    }

    pub fn dump(&self) {
        if self.results.is_empty() {
            println!("Validation report: No errors found.");
            return;
        }

        println!("Validation Report:");
        println!("------------------");

        let mut grouped_errors: HashMap<Term, Vec<(&Context, &String)>> = HashMap::new();

        for (context, error_message) in &self.results {
            grouped_errors
                .entry(context.focus_node().clone())
                .or_default()
                .push((context, error_message));
        }

        for (focus_node, context_error_pairs) in grouped_errors {
            println!("\nFocus Node: {}", focus_node);
            for (context, error) in context_error_pairs {
                println!("  - Error: {}", error);
                println!("    From shape: {}", context.source_shape());
                println!("    Trace: {:?}", context.execution_trace());
            }
        }
        println!("\n------------------");
    }
}

fn path_to_rdf(path: &Path, graph: &mut Graph) -> Term {
    match path {
        Path::Simple(term) => term.clone(),
        Path::Inverse(inner) => {
            let bn: Subject = BlankNode::default().into();
            let inner_term = path_to_rdf(inner, graph);
            graph
                .insert(&Triple::new(bn.clone(), sh::INVERSE_PATH, inner_term))
                .unwrap();
            bn.into_term()
        }
        Path::Sequence(paths) => {
            let items: Vec<Term> = paths.iter().map(|p| path_to_rdf(p, graph)).collect();
            build_rdf_list(items, graph)
        }
        Path::Alternative(paths) => {
            let bn: Subject = BlankNode::default().into();
            let items: Vec<Term> = paths.iter().map(|p| path_to_rdf(p, graph)).collect();
            let list_head = build_rdf_list(items, graph);
            graph
                .insert(&Triple::new(bn.clone(), sh::ALTERNATIVE_PATH, list_head))
                .unwrap();
            bn.into_term()
        }
        Path::ZeroOrMore(inner) => {
            let bn: Subject = BlankNode::default().into();
            let inner_term = path_to_rdf(inner, graph);
            graph
                .insert(&Triple::new(bn.clone(), sh::ZERO_OR_MORE_PATH, inner_term))
                .unwrap();
            bn.into_term()
        }
        Path::OneOrMore(inner) => {
            let bn: Subject = BlankNode::default().into();
            let inner_term = path_to_rdf(inner, graph);
            graph
                .insert(&Triple::new(bn.clone(), sh::ONE_OR_MORE_PATH, inner_term))
                .unwrap();
            bn.into_term()
        }
        Path::ZeroOrOne(inner) => {
            let bn: Subject = BlankNode::default().into();
            let inner_term = path_to_rdf(inner, graph);
            graph
                .insert(&Triple::new(bn.clone(), sh::ZERO_OR_ONE_PATH, inner_term))
                .unwrap();
            bn.into_term()
        }
    }
}

fn build_rdf_list(items: impl IntoIterator<Item = Term>, graph: &mut Graph) -> Term {
    let mut head: Subject = rdf::NIL.into();
    let mut tail = head.clone();

    let items: Vec<Term> = items.into_iter().collect();
    if items.is_empty() {
        return head.into_term();
    }

    let bnodes: Vec<NamedOrBlankNode> =
        (0..items.len()).map(|_| BlankNode::default().into()).collect();
    head = bnodes[0].clone().into();

    for (i, item) in items.iter().enumerate() {
        let subject: Subject = bnodes[i].clone().into();
        graph
            .insert(&Triple::new(subject.clone(), rdf::FIRST, item.clone()))
            .unwrap();
        let rest: Term = if i == items.len() - 1 {
            rdf::NIL.into()
        } else {
            bnodes[i + 1].clone().into()
        };
        graph.insert(&Triple::new(subject, rdf::REST, rest)).unwrap();
    }
    head.into_term()
}
