use crate::context::{format_term_for_label, ValidationContext};
use crate::types::ComponentID;
use oxigraph::model::Term;

use super::GraphvizOutput;

// property pair constraints
#[derive(Debug)]
pub struct EqualsConstraintComponent {
    property: Term, // Should be an IRI
}

impl GraphvizOutput for EqualsConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        let property_name = format_term_for_label(&self.property);
        format!(
            "{} [label=\"Equals: {}\"];",
            component_id.to_graphviz_id(),
            property_name
        )
    }
}

#[derive(Debug)]
pub struct DisjointConstraintComponent {
    property: Term, // Should be an IRI
}

impl GraphvizOutput for DisjointConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        let property_name = format_term_for_label(&self.property);
        format!(
            "{} [label=\"Disjoint: {}\"];",
            component_id.to_graphviz_id(),
            property_name
        )
    }
}

#[derive(Debug)]
pub struct LessThanConstraintComponent {
    property: Term, // Should be an IRI
}

impl GraphvizOutput for LessThanConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        let property_name = format_term_for_label(&self.property);
        format!(
            "{} [label=\"LessThan: {}\"];",
            component_id.to_graphviz_id(),
            property_name
        )
    }
}

#[derive(Debug)]
pub struct LessThanOrEqualsConstraintComponent {
    property: Term, // Should be an IRI
}

impl GraphvizOutput for LessThanOrEqualsConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        let property_name = format_term_for_label(&self.property);
        format!(
            "{} [label=\"LessThanOrEquals: {}\"];",
            component_id.to_graphviz_id(),
            property_name
        )
    }
}
