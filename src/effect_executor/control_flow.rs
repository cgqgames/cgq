use bevy::prelude::*;
use crate::effect::{EffectOperation, EffectContext, Value, Predicate};
use crate::game_state::GameState;
use crate::collections::CollectionManager;
use super::{EffectResult, EffectError, EffectExecutor};

/// Execute control flow operations (IfCondition, While, ForEach)
pub fn execute_control_flow_operation(
    executor: &mut EffectExecutor,
    operation: &EffectOperation,
    context: &mut EffectContext,
    state: &mut GameState,
    world: &mut World,
) -> Option<EffectResult> {
    match operation {
        EffectOperation::IfCondition { condition, then, else_ } => {
            Some(execute_if_condition(executor, condition, then, else_.as_ref(), context, state, world))
        }

        EffectOperation::While { condition, operations, max_iterations } => {
            Some(execute_while(executor, condition, operations, *max_iterations, context, state, world))
        }

        EffectOperation::ForEach { collection, operations } => {
            Some(execute_for_each(executor, collection, operations, context, state, world))
        }

        _ => None,
    }
}

fn execute_if_condition(
    executor: &mut EffectExecutor,
    condition: &Predicate,
    then: &[EffectOperation],
    else_: Option<&Vec<EffectOperation>>,
    context: &mut EffectContext,
    state: &mut GameState,
    world: &mut World,
) -> EffectResult {
    let condition_met = executor.evaluate_predicate(condition, context, state, world)?;

    if condition_met {
        for op in then {
            executor.execute_operation(op, context, state, world)?;
        }
    } else if let Some(else_ops) = else_ {
        for op in else_ops {
            executor.execute_operation(op, context, state, world)?;
        }
    }
    Ok(())
}

fn execute_while(
    executor: &mut EffectExecutor,
    condition: &Predicate,
    operations: &[EffectOperation],
    max_iterations: Option<usize>,
    context: &mut EffectContext,
    state: &mut GameState,
    world: &mut World,
) -> EffectResult {
    let max = max_iterations.unwrap_or(100);
    let mut iterations = 0;

    while executor.evaluate_predicate(condition, context, state, world)? {
        if iterations >= max {
            return Err(EffectError::MaxIterationsReached);
        }

        for op in operations {
            executor.execute_operation(op, context, state, world)?;
        }

        iterations += 1;
    }
    Ok(())
}

fn execute_for_each(
    executor: &mut EffectExecutor,
    collection_path: &str,
    operations: &[EffectOperation],
    context: &mut EffectContext,
    state: &mut GameState,
    world: &mut World,
) -> EffectResult {
    // Get collection items (clone to avoid borrow conflicts)
    let items = if let Some(collections) = world.get_resource::<CollectionManager>() {
        if let Some(collection) = collections.get(collection_path) {
            collection.to_vec()
        } else {
            warn!("Collection not found: {}", collection_path);
            Vec::new()
        }
    } else {
        return Err(EffectError::OperationError("CollectionManager not available".to_string()));
    };

    // Execute operations for each item
    for (index, item) in items.iter().enumerate() {
        // Set iteration variables
        context.set_variable("item".to_string(), item.clone());
        context.set_variable("index".to_string(), Value::Int(index as i32));

        // Execute operations
        for op in operations {
            executor.execute_operation(op, context, state, world)?;
        }
    }
    Ok(())
}
