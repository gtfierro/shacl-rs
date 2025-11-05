#![allow(dead_code)]

use crate::model::components::sparql::SPARQLValidator;
use crate::types::Severity;
use oxigraph::model::{NamedNode, Term};
use std::collections::BTreeMap;

/// Describes a parameter declared on a SHACL template.
#[derive(Debug, Clone)]
pub struct TemplateParameter {
    /// The RDF term that identifies the parameter (often a blank node).
    pub subject: Term,
    /// The predicate path (`sh:path`) that binds values supplied by template callers.
    pub path: NamedNode,
    /// Optional human-readable name exposed via `sh:name`.
    pub name: Option<String>,
    /// Optional description provided through `sh:description`.
    pub description: Option<String>,
    /// Whether the parameter may be omitted (`sh:optional true`).
    pub optional: bool,
    /// Values provided via `sh:defaultValue`.
    pub default_values: Vec<Term>,
    /// Optional query variable override supplied by `sh:varName`.
    pub var_name: Option<String>,
    /// Additional metadata that we do not model explicitly yet (e.g., `sh:datatype`).
    pub extra: BTreeMap<NamedNode, Vec<Term>>,
}

impl TemplateParameter {
    /// Returns `true` when the parameter has at least one declared default value.
    pub fn has_default(&self) -> bool {
        !self.default_values.is_empty()
    }
}

/// Captures the SPARQL validator bodies exposed by a template.
#[derive(Debug, Clone, Default)]
pub struct TemplateValidators {
    /// General-purpose validator referenced via `sh:validator`.
    pub validator: Option<SPARQLValidator>,
    /// Validator specialised for node shapes (`sh:nodeValidator`).
    pub node_validator: Option<SPARQLValidator>,
    /// Validator specialised for property shapes (`sh:propertyValidator`).
    pub property_validator: Option<SPARQLValidator>,
}

impl TemplateValidators {
    /// True when no validator clauses are attached to the template.
    pub fn is_empty(&self) -> bool {
        self.validator.is_none()
            && self.node_validator.is_none()
            && self.property_validator.is_none()
    }
}

/// Represents a SHACL constraint component template (`sh:ConstraintComponent`).
#[derive(Debug, Clone)]
pub struct ComponentTemplateDefinition {
    /// Template IRI.
    pub iri: NamedNode,
    /// Optional label (e.g., `rdfs:label`).
    pub label: Option<String>,
    /// Optional description/comment.
    pub comment: Option<String>,
    /// Declared parameters.
    pub parameters: Vec<TemplateParameter>,
    /// SPARQL validators associated with the template.
    pub validators: TemplateValidators,
    /// Messages defined directly on the template (`sh:message`).
    pub messages: Vec<Term>,
    /// Severity override declared on the template (`sh:severity`).
    pub severity: Option<Severity>,
    /// Prefix declarations attached to the template (`sh:declare`).
    pub prefix_declarations: Vec<PrefixDeclaration>,
    /// Additional predicates preserved for future use.
    pub extra: BTreeMap<NamedNode, Vec<Term>>,
}

impl ComponentTemplateDefinition {
    /// Convenience helper to fetch a parameter by its path predicate.
    pub fn parameter_by_path(&self, path: &NamedNode) -> Option<&TemplateParameter> {
        self.parameters.iter().find(|param| &param.path == path)
    }
}

/// Represents a SHACL shape template (`sh:Shape`).
#[derive(Debug, Clone)]
pub struct ShapeTemplateDefinition {
    /// Template IRI.
    pub iri: NamedNode,
    /// Optional label/comment metadata.
    pub label: Option<String>,
    pub comment: Option<String>,
    /// Declared template parameters.
    pub parameters: Vec<TemplateParameter>,
    /// The root node of the template body within the shapes graph.
    pub body: Term,
    /// Prefix declarations available to the template.
    pub prefix_declarations: Vec<PrefixDeclaration>,
    /// Additional predicates attached to the template declaration.
    pub extra: BTreeMap<NamedNode, Vec<Term>>,
}

/// Represents a `sh:PrefixDeclaration` used by templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixDeclaration {
    pub prefix: String,
    pub namespace: String,
}
