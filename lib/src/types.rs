use crate::context::{Context, SourceShape, ValidationContext};
use crate::named_nodes::SHACL;
use oxigraph::model::{NamedNodeRef, Term, TermRef, Variable};
use oxigraph::sparql::{Query, QueryOptions, QueryResults}; // Added Query
use std::fmt; // Added for Display trait
use std::hash::Hash; // Added Hash for derived traits

/// A unique identifier for a `Term` within the validation context, derived from its hash.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TermID(pub u64);

/// A unique identifier for a `NodeShape`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ID(pub u64);

impl From<u64> for ID {
    fn from(item: u64) -> Self {
        ID(item)
    }
}

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ID {
    /// Converts the ID to a string suitable for use as a node identifier in Graphviz.
    pub fn to_graphviz_id(&self) -> String {
        format!("n{}", self.0)
    }
}

/// A unique identifier for a constraint `Component`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ComponentID(pub u64);

impl From<u64> for ComponentID {
    fn from(item: u64) -> Self {
        ComponentID(item)
    }
}

impl fmt::Display for ComponentID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ComponentID {
    /// Converts the ComponentID to a string suitable for use as a node identifier in Graphviz.
    pub fn to_graphviz_id(&self) -> String {
        format!("c{}", self.0)
    }
    /// Retrieves the name (Term) of the component from the validation context.
    pub fn name(&self, context: &ValidationContext) -> String {
        context
            .component_id_lookup
            .borrow()
            .get_term(*self)
            .unwrap()
            .to_string()
    }
}

/// A unique identifier for a `PropertyShape`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PropShapeID(pub u64);

impl From<u64> for PropShapeID {
    fn from(item: u64) -> Self {
        PropShapeID(item)
    }
}

impl fmt::Display for PropShapeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PropShapeID {
    /// Converts the PropShapeID to a string suitable for use as a node identifier in Graphviz.
    pub fn to_graphviz_id(&self) -> String {
        format!("p{}", self.0)
    }
}

/// Represents a SHACL Property Path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Path {
    /// A simple path, which is a single IRI.
    Simple(Term),
    /// An inverse path (`sh:inversePath`).
    Inverse(Box<Path>),
    /// A sequence of paths (`sh:sequencePath`).
    Sequence(Vec<Path>),
    /// A set of alternative paths (`sh:alternativePath`).
    Alternative(Vec<Path>),
    /// A path that can be traversed zero or more times (`sh:zeroOrMorePath`).
    ZeroOrMore(Box<Path>),
    /// A path that can be traversed one or more times (`sh:oneOrMorePath`).
    OneOrMore(Box<Path>),
    /// A path that can be traversed zero or one time (`sh:zeroOrOnePath`).
    ZeroOrOne(Box<Path>),
}

impl Path {
    // Helper to format a NamedNode for SPARQL path
    fn format_named_node_for_sparql(nn: &oxigraph::model::NamedNode) -> String {
        format!("<{}>", nn.as_str())
    }

    /// Converts the SHACL path to its SPARQL 1.1 property path string representation.
    pub fn to_sparql_path(&self) -> Result<String, String> {
        match self {
            Path::Simple(term) => match term {
                Term::NamedNode(nn) => Ok(Self::format_named_node_for_sparql(nn)),
                _ => Err(format!("Simple path must be an IRI {:?}", self).to_string()),
            },
            Path::Inverse(inner_path) => {
                let inner_sparql = inner_path.to_sparql_path()?;
                // Based on SPARQL grammar, if inner_sparql is already a (Path) or an IRIref,
                // it doesn't need more parentheses for ^.
                // Our Sequence and Alternative variants return parenthesized strings.
                // Simple returns an IRIref.
                Ok(format!("^{}", inner_sparql))
            }
            Path::Sequence(paths) => {
                if paths.len() < 2 {
                    // SHACL specification: sequence path is a list of at least two members.
                    // If parsing allowed a single path, it should be simplified upstream.
                    // For now, error if not adhering to the typical structure.
                    return Err(format!(
                        "Sequence path must have at least two elements, found {}",
                        paths.len()
                    ));
                }
                let mut sparql_paths = Vec::new();
                for p in paths {
                    sparql_paths.push(p.to_sparql_path()?);
                }
                let result = sparql_paths.join(" / ");
                Ok(format!("({})", result)) // Always parenthesize sequence for clarity and safety.
            }
            Path::Alternative(paths) => {
                if paths.len() < 2 {
                    // SHACL specification: alternative path is a list of at least two members.
                    return Err(format!(
                        "Alternative path must have at least two elements, found {}",
                        paths.len()
                    ));
                }
                let mut sparql_paths = Vec::new();
                for p in paths {
                    sparql_paths.push(p.to_sparql_path()?);
                }
                let result = sparql_paths.join(" | ");
                Ok(format!("({})", result)) // Always parenthesize alternative for clarity and safety.
            }
            Path::ZeroOrMore(inner_path) => {
                let inner_sparql = inner_path.to_sparql_path()?;
                Ok(format!("{}*", inner_sparql))
            }
            Path::OneOrMore(inner_path) => {
                let inner_sparql = inner_path.to_sparql_path()?;
                Ok(format!("{}+", inner_sparql))
            }
            Path::ZeroOrOne(inner_path) => {
                let inner_sparql = inner_path.to_sparql_path()?;
                Ok(format!("{}?", inner_sparql))
            }
        }
    }
}

