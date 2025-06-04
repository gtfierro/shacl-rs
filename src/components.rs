use oxigraph::model::{Term, TermRef, NamedNode, BlankNode, Literal};
use oxigraph::model::{Graph, QuadRef}; // Removed unused Dataset, GraphName, Quad, Triple
use oxigraph::vocab::xsd;
use crate::types::ID;

pub fn parse_components(start: Term, shape_graph: &Graph) -> Vec<Component> {
    let mut components = Vec::new();

    // SHACL vocabulary terms
    let sh_class = NamedNode::new_unchecked("http://www.w3.org/ns/shacl#class");
    let sh_node = NamedNode::new_unchecked("http://www.w3.org/ns/shacl#node");
    let sh_property = NamedNode::new_unchecked("http://www.w3.org/ns/shacl#property");
    let sh_qualified_value_shape = NamedNode::new_unchecked("http://www.w3.org/ns/shacl#qualifiedValueShape");
    let sh_qualified_min_count = NamedNode::new_unchecked("http://www.w3.org/ns/shacl#qualifiedMinCount");
    let sh_qualified_max_count = NamedNode::new_unchecked("http://www.w3.org/ns/shacl#qualifiedMaxCount");
    let sh_disjoint = NamedNode::new_unchecked("http://www.w3.org/ns/shacl#disjoint");

    for quad in shape_graph.quads_for_subject(start.as_ref()) {
        let predicate_ref = quad.predicate; // This is &NamedNodeRef
        let object_ref = quad.object; // This is &TermRef

        if predicate_ref == sh_class.as_ref() {
            components.push(Component::ClassConstraint(ClassConstraintComponent {
                identifier: start.clone(),
                class: Term::from(object_ref),
            }));
        } else if predicate_ref == sh_node.as_ref() {
            components.push(Component::NodeConstraint(NodeConstraintComponent {
                identifier: start.clone(),
                shape: Term::from(object_ref),
            }));
        } else if predicate_ref == sh_property.as_ref() {
            components.push(Component::PropertyConstraint(PropertyConstraintComponent {
                identifier: start.clone(),
                shape: Term::from(object_ref), // object_ref is the PropertyShape ID
            }));
        } else if predicate_ref == sh_qualified_value_shape.as_ref() {
            let qvs_details_node_term = Term::from(object_ref); // This is the node detailing QVS params
            let mut min_count = None;
            let mut max_count = None;
            let mut disjoint_prop = None;

            for q_quad in shape_graph.quads_for_subject(qvs_details_node_term.as_ref()) {
                let q_predicate_ref = q_quad.predicate;
                let q_object_ref = q_quad.object;

                if q_predicate_ref == sh_qualified_min_count.as_ref() {
                    if let TermRef::Literal(lit) = q_object_ref {
                        if lit.datatype() == xsd::INTEGER.as_ref() {
                            if let Ok(val) = lit.value().parse::<u64>() {
                                min_count = Some(val);
                            }
                            // Optional: else, log or handle malformed integer literal
                        }
                        // Optional: else, log or handle wrong datatype
                    }
                } else if q_predicate_ref == sh_qualified_max_count.as_ref() {
                    if let TermRef::Literal(lit) = q_object_ref {
                        if lit.datatype() == xsd::INTEGER.as_ref() {
                            if let Ok(val) = lit.value().parse::<u64>() {
                                max_count = Some(val);
                            }
                            // Optional: else, log or handle malformed integer literal
                        }
                        // Optional: else, log or handle wrong datatype
                    }
                } else if q_predicate_ref == sh_disjoint.as_ref() {
                    if let TermRef::Literal(lit) = q_object_ref {
                        if lit.datatype() == xsd::BOOLEAN.as_ref() {
                            match lit.value() {
                                "true" => disjoint_prop = Some(true),
                                "false" => disjoint_prop = Some(false),
                                _ => {} // Optional: log or handle malformed boolean literal
                            }
                        }
                        // Optional: else, log or handle wrong datatype
                    }
                }
            }

            components.push(Component::QualifiedValueShape(QualifiedValueShapeComponent {
                identifier: start.clone(),
                shape: qvs_details_node_term,
                min_count,
                max_count,
                disjoint: disjoint_prop,
            }));
        }
    }
    components
}

pub enum Component {
    ClassConstraint(ClassConstraintComponent),
    NodeConstraint(NodeConstraintComponent),
    PropertyConstraint(PropertyConstraintComponent),
    QualifiedValueShape(QualifiedValueShapeComponent),
}

pub struct ClassConstraintComponent {
    identifier: ID,
    class: Term,
}

pub struct NodeConstraintComponent {
    identifier: ID,
    shape: ID,
}

pub struct PropertyConstraintComponent {
    identifier: ID,
    shape: ID,
}

pub struct QualifiedValueShapeComponent {
    identifier: ID,
    shape: ID,
    min_count: Option<u64>,
    max_count: Option<u64>,
    disjoint: Option<bool>,
}
