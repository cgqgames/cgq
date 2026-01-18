use bevy::prelude::*;
use crate::effect::{EffectOperation, EffectContext, Value};
use crate::game_state::GameState;
use super::{EffectResult, EffectExecutor};
use std::collections::HashMap;

/// Execute event-related operations (OnEvent, EmitEvent, ScheduleOperation)
pub fn execute_event_operation(
    executor: &mut EffectExecutor,
    operation: &EffectOperation,
    context: &mut EffectContext,
    state: &mut GameState,
    world: &mut World,
) -> Option<EffectResult> {
    match operation {
        EffectOperation::OnEvent { event, operations } => {
            executor.register_event_listener(event.clone(), operations.clone());
            Some(Ok(()))
        }

        EffectOperation::EmitEvent { event, data } => {
            Some(execute_emit_event(executor, event, data.as_ref(), context, state, world))
        }

        EffectOperation::ScheduleOperation { delay_seconds: _, operations: _ } => {
            warn!("ScheduleOperation not yet implemented");
            Some(Ok(()))
        }

        _ => None,
    }
}

fn execute_emit_event(
    executor: &mut EffectExecutor,
    event: &str,
    data: Option<&HashMap<String, Value>>,
    context: &mut EffectContext,
    state: &mut GameState,
    world: &mut World,
) -> EffectResult {
    // Get listeners for this event
    if let Some(listeners) = executor.get_event_listeners(event) {
        // Update context with event data
        if let Some(event_data) = data {
            for (key, value) in event_data {
                context.set_variable(format!("event.{}", key), value.clone());
            }
        }

        // Execute all listeners
        for operations in listeners {
            for op in &operations {
                executor.execute_operation(op, context, state, world)?;
            }
        }
    }
    Ok(())
}
