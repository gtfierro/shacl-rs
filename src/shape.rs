use crate::types::{ID, ComponentID};
use crate::types::{Path, Target};
// SHACL, Term, NamedNode, TermRef were unused

pub enum Shape {
    NodeShape(NodeShape),
    PropertyShape(PropertyShape),
}

pub struct NodeShape {
    identifier: ID,
    targets: Vec<Target>,
    property_shapes: Vec<ID>,
    constraints: Vec<ComponentID>,
    // TODO severity
    // TODO message
}

impl NodeShape {
    pub fn new(identifier: ID, targets: Vec<Target>, property_shapes: Vec<ID>, constraints: Vec<ComponentID>) -> Self {
        NodeShape {
            identifier,
            targets,
            property_shapes,
            constraints,
        }
    }
}

pub struct PropertyShape {
    identifier: ID,
    path: Path,
    constraints: Vec<ID>,
    // TODO severity
    // TODO message
}
