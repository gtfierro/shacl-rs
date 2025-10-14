//! A SHACL validator library.
#![deny(clippy::all)]

// Publicly visible items
pub mod model;
pub mod shape;
pub mod types;

pub use report::ValidationReport;

// Internal modules.
pub mod canonicalization;
pub(crate) mod context;
pub(crate) mod named_nodes;
pub(crate) mod optimize;
pub(crate) mod parser;
pub(crate) mod report;
pub(crate) mod runtime;
pub mod test_utils; // Often pub for integration tests
pub(crate) mod validate;

use crate::canonicalization::skolemize;
use crate::context::{ParsingContext, ShapesModel, ValidationContext};
use crate::model::components::ComponentDescriptor;
use crate::optimize::Optimizer;
use crate::parser as shacl_parser;
use log::{debug, info};
use ontoenv::api::OntoEnv;
use ontoenv::config::Config;
use ontoenv::ontology::OntologyLocation;
use ontoenv::options::{Overwrite, RefreshStrategy};
use oxigraph::model::GraphNameRef;
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;

/// Represents the source of shapes or data, which can be either a local file or a named graph from an `OntoEnv`.
#[derive(Debug)]
pub enum Source {
    /// A local file path.
    File(PathBuf),
    /// The URI of a named graph.
    Graph(String),
}

/// A simple facade for the SHACL validator.
///
/// This provides a straightforward interface for common validation tasks.
/// It handles the creation of a `ValidationContext`, parsing of shapes and data,
/// running the validation, and generating reports.
///
/// For more advanced control, such as inspecting the parsed shapes or performing
/// optimizations, use `ValidationContext` directly.
pub struct Validator {
    context: ValidationContext,
}

impl Validator {
    /// Creates a new Validator from local files.
    ///
    /// This method initializes the underlying `ValidationContext` by loading data from files.
    ///
    /// # Arguments
    ///
    /// * `shape_graph_path` - The file path for the SHACL shapes.
    /// * `data_graph_path` - The file path for the data to be validated.
    pub fn from_files(
        shape_graph_path: &str,
        data_graph_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        Self::from_sources(
            Source::File(PathBuf::from(shape_graph_path)),
            Source::File(PathBuf::from(data_graph_path)),
        )
    }

