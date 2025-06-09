use crate::context::{format_term_for_label, Context, ValidationContext};
use crate::types::{ComponentID, PropShapeID, ID};
use oxigraph::model::Term; // For Graphviz labels if term not found

use super::{GraphvizOutput, ValidateComponent, ComponentValidationResult, check_conformance_for_node};

#[derive(Debug)]
pub struct NodeConstraintComponent {
    shape: ID,
}

impl NodeConstraintComponent {
    pub fn new(shape: ID) -> Self {
        NodeConstraintComponent { shape }
    }
}

impl GraphvizOutput for NodeConstraintComponent {
    fn to_graphviz_string(&self, component_id: ComponentID, context: &ValidationContext) -> String {
        let shape_term_str = context
            .nodeshape_id_lookup()
            .borrow()
            .get_term(self.shape)
            .map_or_else(
                || format!("MissingNodeShape:{}", self.shape),
                |term| format_term_for_label(term),
            );
        let label = format!("NodeConstraint\\n({})", shape_term_str);
        format!(
            "{0} [label=\"{1}\"];\n    {0} -> {2} [style=dashed, label=\"validates\"];",
            component_id.to_graphviz_id(),
            label,
            self.shape.to_graphviz_id()
        )
    }
}

impl ValidateComponent for NodeConstraintComponent {
    fn validate(
        &self,
        component_id: ComponentID,
        c: &Context, // Context of the shape that has the sh:node constraint
        validation_context: &ValidationContext,
    ) -> Result<ComponentValidationResult, String> {
        let Some(value_nodes) = c.value_nodes() else {
            // No value nodes to check against the node constraint.
            return Ok(ComponentValidationResult::Pass(component_id));
        };

        let Some(target_node_shape) = validation_context.get_node_shape_by_id(&self.shape) else {
            return Err(format!(
                "sh:node referenced shape {:?} not found",
                self.shape
            ));
        };

        for value_node_to_check in value_nodes {
            // Create a new context where the current value_node is the focus node.
            // The path and other aspects of the original context 'c' are not directly relevant
            // for this specific conformance check of the value_node against target_node_shape.
            let value_node_as_context = Context::new(
                value_node_to_check.clone(),
                None, // Path is not directly relevant for this sub-check's context
                Some(vec![value_node_to_check.clone()]), // Value nodes for the sub-check
            );

            match check_conformance_for_node(
                &value_node_as_context,
                target_node_shape,
                validation_context,
            ) {
                Ok(true) => {
                    // value_node_to_check CONFORMS to the target_node_shape.
                    // This is the desired outcome for sh:node, so continue to the next value_node.
                }
                Ok(false) => {
                    // value_node_to_check DOES NOT CONFORM to the target_node_shape.
                    // This means the sh:node constraint FAILS for this value_node.
                    return Err(format!(
                        "Value {:?} does not conform to sh:node shape {:?}",
                        value_node_to_check, self.shape
                    ));
                }
                Err(e) => {
                    // An error occurred during the conformance check itself.
                    return Err(format!(
                        "Error checking conformance for sh:node shape {:?}: {}",
                        self.shape, e
                    ));
                }
            }
        }

        // All value_nodes successfully conformed to the target_node_shape.
        Ok(ComponentValidationResult::Pass(component_id))
    }
}

#[derive(Debug)]
pub struct PropertyConstraintComponent {
    shape: PropShapeID,
}

impl PropertyConstraintComponent {
    pub fn new(shape: PropShapeID) -> Self {
        PropertyConstraintComponent { shape }
    }

    pub fn shape(&self) -> &PropShapeID {
        &self.shape
    }
}

impl GraphvizOutput for PropertyConstraintComponent {
    fn to_graphviz_string(&self, component_id: ComponentID, context: &ValidationContext) -> String {
        let shape_term_str = context
            .propshape_id_lookup()
            .borrow()
            .get_term(self.shape)
            .map_or_else(
                || format!("MissingPropShape:{}", self.shape),
                |term| format_term_for_label(term),
            );
        let label = format!("PropertyConstraint\\n({})", shape_term_str);
        format!(
            "{0} [label=\"{1}\"];\n    {0} -> {2} [style=dashed, label=\"validates\"];",
            component_id.to_graphviz_id(),
            label,
            self.shape.to_graphviz_id()
        )
    }
}

impl ValidateComponent for PropertyConstraintComponent {
    fn validate(
        &self,
        component_id: ComponentID,
        _c: &Context, // Context may not be directly used here if PSS::validate is called elsewhere
        context: &ValidationContext, // May be used to check existence of self.shape
    ) -> Result<ComponentValidationResult, String> {
        // Ensure the referenced property shape exists.
        // The actual validation via PropertyShape::validate (which uses an RB)
        // is assumed to be handled by the caller of Component::validate (e.g. NodeShape::validate).
        // This component's validation, under the new trait, primarily confirms its own structural validity
        // or delegates checks that don't involve the RB directly.
        if context.get_prop_shape_by_id(&self.shape).is_none() {
            return Err(format!(
                "Referenced property shape not found for ID: {}",
                self.shape
            ));
        }
        Ok(ComponentValidationResult::Pass(component_id))
    }
}

#[derive(Debug)]
pub struct QualifiedValueShapeComponent {
    shape: ID, // This is a NodeShape ID
    min_count: Option<u64>,
    max_count: Option<u64>,
    disjoint: Option<bool>,
}

impl QualifiedValueShapeComponent {
    pub fn new(
        shape: ID,
        min_count: Option<u64>,
        max_count: Option<u64>,
        disjoint: Option<bool>,
    ) -> Self {
        QualifiedValueShapeComponent {
            shape,
            min_count,
            max_count,
            disjoint,
        }
    }
}

impl GraphvizOutput for QualifiedValueShapeComponent {
    fn to_graphviz_string(&self, component_id: ComponentID, context: &ValidationContext) -> String {
        let shape_term_str = context
            .nodeshape_id_lookup()
            .borrow()
            .get_term(self.shape)
            .map_or_else(
                || format!("MissingNodeShape:{}", self.shape),
                |term| format_term_for_label(term),
            );
        let mut label_parts = vec![format!("QualifiedValueShape\\nShape: {}", shape_term_str)];
        if let Some(min) = self.min_count {
            label_parts.push(format!("MinCount: {}", min));
        }
        if let Some(max) = self.max_count {
            label_parts.push(format!("MaxCount: {}", max));
        }
        if let Some(disjoint) = self.disjoint {
            label_parts.push(format!("Disjoint: {}", disjoint));
        }
        let label = label_parts.join("\\n");
        format!(
            "{0} [label=\"{1}\"];\n    {0} -> {2} [style=dashed, label=\"qualifies\"];",
            component_id.to_graphviz_id(),
            label,
            self.shape.to_graphviz_id()
        )
    }
}
