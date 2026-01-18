use bevy::prelude::*;
use crate::effect::{EffectOperation, EffectContext, Value};
use crate::collections::{CollectionManager, evaluate_item_predicate};
use super::{EffectResult, EffectError};

/// Execute collection operations (Filter, Remove, Append, Insert)
pub fn execute_collection_operation(
    operation: &EffectOperation,
    context: &mut EffectContext,
    world: &mut World,
) -> Option<EffectResult> {
    match operation {
        EffectOperation::Filter { target, predicate } => {
            Some(execute_filter(target, predicate, world))
        }

        EffectOperation::Remove { target, count, filter, random } => {
            Some(execute_remove(target, *count, filter.as_ref(), *random, context, world))
        }

        EffectOperation::Append { target, item } => {
            Some(execute_append(target, item, world))
        }

        EffectOperation::Insert { target, index, item } => {
            Some(execute_insert(target, *index, item, world))
        }

        _ => None,
    }
}

fn execute_filter(
    target: &str,
    predicate: &crate::effect::Predicate,
    world: &mut World,
) -> EffectResult {
    if let Some(mut collections) = world.get_resource_mut::<CollectionManager>() {
        if let Some(collection) = collections.get_mut(target) {
            collection.filter(|item| evaluate_item_predicate(predicate, item));
        } else {
            warn!("Collection not found: {}", target);
        }
        Ok(())
    } else {
        Err(EffectError::OperationError("CollectionManager not available".to_string()))
    }
}

fn execute_remove(
    target: &str,
    count: usize,
    filter: Option<&crate::effect::Predicate>,
    random: Option<bool>,
    context: &mut EffectContext,
    world: &mut World,
) -> EffectResult {
    if let Some(mut collections) = world.get_resource_mut::<CollectionManager>() {
        if let Some(collection) = collections.get_mut(target) {
            // Apply filter first if provided
            if let Some(pred) = filter {
                collection.filter(|item| evaluate_item_predicate(pred, item));
            }

            // Remove items
            let removed = collection.remove(count, random.unwrap_or(false));

            // Store removed items in context
            if removed.len() == 1 {
                context.set_variable("removed".to_string(), removed[0].clone());
            } else {
                context.set_variable("removed".to_string(), Value::Array(removed));
            }
        } else {
            warn!("Collection not found: {}", target);
        }
        Ok(())
    } else {
        Err(EffectError::OperationError("CollectionManager not available".to_string()))
    }
}

fn execute_append(
    target: &str,
    item: &Value,
    world: &mut World,
) -> EffectResult {
    if let Some(mut collections) = world.get_resource_mut::<CollectionManager>() {
        let collection = collections.get_or_create(target);
        collection.append(item.clone());
        Ok(())
    } else {
        Err(EffectError::OperationError("CollectionManager not available".to_string()))
    }
}

fn execute_insert(
    target: &str,
    index: usize,
    item: &Value,
    world: &mut World,
) -> EffectResult {
    if let Some(mut collections) = world.get_resource_mut::<CollectionManager>() {
        let collection = collections.get_or_create(target);
        collection.insert(index, item.clone());
        Ok(())
    } else {
        Err(EffectError::OperationError("CollectionManager not available".to_string()))
    }
}
