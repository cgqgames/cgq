use bevy::prelude::*;
use crate::effect::EffectOperation;
use crate::game_state::GameState;
use super::{EffectResult, EffectError};

/// Execute value modification operations (Add, Subtract, Multiply, Set)
pub fn execute_value_operation(
    operation: &EffectOperation,
    state: &mut GameState,
    world: &mut World,
) -> Option<EffectResult> {
    match operation {
        EffectOperation::Add { target, amount } => {
            if !state.add(target, *amount, world) {
                return Some(Err(EffectError::InvalidPath(target.clone())));
            }
            Some(Ok(()))
        }

        EffectOperation::Subtract { target, amount } => {
            if !state.subtract(target, *amount, world) {
                return Some(Err(EffectError::InvalidPath(target.clone())));
            }
            Some(Ok(()))
        }

        EffectOperation::Multiply { target, factor } => {
            if !state.multiply(target, *factor, world) {
                return Some(Err(EffectError::InvalidPath(target.clone())));
            }
            Some(Ok(()))
        }

        EffectOperation::Set { target, value } => {
            if !state.set(target, value.clone(), world) {
                return Some(Err(EffectError::InvalidPath(target.clone())));
            }
            Some(Ok(()))
        }

        _ => None,
    }
}
