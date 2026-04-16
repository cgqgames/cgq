//! Bridges `CardManager::deployed_card_ids` to the `EffectExecutor`.
//!
//! Each frame, any card ID newly appended to `deployed_card_ids` has its
//! effects expanded and executed against the live game state. Deployment is
//! triggered by chat votes or local keybinds — both paths feed the same list.

use bevy::prelude::*;

use crate::collections::{Collection, CollectionManager};
use crate::components::{ActiveQuestion, Question, QuestionOption};
use crate::effect::{EffectContext, Value};
use crate::effect_executor::EffectExecutor;
use crate::game_state::GameState;
use crate::cards::Permanence;
use crate::resources::{CardDefinition, CardManager, QuizState};

#[derive(Resource, Default)]
pub struct DeployedEffectsApplied {
    card_ids: Vec<String>,
}

/// Drives card-effect execution. Runs as an exclusive system so the executor
/// can mutate arbitrary resources/components through the `GameState` path API.
pub fn apply_deployed_card_effects(world: &mut World) {
    let pending = collect_pending(world);
    if pending.is_empty() {
        return;
    }

    sync_question_options_into_collection(world);

    let mut executor = world
        .remove_resource::<EffectExecutor>()
        .unwrap_or_default();
    let mut state = world
        .remove_resource::<GameState>()
        .unwrap_or_default();

    for card in pending {
        let mut context = EffectContext::new(card.id.clone(), String::new());
        for effect in &card.effects {
            context.effect_id = effect.id.clone();
            if let Err(e) =
                executor.execute_effect(effect, &mut context, &mut state, world)
            {
                warn!(
                    "Card '{}' effect '{}' failed: {}",
                    card.id, effect.id, e
                );
            }
        }
        if let Some(mut applied) = world.get_resource_mut::<DeployedEffectsApplied>() {
            applied.card_ids.push(card.id);
        }
    }

    world.insert_resource(executor);
    world.insert_resource(state);

    sync_question_options_out_of_collection(world);
}

fn collect_pending(world: &mut World) -> Vec<CardDefinition> {
    let applied_ids: Vec<String> = world
        .get_resource::<DeployedEffectsApplied>()
        .map(|a| a.card_ids.clone())
        .unwrap_or_default();

    let Some(card_manager) = world.get_resource::<CardManager>() else {
        return Vec::new();
    };

    card_manager
        .deployed_card_ids
        .iter()
        .filter(|id| !applied_ids.contains(id))
        .filter_map(|id| {
            card_manager
                .available_cards
                .iter()
                .find(|c| &c.id == id)
                .cloned()
        })
        .collect()
}

/// Projects the active question's options into the `CollectionManager` at
/// `question.options` so `Remove`/`Filter` effect operations can act on them.
fn sync_question_options_into_collection(world: &mut World) {
    let options: Option<Vec<QuestionOption>> = world
        .iter_entities()
        .find(|e| e.contains::<ActiveQuestion>())
        .and_then(|e| e.get::<Question>())
        .map(|q| q.options.clone());

    let Some(options) = options else { return };

    let collection = Collection::from_vec(
        options.iter().map(question_option_to_value).collect(),
    );

    let mut collections = world.get_resource_or_insert_with(CollectionManager::default);
    collections.set("question.options", collection);
}

/// Mirrors the possibly-mutated `question.options` collection back onto the
/// active question entity.
fn sync_question_options_out_of_collection(world: &mut World) {
    let Some(collections) = world.get_resource::<CollectionManager>() else {
        return;
    };
    let Some(collection) = collections.get("question.options") else {
        return;
    };

    let new_options: Vec<QuestionOption> = collection
        .iter()
        .filter_map(value_to_question_option)
        .collect();

    let mut query = world.query_filtered::<&mut Question, With<ActiveQuestion>>();
    if let Some(mut q) = query.iter_mut(world).next() {
        q.options = new_options;
    }
}

/// Remove one-shot cards when the question changes.
pub fn expire_one_shot_cards(world: &mut World) {
    let changed = world
        .get_resource_ref::<QuizState>()
        .is_some_and(|qs| qs.is_changed() && qs.game_started);
    if !changed {
        return;
    }

    let Some(card_manager) = world.get_resource::<CardManager>() else {
        return;
    };

    let expired: Vec<String> = card_manager
        .deployed_card_ids
        .iter()
        .filter(|id| {
            card_manager
                .available_cards
                .iter()
                .find(|c| &c.id == *id)
                .is_some_and(|c| c.permanence == Permanence::OneShot)
        })
        .cloned()
        .collect();

    if expired.is_empty() {
        return;
    }

    let mut card_manager = world.get_resource_mut::<CardManager>().unwrap();
    card_manager
        .deployed_card_ids
        .retain(|id| !expired.contains(id));

    if let Some(mut applied) = world.get_resource_mut::<DeployedEffectsApplied>() {
        applied.card_ids.retain(|id| !expired.contains(id));
    }

    for id in &expired {
        info!("Expired one-shot card: {}", id);
    }
}

fn question_option_to_value(option: &QuestionOption) -> Value {
    let mut map = std::collections::HashMap::new();
    map.insert("id".to_string(), Value::String(option.id.clone()));
    map.insert("text".to_string(), Value::String(option.text.clone()));
    map.insert("correct".to_string(), Value::Bool(option.correct));
    Value::Object(map)
}

fn value_to_question_option(value: &Value) -> Option<QuestionOption> {
    let Value::Object(map) = value else { return None };
    Some(QuestionOption {
        id: map.get("id").and_then(|v| v.as_string())?.to_string(),
        text: map.get("text").and_then(|v| v.as_string())?.to_string(),
        correct: map.get("correct").and_then(|v| v.as_bool())?,
    })
}
