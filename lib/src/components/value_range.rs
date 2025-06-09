use crate::context::{format_term_for_label, ValidationContext};
use crate::types::ComponentID;
use oxigraph::model::Term;

use super::GraphvizOutput;

// value range constraints
#[derive(Debug)]
pub struct MinExclusiveConstraintComponent {
    min_exclusive: Term,
}

impl GraphvizOutput for MinExclusiveConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        format!(
            "{} [label=\"MinExclusive: {}\"];",
            component_id.to_graphviz_id(),
            format_term_for_label(&self.min_exclusive)
        )
    }
}

#[derive(Debug)]
pub struct MinInclusiveConstraintComponent {
    min_inclusive: Term,
}

impl GraphvizOutput for MinInclusiveConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        format!(
            "{} [label=\"MinInclusive: {}\"];",
            component_id.to_graphviz_id(),
            format_term_for_label(&self.min_inclusive)
        )
    }
}

#[derive(Debug)]
pub struct MaxExclusiveConstraintComponent {
    max_exclusive: Term,
}

impl GraphvizOutput for MaxExclusiveConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        format!(
            "{} [label=\"MaxExclusive: {}\"];",
            component_id.to_graphviz_id(),
            format_term_for_label(&self.max_exclusive)
        )
    }
}

#[derive(Debug)]
pub struct MaxInclusiveConstraintComponent {
    max_inclusive: Term,
}

impl GraphvizOutput for MaxInclusiveConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        format!(
            "{} [label=\"MaxInclusive: {}\"];",
            component_id.to_graphviz_id(),
            format_term_for_label(&self.max_inclusive)
        )
    }
}
