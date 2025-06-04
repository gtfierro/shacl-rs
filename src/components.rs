use oxigraph::model::{Term, NamedNode, BlankNode, Literal};
use oxigraph::model::{Dataset, Graph, GraphName, Quad, Triple};
use crate::types::ID;

pub fn parse_component(start: Term, shape_graph: &Graph) -> Option<Component> {
    if let Some(class_constraint) = parse_class_constraint(start, shape_graph) {
        return Some(Component::ClassConstraint(class_constraint));
    }
    if let Some(node_constraint) = parse_node_constraint(start, shape_graph) {
        return Some(Component::NodeConstraint(node_constraint));
    }
    if let Some(property_constraint) = parse_property_constraint(start, shape_graph) {
        return Some(Component::PropertyConstraint(property_constraint));
    }
    None
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
