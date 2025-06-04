use crate::types::{ID, ComponentID};
use crate::types::{Path, Target};
// SHACL, Term, NamedNode, TermRef were unused

#[derive(Debug)]
pub enum Shape {
    NodeShape(NodeShape),
    PropertyShape(PropertyShape),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct PropertyShape {
    identifier: ID,
    path: Path,
    constraints: Vec<ComponentID>,
    // TODO severity
    // TODO message
}

impl PropertyShape {
    pub fn new(identifier: ID, path: Path, constraints: Vec<ComponentID>) -> Self {
        PropertyShape { identifier, path, constraints }
    }
}
