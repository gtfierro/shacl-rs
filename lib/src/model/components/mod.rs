use std::collections::HashMap;

use oxigraph::model::{NamedNode, Term};

use crate::types::{PropShapeID, ID};

pub mod sparql;

use crate::model::templates::ComponentTemplateDefinition;
use sparql::CustomConstraintComponentDefinition;

/// Data-only description of a SHACL constraint component extracted during parsing.
#[derive(Debug, Clone)]
pub enum ComponentDescriptor {
    Node {
        shape: ID,
    },
    Property {
        shape: PropShapeID,
    },
    QualifiedValueShape {
        shape: ID,
        min_count: Option<u64>,
        max_count: Option<u64>,
        disjoint: Option<bool>,
    },
    Class {
        class: Term,
    },
    Datatype {
        datatype: Term,
    },
    NodeKind {
        node_kind: Term,
    },
    MinCount {
        min_count: u64,
    },
    MaxCount {
        max_count: u64,
    },
    MinExclusive {
        value: Term,
    },
    MinInclusive {
        value: Term,
    },
    MaxExclusive {
        value: Term,
    },
    MaxInclusive {
        value: Term,
    },
    MinLength {
        length: u64,
    },
    MaxLength {
        length: u64,
    },
    Pattern {
        pattern: String,
        flags: Option<String>,
    },
    LanguageIn {
        languages: Vec<String>,
    },
    UniqueLang {
        enabled: bool,
    },
    Equals {
        property: Term,
    },
    Disjoint {
        property: Term,
    },
    LessThan {
        property: Term,
    },
    LessThanOrEquals {
        property: Term,
    },
    Not {
        shape: ID,
    },
    And {
        shapes: Vec<ID>,
    },
    Or {
        shapes: Vec<ID>,
    },
    Xone {
        shapes: Vec<ID>,
    },
    Closed {
        closed: bool,
        ignored_properties: Vec<Term>,
    },
    HasValue {
        value: Term,
    },
    In {
        values: Vec<Term>,
    },
    Sparql {
        constraint_node: Term,
    },
    Custom {
        definition: CustomConstraintComponentDefinition,
        parameter_values: HashMap<NamedNode, Vec<Term>>,
    },
}

impl ComponentDescriptor {
    #[allow(dead_code)]
    /// Returns the template definition that produced this descriptor, if any.
    pub fn template_definition(&self) -> Option<&ComponentTemplateDefinition> {
        match self {
            ComponentDescriptor::Custom { definition, .. } => definition.template.as_ref(),
            _ => None,
        }
    }
}
