use crate::context::ValidationContext;
use crate::types::Target;
use oxigraph::model::Term;
use oxigraph::sparql::{Query, QueryOptions, QueryResults};
use std::collections::HashSet;

/// A struct to hold statistics about the optimizations performed.
#[derive(Default, Debug)]
pub(crate) struct OptimizerStats {
    /// The number of `sh:targetClass` targets removed because the class has no instances in the data graph.
    pub(crate) unreachable_targets_removed: u64,
}

impl OptimizerStats {
    /// Creates a new `OptimizerStats` with all counters at zero.
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

/// A struct that holds the `ValidationContext` and performs optimizations on it.
pub(crate) struct Optimizer {
    /// The `ValidationContext` to be optimized.
    pub(crate) ctx: ValidationContext,
    /// Statistics collected during optimization.
    pub(crate) stats: OptimizerStats,
}

const TYPE_QUERY: &str = "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT DISTINCT ?type WHERE { ?s rdf:type/rdfs:subClassOf* ?type . }";

impl Optimizer {
    /// Creates a new `Optimizer` for a given `ValidationContext`.
    pub(crate) fn new(ctx: ValidationContext) -> Self {
        Optimizer {
            ctx,
            stats: OptimizerStats::new(),
        }
    }

    /// Runs all optimization passes.
    pub(crate) fn optimize(&mut self) -> Result<(), String> {
        // Remove unreachable targets from node shapes
        self.remove_unreachable_targets()?;
        Ok(())
    }

    /// Consumes the optimizer and returns the optimized `ValidationContext`.
    pub(crate) fn finish(self) -> ValidationContext {
        self.ctx
    }

    // Add methods for optimization logic here
    fn remove_unreachable_targets(&mut self) -> Result<(), String> {
        // run a query on  ctx.data_graph to figure out what types of things there are:
        // SELECT DISTINCT ?type WHERE { ?thing rdf:type/rdfs:subClassOf* ?type . }
        // make a hashset of these types
        // Then remove all TargetClasses from nodeshapes where their class does not exist in this
        // hashset.
        let mut parsed_query = Query::parse(TYPE_QUERY, None)
            .map_err(|e| format!("SPARQL parse error: {} {:?}", TYPE_QUERY, e))?;
        parsed_query
            .dataset_mut()
            .set_default_graph(vec![self.ctx.data_graph_iri.clone().into()]);

        let results = self
            .ctx
            .store()
            .query_opt(parsed_query, QueryOptions::default())
            .map_err(|e| e.to_string())?;

        let mut types = HashSet::<Term>::new();
        match results {
            QueryResults::Solutions(solutions) => {
                for solution_result in solutions {
                    let solution = solution_result.map_err(|e| e.to_string())?;
                    if let Some(term_ref) = solution.get("type") {
                        types.insert(term_ref.to_owned());
                    }
                }
            }
            _ => return Err("Unexpected query result type when fetching types".to_string()),
        }

        for shape in self.ctx.node_shapes.values_mut() {
            let targets_before = shape.targets.len();
            shape.targets.retain(|target| match target {
                Target::Class(class_term) => types.contains(&class_term),
                _ => true, // Keep other target types
            });
            let targets_after = shape.targets.len();
            self.stats.unreachable_targets_removed += (targets_before - targets_after) as u64;
        }

        Ok(())
    }
}
