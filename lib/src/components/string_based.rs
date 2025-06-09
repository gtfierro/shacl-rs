use crate::context::{sanitize_graphviz_string, ValidationContext};
use crate::types::ComponentID;

use super::GraphvizOutput;

// string-based constraints
#[derive(Debug)]
pub struct MinLengthConstraintComponent {
    min_length: u64,
}

impl GraphvizOutput for MinLengthConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        format!(
            "{} [label=\"MinLength: {}\"];",
            component_id.to_graphviz_id(),
            self.min_length
        )
    }
}

#[derive(Debug)]
pub struct MaxLengthConstraintComponent {
    max_length: u64,
}

impl GraphvizOutput for MaxLengthConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        format!(
            "{} [label=\"MaxLength: {}\"];",
            component_id.to_graphviz_id(),
            self.max_length
        )
    }
}

#[derive(Debug)]
pub struct PatternConstraintComponent {
    pattern: String,
    flags: Option<String>,
}

impl GraphvizOutput for PatternConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        let flags_str = self.flags.as_deref().unwrap_or("");
        format!(
            "{} [label=\"Pattern: {}\\nFlags: {}\"];",
            component_id.to_graphviz_id(),
            sanitize_graphviz_string(&self.pattern), // Pattern is a String, not a Term
            flags_str
        )
    }
}

#[derive(Debug)]
pub struct LanguageInConstraintComponent {
    languages: Vec<String>,
}

impl GraphvizOutput for LanguageInConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        format!(
            "{} [label=\"LanguageIn: [{}]\"];",
            component_id.to_graphviz_id(),
            self.languages.join(", ")
        )
    }
}

#[derive(Debug)]
pub struct UniqueLangConstraintComponent {
    unique_lang: bool,
}

impl GraphvizOutput for UniqueLangConstraintComponent {
    fn to_graphviz_string(
        &self,
        component_id: ComponentID,
        _context: &ValidationContext,
    ) -> String {
        format!(
            "{} [label=\"UniqueLang: {}\"];",
            component_id.to_graphviz_id(),
            self.unique_lang
        )
    }
}