    /// Creates a new Validator from the given shapes and data sources.
    ///
    /// This method initializes the underlying `ValidationContext`, loading data from files
    /// or an `OntoEnv` as specified.
    ///
    /// # Arguments
    ///
    /// * `shapes_source` - The source for the SHACL shapes.
    /// * `data_source` - The source for the data to be validated.
    pub fn from_sources(
        shapes_source: Source,
        data_source: Source,
    ) -> Result<Self, Box<dyn Error>> {
        let config = Config::builder()
            .root(std::env::current_dir()?)
            .offline(true)
            .no_search(true)
            .temporary(true)
            .build()?;
        let mut env: OntoEnv = OntoEnv::init(config, false)?;

        let shapes_graph_id = match shapes_source {
            Source::Graph(uri) => env.add(
                OntologyLocation::Url(uri.clone()),
                Overwrite::Allow,
                RefreshStrategy::Force,
            )?,
            Source::File(path) => env.add(
                OntologyLocation::File(path.clone()),
                Overwrite::Allow,
                RefreshStrategy::Force,
            )?,
        };
        let shape_ontology = env.get_ontology(&shapes_graph_id).unwrap().clone();
        let shape_graph_iri = shape_ontology.name().clone();
        eprintln!(
            "Loaded shapes graph {} (location {})",
            shape_graph_iri,
            shape_ontology
                .location()
                .map(|loc| loc.as_str().to_string())
                .unwrap_or_else(|| "<unknown>".into())
        );
        info!("Added shape graph: {}", shape_graph_iri);

        let data_graph_id = match data_source {
            Source::Graph(uri) => env.add(
                OntologyLocation::Url(uri.clone()),
                Overwrite::Allow,
                RefreshStrategy::Force,
            )?,
            Source::File(path) => env.add(
                OntologyLocation::File(path.clone()),
                Overwrite::Allow,
                RefreshStrategy::Force,
            )?,
        };
        let data_ontology = env.get_ontology(&data_graph_id).unwrap().clone();
        let data_graph_iri = data_ontology.name().clone();
        eprintln!(
            "Loaded data graph {} (location {})",
            data_graph_iri,
            data_ontology
                .location()
                .map(|loc| loc.as_str().to_string())
                .unwrap_or_else(|| "<unknown>".into())
        );
        info!("Added data graph: {}", data_graph_iri);

        let store = env.io().store().clone();

        let shape_graph_base_iri = format!(
            "{}/.well-known/skolem/",
            shape_graph_iri.as_str().trim_end_matches('/')
        );
        info!(
            "Skolemizing shape graph <{}> with base IRI <{}>",
            shape_graph_iri, shape_graph_base_iri
        );
        // TODO: The skolemization is necessary for now because of some behavior in oxigraph where
        // blank nodes in SPARQL queries will match *any* blank node in the graph, rather than
        // just the blank node with the same "identifier". This makes it difficult to compose
        // queries that find the propertyshape value nodes when the focus node is a blank node.
        // This means the query ends up looking like:
        //    SELECT ?valuenode WHERE { _:focusnode <http://example.org/path> ?valuenode . }
        // The _:focusnode will match any blank node in the graph, which is not what we want.
        // This pops up in test cases like property_and_001
        skolemize(
            &store,
            GraphNameRef::NamedNode(shape_graph_iri.as_ref()),
            &shape_graph_base_iri,
        )?;

        let data_graph_base_iri = format!(
            "{}/.well-known/skolem/",
            data_graph_iri.as_str().trim_end_matches('/')
        );
        info!(
            "Skolemizing data graph <{}> with base IRI <{}>",
            data_graph_iri, data_graph_base_iri
        );
        skolemize(
            &store,
            GraphNameRef::NamedNode(data_graph_iri.as_ref()),
            &data_graph_base_iri,
        )?;

        info!(
            "Optimizing store with shape graph <{}> and data graph <{}>",
            shape_graph_iri, data_graph_iri
        );
        store.optimize().map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error optimizing store: {}", e),
            ))
        })?;

        let mut parsing_context =
            ParsingContext::new(store, env, shape_graph_iri, data_graph_iri.clone());

        shacl_parser::run_parser(&mut parsing_context)?;
        {
            debug!("prop_shapes count: {}", parsing_context.prop_shapes.len());
            let props_lookup = parsing_context.propshape_id_lookup.borrow();
            for (id, shape) in parsing_context.prop_shapes.iter() {
                if let Some(term) = props_lookup.get_term(*id) {
                    debug!("prop_shape {:?} term {}", id, term);
                }
                debug!("  constraints: {:?}", shape.constraints());
                for cid in shape.constraints() {
                    if let Some(descriptor) = parsing_context.component_descriptors.get(cid) {
                        match descriptor {
                            ComponentDescriptor::NodeKind { node_kind } => {
                                debug!("    component {:?} node_kind {}", cid, node_kind);
                            }
                            other => {
                                debug!("    component {:?} {:?}", cid, other);
                            }
                        }
                    }
                }
            }
        }

        info!("Optimizing shape graph");
        let mut o = Optimizer::new(parsing_context);
        o.optimize()?;
        info!("Finished parsing shapes and optimizing context");
        let final_ctx = o.finish();

        let model = ShapesModel {
            nodeshape_id_lookup: final_ctx.nodeshape_id_lookup,
            propshape_id_lookup: final_ctx.propshape_id_lookup,
            component_id_lookup: final_ctx.component_id_lookup,
            store: final_ctx.store,
            shape_graph_iri: final_ctx.shape_graph_iri,
            node_shapes: final_ctx.node_shapes,
            prop_shapes: final_ctx.prop_shapes,
            component_descriptors: final_ctx.component_descriptors,
            env: final_ctx.env,
        };

        let context = ValidationContext::new(Rc::new(model), data_graph_iri);

        Ok(Validator { context })
    }

    /// Validates the data graph against the shapes graph.
    ///
    /// This method executes the core validation logic and returns a `ValidationReport`.
    /// The report contains the outcome of the validation (conformity) and detailed
    /// results for any failures. The returned report is tied to the lifetime of the Validator.
    pub fn validate(&self) -> ValidationReport<'_> {
        let report_builder = validate::validate(&self.context);
        // The report needs the context to be able to serialize itself later.
        ValidationReport::new(report_builder.unwrap(), &self.context)
    }

    /// Generates a Graphviz DOT string representation of the shapes.
    ///
    /// This can be used to visualize the structure of the SHACL shapes, including
    /// their constraints and relationships.
    pub fn to_graphviz(&self) -> Result<String, String> {
        self.context.model.graphviz()
    }

    /// Generates a Graphviz DOT string representation of the shapes, with nodes colored by execution frequency.
    ///
    /// This can be used to visualize which parts of the shapes graph were most active during validation.
    /// Note: `validate()` must be called before this method to populate the execution traces.
    pub fn to_graphviz_heatmap(&self, include_all_nodes: bool) -> Result<String, String> {
        self.context.graphviz_heatmap(include_all_nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::named_nodes::SHACL;
    use oxigraph::model::vocab::rdf;
    use oxigraph::model::{NamedOrBlankNode, Term, TermRef};
    use std::error::Error;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
        let mut dir = std::env::temp_dir();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        dir.push(format!("{}_{}", prefix, timestamp));
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    #[test]
    fn custom_sparql_message_and_severity() -> Result<(), Box<dyn Error>> {
        let temp_dir = unique_temp_dir("shacl_message_test")?;

        let shapes_ttl = r#"@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example.com/ns#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:MinScoreConstraintComponent
    a sh:ConstraintComponent ;
    sh:parameter [
        sh:path ex:minScore ;
        sh:optional false
    ] ;
    sh:propertyValidator [
        sh:message "Score must be at least {?minScore} (got {?value})."@en ;
        sh:select """
            PREFIX ex: <http://example.com/ns#>
            PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
            SELECT ?this ?value ?minScore
            WHERE {
                ?this ex:score ?value .
                FILTER(xsd:integer(?value) < xsd:integer(?minScore))
            }
        """ ;
        sh:severity sh:Warning ;
    ] ;
    sh:message "Default {?minScore}"@en ;
    sh:severity sh:Info .

ex:ScoreShape
    a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [
        sh:path ex:score ;
        ex:minScore 5 ;
    ] .
"#;

        let data_ttl = r#"@prefix ex: <http://example.com/ns#> .

ex:Alice a ex:Person ;
    ex:score 3 .
"#;

        let shapes_path = temp_dir.join("shapes.ttl");
        let data_path = temp_dir.join("data.ttl");

        {
            let mut file = fs::File::create(&shapes_path)?;
            file.write_all(shapes_ttl.as_bytes())?;
        }
        {
            let mut file = fs::File::create(&data_path)?;
            file.write_all(data_ttl.as_bytes())?;
        }

        let shapes_path_str = shapes_path.to_string_lossy().to_string();
        let data_path_str = data_path.to_string_lossy().to_string();

        let validator = Validator::from_files(&shapes_path_str, &data_path_str)?;
        let report = validator.validate();
        assert!(!report.conforms(), "Expected validation to fail");

        let graph = report.to_graph();
        let sh = SHACL::new();

        let mut result_nodes: Vec<NamedOrBlankNode> = Vec::new();
        for triple in graph.iter() {
            if triple.predicate == rdf::TYPE
                && triple.object == TermRef::NamedNode(sh.validation_result)
            {
                result_nodes.push(triple.subject.into_owned());
            }
        }

        assert_eq!(
            result_nodes.len(),
            1,
            "Expected exactly one validation result"
        );
        let result_subject = result_nodes[0].as_ref();

        let severity_terms: Vec<Term> = graph
            .objects_for_subject_predicate(result_subject, sh.result_severity)
            .map(|t| t.into_owned())
            .collect();
        assert_eq!(
            severity_terms,
            vec![Term::from(sh.warning)],
            "Severity should inherit sh:Warning from the validator"
        );

        let message_terms: Vec<Term> = graph
            .objects_for_subject_predicate(result_subject, sh.result_message)
            .map(|t| t.into_owned())
            .collect();
        assert!(
            message_terms.iter().any(|term| {
                if let Term::Literal(lit) = term {
                    lit.value() == "Score must be at least 5 (got 3)."
                        && matches!(lit.language(), Some(lang) if lang == "en")
                } else {
                    false
                }
            }),
            "Expected substituted result message literal"
        );

        fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }
}