/// Represents a SHACL target, which specifies the nodes to be validated against a shape.
#[derive(Debug)]
pub enum Target {
    /// Targets all instances of a given class (`sh:targetClass`).
    Class(Term),
    /// Targets a specific node (`sh:targetNode`).
    Node(Term),
    /// Targets all subjects of triples with a given predicate (`sh:targetSubjectsOf`).
    SubjectsOf(Term),
    /// Targets all objects of triples with a given predicate (`sh:targetObjectsOf`).
    ObjectsOf(Term),
}

/// Represents the severity level of a validation result, corresponding to `sh:severity`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Corresponds to `sh:Info`.
    Info,
    /// Corresponds to `sh:Warning`.
    Warning,
    /// Corresponds to `sh:Violation`.
    Violation,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Violation
    }
}

impl Severity {
    /// Creates a `Severity` from a `Term` if it matches a SHACL severity IRI.
    pub fn from_term(term: &Term) -> Option<Self> {
        let shacl = SHACL::new();
        if let Term::NamedNode(nn) = term {
            if *nn == shacl.info {
                Some(Severity::Info)
            } else if *nn == shacl.warning {
                Some(Severity::Warning)
            } else if *nn == shacl.violation {
                Some(Severity::Violation)
            } else {
                None // Or default to Violation if an unknown IRI is used?
                     // For now, strictly None if not recognized.
            }
        } else {
            None
        }
    }
}

impl Target {
    /// Creates a `Target` from a predicate and object `TermRef` if they correspond to a known SHACL target property.
    pub fn from_predicate_object(predicate: NamedNodeRef, object: TermRef) -> Option<Self> {
        let shacl = SHACL::new();
        if predicate == shacl.target_class {
            Some(Target::Class(object.into_owned()))
        } else if predicate == shacl.target_node {
            Some(Target::Node(object.into_owned()))
        } else if predicate == shacl.target_subjects_of {
            Some(Target::SubjectsOf(object.into_owned()))
        } else if predicate == shacl.target_objects_of {
            Some(Target::ObjectsOf(object.into_owned()))
        } else {
            None
        }
    }

