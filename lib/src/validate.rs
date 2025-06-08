use crate::context::{ValidationContext, Context};
use crate::report::ValidationReportBuilder;
use crate::types::{ID, ComponentID};
use crate::shape::{ValidateShape, NodeShape, PropertyShape};

impl ValidateShape for NodeShape {
    fn validate(
        &self,
        context: &ValidationContext,
        rb: &mut ValidationReportBuilder,
    ) -> Result<(), String> {
        println!("Validating NodeShape with identifier: {}", self.identifier());
        // first gather all of the targets
        println!("targets: {:?}", self.targets());
        let target_contexts = self
            .targets()
            .iter()
            .map(|t| t.get_target_nodes(context))
            .flatten();
        let target_contexts: Vec<_> = target_contexts.collect();

        if target_contexts.len() > 0 {
            println!("Targets: {:?}", target_contexts.len());
        }

        for target in target_contexts {
            // for each target, validate the constraints
            for constraint in self.constraints() {
                let comp = context
                    .get_component_by_id(constraint)
                    .ok_or_else(|| format!("Component not found: {}", constraint))?;
                if let Err(e) = comp.validate(&[&target], context, rb) {
                    rb.add_error(&target, e);
                }
            }
        }
        Ok(())
    }
}

impl PropertyShape {
    pub fn validate(
        &self,
        c: &[&Context],
        context: &ValidationContext,
        rb: &mut ValidationReportBuilder,
    ) -> Result<(), String> {
        // for each context, follow the path from the target_node
        // to get the set of value nodes.
        println!("Validating PropertyShape with identifier: {}", self.identifier());
        println!("Path: {:?}", self.path());
        for ctx in c {
            let mut value_nodes = Vec::new();
            // get value nodes from the query:
            // SELECT ?vn WHERE { <c.targeT_node> <self.path> ?vn }
            let query = format!(
                "SELECT ?vn WHERE {{ <{}> <{}> ?vn }}",
                ctx.focus_node(), self.sparql_path()
            );
        }

        Ok(())
    }
}
