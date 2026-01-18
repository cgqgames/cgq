use bevy::prelude::*;
use crate::effect::{EffectOperation, EffectContext};
use super::EffectResult;

/// Execute variable operations (SetVariable, GetVariable)
pub fn execute_variable_operation(
    operation: &EffectOperation,
    context: &mut EffectContext,
) -> Option<EffectResult> {
    match operation {
        EffectOperation::SetVariable { name, value } => {
            context.set_variable(name.clone(), value.clone());
            Some(Ok(()))
        }

        EffectOperation::GetVariable { name } => {
            // Variable retrieval happens during value resolution
            // This operation is mainly for explicit documentation
            if context.get_variable(name).is_none() {
                warn!("Variable '{}' not found in context", name);
            }
            Some(Ok(()))
        }

        _ => None,
    }
}
