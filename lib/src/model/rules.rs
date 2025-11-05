use crate::types::{Path, RuleID, ID};
use oxigraph::model::{NamedNode, Term};

/// Ordering value for rules (mirrors `sh:order`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuleOrder(pub f64);

/// Enumerates the supported rule kinds.
#[derive(Debug, Clone)]
pub enum Rule {
    Sparql(SparqlRule),
    Triple(TripleRule),
}

impl Rule {
    pub fn id(&self) -> RuleID {
        match self {
            Rule::Sparql(rule) => rule.id,
            Rule::Triple(rule) => rule.id,
        }
    }

    pub fn order(&self) -> Option<RuleOrder> {
        match self {
            Rule::Sparql(rule) => rule.order,
            Rule::Triple(rule) => rule.order,
        }
    }

    pub fn is_deactivated(&self) -> bool {
        match self {
            Rule::Sparql(rule) => rule.deactivated,
            Rule::Triple(rule) => rule.deactivated,
        }
    }
}

/// Details for a SPARQL rule.
#[derive(Debug, Clone)]
pub struct SparqlRule {
    pub id: RuleID,
    pub query: String,
    pub source_term: Term,
    pub condition_shapes: Vec<RuleCondition>,
    pub deactivated: bool,
    pub order: Option<RuleOrder>,
}

/// Details for a triple rule.
#[derive(Debug, Clone)]
pub struct TripleRule {
    pub id: RuleID,
    pub subject: TriplePatternTerm,
    pub predicate: NamedNode,
    pub object: TriplePatternTerm,
    pub condition_shapes: Vec<RuleCondition>,
    pub deactivated: bool,
    pub order: Option<RuleOrder>,
    pub source_term: Term,
}

/// Represents a rule condition that must be satisfied before the rule fires.
#[derive(Debug, Clone)]
pub enum RuleCondition {
    NodeShape(ID),
}

/// Represents a term template used by triple rules.
#[derive(Debug, Clone)]
pub enum TriplePatternTerm {
    This,
    Constant(Term),
    Path(Path),
}

impl TriplePatternTerm {
    pub fn is_this(&self) -> bool {
        matches!(self, TriplePatternTerm::This)
    }
}
