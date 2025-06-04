use oxigraph::model::{Term, NamedNode, BlankNode, Literal};
use crate::components::Component;
use crate::shape::Shape;

pub type ID = u64;

pub enum Path {
    Simple(Term),
}

pub enum Target {
    Class(Term),
    Node(Term),
    SubjectsOf(NamedNode),
    ObjectsOf(NamedNode),
}

