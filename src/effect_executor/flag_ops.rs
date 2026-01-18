use bevy::prelude::*;
use crate::effect::EffectOperation;
use crate::game_state::GameState;
use super::{EffectResult, EffectError};

/// Execute flag operations (SetFlag, ToggleFlag)
pub fn execute_flag_operation(
    operation: &EffectOperation,
    state: &mut GameState,
    world: &mut World,
) -> Option<EffectResult> {
    match operation {
        EffectOperation::SetFlag { flag, value } => {
            if !state.set_flag(flag, *value, world) {
                return Some(Err(EffectError::InvalidPath(flag.clone())));
            }
            Some(Ok(()))
        }

        EffectOperation::ToggleFlag { flag } => {
            if !state.toggle_flag(flag, world) {
                return Some(Err(EffectError::InvalidPath(flag.clone())));
            }
            Some(Ok(()))
        }

        _ => None,
    }
}
