use crate::types::ID;
use crate::types::Path;

pub enum Shape {
    NodeShape(NodeShape),
    PropertyShape(PropertyShape),
}

pub struct NodeShape {
    identifier: ID,
    targets: Vec<ID>,
    property_shapes: Vec<ID>,
    constraints: Vec<ID>,
    // TODO severity
    // TODO message
}

pub struct PropertyShape {
    identifier: ID,
    path: Path,
    constraints: Vec<ID>,
    // TODO severity
    // TODO message
}

