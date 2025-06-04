use oxigraph::model::{Term, NamedNode, BlankNode, Literal, SubjectRef, NamedNodeRef, TermRef};
use oxigraph::model::{Dataset, Graph, GraphName, Quad, Triple};
use crate::types::ID;

pub fn parse_component(start: Term, shape_graph: &Graph) -> Option<Component> {
    // SHACL vocabulary constants
    const SH_CLASS_STR: &str = "http://www.w3.org/ns/shacl#class";
    const SH_NODE_STR: &str = "http://www.w3.org/ns/shacl#node";
    const SH_PROPERTY_STR: &str = "http://www.w3.org/ns/shacl#property";

    let identifier = match ID::from_term(&start) {
        Ok(id) => id,
        Err(_) => return None, // Failed to create ID from start term
    };

    let start_term_ref = start.as_ref();
    let subject_for_graph_query: SubjectRef = match start_term_ref {
        TermRef::NamedNode(nn_ref) => nn_ref.into(),
        TermRef::BlankNode(bn_ref) => bn_ref.into(),
        _ => return None, // Literals or other term types cannot be subjects of shape definitions
    };

    let sh_class_node = NamedNode::new_unchecked(SH_CLASS_STR);
    let sh_node_node = NamedNode::new_unchecked(SH_NODE_STR);
    let sh_property_node = NamedNode::new_unchecked(SH_PROPERTY_STR);

    let mut found_class_param: Option<Term> = None;
    let mut found_node_param: Option<ID> = None;
    let mut found_property_param: Option<ID> = None;

    // A single pass over the quads to find relevant parameters
    for quad in shape_graph.quads_for_subject(subject_for_graph_query) {
        let triple = quad.as_triple();
        let predicate_ref = triple.predicate; // type NamedNodeRef
        let object_term = triple.object.clone(); // type Term

        if predicate_ref == sh_class_node.as_ref() {
            found_class_param = Some(object_term);
        } else if predicate_ref == sh_node_node.as_ref() {
            if let Ok(id) = ID::from_term(&object_term) {
                found_node_param = Some(id);
            }
            // else: object is not a valid ID for a shape, could log or handle as error
        } else if predicate_ref == sh_property_node.as_ref() {
            if let Ok(id) = ID::from_term(&object_term) {
                found_property_param = Some(id);
            }
            // else: object is not a valid ID for a shape, could log or handle as error
        }
    }

    // Apply priority: sh:class > sh:node > sh:property
    if let Some(class_val) = found_class_param {
        return Some(Component::ClassConstraint(ClassConstraintComponent {
            identifier,
            class: class_val,
        }));
    }
    if let Some(node_shape_id) = found_node_param {
        return Some(Component::NodeConstraint(NodeConstraintComponent {
            identifier,
            shape: node_shape_id,
        }));
    }
    if let Some(property_shape_id) = found_property_param {
        return Some(Component::PropertyConstraint(PropertyConstraintComponent {
            identifier,
            shape: property_shape_id,
        }));
    }

    None // No recognized SHACL constraint parameter found for 'start'
}

pub enum Component {
    ClassConstraint(ClassConstraintComponent),
    NodeConstraint(NodeConstraintComponent),
    PropertyConstraint(PropertyConstraintComponent),
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