    /// Retrieves the set of focus nodes for this target from the data graph.
    pub fn get_target_nodes(
        &self,
        context: &ValidationContext,
        source_shape_id: ID,
    ) -> Result<Vec<Context>, String> {
        match self {
            Target::Node(t) => Ok(vec![Context::new(
                t.clone(),
                None,
                Some(vec![t.clone()]),
                SourceShape::NodeShape(source_shape_id),
            )]),
            Target::Class(c) => {
                let query_str = "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
                    SELECT DISTINCT ?inst ?target_class WHERE { ?inst rdf:type ?c . ?c rdfs:subClassOf* ?target_class }";
                let target_class_var = Variable::new("target_class").map_err(|e| e.to_string())?;

                let mut parsed_query = Query::parse(query_str, None).map_err(|e| {
                    format!(
                        "SPARQL parse error for Target::Class: {} {:?}",
                        query_str, e
                    )
                })?;
                parsed_query.dataset_mut().set_default_graph_as_union();

                let results = context
                    .store()
                    .query_opt_with_substituted_variables(
                        parsed_query,
                        QueryOptions::default(),
                        [(target_class_var, c.clone())],
                    )
                    .map_err(|e| {
                        format!(
                            "SPARQL query error for Target::Class: {} {:?}",
                            query_str, e
                        )
                    })?;

                match results {
                    QueryResults::Solutions(solutions) => solutions
                        .map(|solution_result| {
                            let solution = solution_result.map_err(|e| e.to_string())?;
                            solution
                                .get("inst")
                                .map(|term_ref| {
                                    Context::new(
                                        term_ref.to_owned(),
                                        None,
                                        Some(vec![term_ref.clone()]),
                                        SourceShape::NodeShape(source_shape_id),
                                    )
                                })
                                .ok_or_else(|| {
                                    "Variable 'inst' not found in Target::Class query solution"
                                        .to_string()
                                })
                        })
                        .collect(),
                    _ => Err(format!(
                        "Unexpected result type for Target::Class: {}",
                        query_str
                    )),
                }
            }
            Target::SubjectsOf(p) => {
                if let Term::NamedNode(predicate_node) = p {
                    let query_str = format!(
                        "SELECT DISTINCT ?s WHERE {{ ?s <{}> ?any . }}",
                        predicate_node.as_str()
                    );
                    let mut parsed_query = Query::parse(&query_str, None).map_err(|e| {
                        format!(
                            "SPARQL parse error for Target::SubjectsOf: {} {:?}",
                            query_str, e
                        )
                    })?;
                    parsed_query.dataset_mut().set_default_graph_as_union();

                    let results = context
                        .store()
                        .query_opt(parsed_query, QueryOptions::default())
                        .map_err(|e| e.to_string())?;

                    match results {
                        QueryResults::Solutions(solutions) => solutions
                            .map(|solution_result| {
                                let solution = solution_result.map_err(|e| e.to_string())?;
                                solution
                                    .get("s")
                                    .map(|term_ref| {
                                        Context::new(
                                            term_ref.to_owned(),
                                            None,
                                            Some(vec![term_ref.clone()]),
                                            SourceShape::NodeShape(source_shape_id)
                                        )
                                    })
                                    .ok_or_else(|| {
                                        "Variable 's' not found in Target::SubjectsOf query solution"
                                            .to_string()
                                    })
                            })
                            .collect(),
                        _ => Err("Unexpected result type for Target::SubjectsOf query".to_string()),
                    }
                } else {
                    Ok(vec![]) // Predicate for SubjectsOf must be an IRI
                }
            }
            Target::ObjectsOf(p) => {
                if let Term::NamedNode(predicate_node) = p {
                    let query_str = format!(
                        "SELECT DISTINCT ?o WHERE {{ ?any <{}> ?o . }}",
                        predicate_node.as_str()
                    );
                    let mut parsed_query = Query::parse(&query_str, None).map_err(|e| {
                        format!(
                            "SPARQL parse error for Target::ObjectsOf: {} {:?}",
                            query_str, e
                        )
                    })?;
                    parsed_query.dataset_mut().set_default_graph_as_union();

                    let results = context
                        .store()
                        .query_opt(parsed_query, QueryOptions::default())
                        .map_err(|e| e.to_string())?;

                    match results {
                        QueryResults::Solutions(solutions) => solutions
                            .map(|solution_result| {
                                let solution = solution_result.map_err(|e| e.to_string())?;
                                solution
                                    .get("o")
                                    .map(|term_ref| {
                                        Context::new(
                                            term_ref.to_owned(),
                                            None,
                                            Some(vec![term_ref.clone()]),
                                            SourceShape::NodeShape(source_shape_id),
                                        )
                                    })
                                    .ok_or_else(|| {
                                        "Variable 'o' not found in Target::ObjectsOf query solution"
                                            .to_string()
                                    })
                            })
                            .collect(),
                        _ => Err("Unexpected result type for Target::ObjectsOf query".to_string()),
                    }
                } else {
                    Ok(vec![]) // Predicate for ObjectsOf must be an IRI
                }
            }
        }
    }
}
