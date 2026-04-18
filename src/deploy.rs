//! Bridges `CardManager::deployed_card_ids` to the `EffectExecutor`.
//!
//! Each frame, any card ID newly appended to `deployed_card_ids` has its
//! effects expanded and executed against the live game state. Deployment is
//! triggered by chat votes or local keybinds — both paths feed the same list.

use bevy::prelude::*;

use crate::collections::{Collection, CollectionManager};
use crate::components::{ActiveQuestion, Permanence, Question, QuestionOption};
use crate::effect::{EffectContext, Value};
use crate::effect_executor::EffectExecutor;
use crate::game_state::GameState;
use crate::modes::CardMode;
use crate::resources::{CardDefinition, CardManager, QuizState};

pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CardManager>()
            .init_resource::<CollectionManager>()
            .init_resource::<EffectExecutor>()
            .init_resource::<GameState>()
            .init_resource::<DeployedEffectsApplied>()
            .add_event::<AnswerSubmittedEvent>()
            .add_systems(
                Update,
                (
                    apply_deployed_card_effects,
                    forward_answer_events,
                    expire_cards_on_question_change,
                )
                    .run_if(in_state(CardMode::Active)),
            );
    }
}

#[derive(Resource, Default)]
pub struct DeployedEffectsApplied {
    card_ids: Vec<String>,
}

/// Emitted when an answer resolves, whether from keyboard input or chat
/// consensus. `correct` reflects the team's consensus correctness;
/// per-player attribution lives in `correct_voters` / `wrong_voters`
/// (empty for keyboard input, populated for chat consensus).
#[derive(Event, Debug, Clone)]
pub struct AnswerSubmittedEvent {
    pub correct: bool,
    pub question_id: String,
    pub correct_voters: Vec<String>,
    pub wrong_voters: Vec<String>,
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
        register_turn_counter(world, &card);
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

    let banned_ids = banned_strings(world, "cards.banned_ids");
    let banned_types = banned_strings(world, "cards.banned_types");

    let Some(card_manager) = world.get_resource::<CardManager>() else {
        return Vec::new();
    };

    let mut pending = Vec::new();
    let mut blocked_ids = Vec::new();
    for id in card_manager.deployed_card_ids.iter() {
        if applied_ids.contains(id) {
            continue;
        }
        let Some(card) = card_manager.available_cards.iter().find(|c| &c.id == id) else {
            continue;
        };
        let type_str = card_type_key(&card.card_type);
        if banned_ids.contains(&card.id) || banned_types.contains(&type_str) {
            info!("Card '{}' blocked by active ban", card.id);
            blocked_ids.push(card.id.clone());
            continue;
        }
        pending.push(card.clone());
    }

    if !blocked_ids.is_empty() {
        if let Some(mut cm) = world.get_resource_mut::<CardManager>() {
            cm.deployed_card_ids.retain(|id| !blocked_ids.contains(id));
        }
    }

    pending
}

fn banned_strings(world: &World, path: &str) -> Vec<String> {
    world
        .get_resource::<CollectionManager>()
        .and_then(|c| c.get(path))
        .map(|c| {
            c.iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn card_type_key(card_type: &crate::components::CardType) -> String {
    use crate::components::CardType::*;
    match card_type {
        Resistance => "resistance",
        Palestinian => "palestinian",
        Politics => "politics",
        Negative => "negative",
        IDF => "idf",
        Hasbara => "hasbara",
        Ceasefire => "ceasefire",
        Other => "other",
    }
    .to_string()
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

/// Forwards `AnswerSubmittedEvent` into the effect executor's registered
/// event listeners. Any `OnEvent { event: "answer_correct" | "answer_wrong" }`
/// listeners that were registered by deployed cards run here.
pub fn forward_answer_events(world: &mut World) {
    let events: Vec<AnswerSubmittedEvent> = {
        let Some(mut e) = world.get_resource_mut::<Events<AnswerSubmittedEvent>>() else {
            return;
        };
        e.drain().collect()
    };
    if events.is_empty() {
        return;
    }

    sync_question_options_into_collection(world);

    let mut executor = world.remove_resource::<EffectExecutor>().unwrap_or_default();
    let mut state = world.remove_resource::<GameState>().unwrap_or_default();

    for event in events {
        let event_name = if event.correct { "answer_correct" } else { "answer_wrong" };
        let Some(listeners) = executor.get_event_listeners(event_name) else { continue };
        for ops in listeners {
            let mut context = EffectContext::new(String::new(), event_name.to_string());
            for op in &ops {
                if let Err(e) = executor.execute_operation(op, &mut context, &mut state, world) {
                    warn!("Answer-event handler '{}' failed: {}", event_name, e);
                }
            }
        }
    }

    world.insert_resource(executor);
    world.insert_resource(state);

    sync_question_options_out_of_collection(world);
}

fn register_turn_counter(world: &mut World, card: &CardDefinition) {
    let turns = match card.permanence {
        Permanence::Permanent => return,
        Permanence::OneShot => 1,
        Permanence::Turns { count } => count,
    };
    if let Some(mut cm) = world.get_resource_mut::<CardManager>() {
        cm.turn_counters.insert(card.id.clone(), turns);
    }
}

/// Decrement turn counters on question change; expire cards whose counter
/// reaches zero. Permanent cards have no counter and are never touched here.
pub fn expire_cards_on_question_change(world: &mut World) {
    let changed = world
        .get_resource_ref::<QuizState>()
        .is_some_and(|qs| qs.is_changed());
    if !changed {
        return;
    }

    let Some(mut cm) = world.get_resource_mut::<CardManager>() else {
        return;
    };

    let mut expired: Vec<String> = Vec::new();
    for (id, count) in cm.turn_counters.iter_mut() {
        *count = count.saturating_sub(1);
        if *count == 0 {
            expired.push(id.clone());
        }
    }
    for id in &expired {
        cm.turn_counters.remove(id);
    }
    cm.deployed_card_ids.retain(|id| !expired.contains(id));
    drop(cm);

    if let Some(mut applied) = world.get_resource_mut::<DeployedEffectsApplied>() {
        applied.card_ids.retain(|id| !expired.contains(id));
    }

    for id in &expired {
        info!("Expired card: {}", id);
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
