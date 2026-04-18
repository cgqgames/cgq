//! Per-chatter identity, answer history, and timeout state.
//!
//! Players exist as entities keyed by their stable Twitch user ID (with
//! username as a fallback when a provider doesn't surface an ID). The chat
//! plugin upserts a Player on any incoming vote. Answer resolution writes
//! per-player `AnswerRecord`s so card effects targeting specific players
//! (last_answerer, most_correct, etc.) and any future shop/currency system
//! can compute from authoritative history rather than derived counters.

use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerRegistry>();
    }
}

#[derive(Component, Debug)]
pub struct Player {
    pub twitch_id: String,
    pub username: String,
    pub joined_at: Instant,
    pub timeout_until: Option<Instant>,
    pub answer_history: Vec<AnswerRecord>,
}

#[derive(Clone, Debug)]
pub struct AnswerRecord {
    pub question_id: String,
    pub vote: String,
    pub was_correct: bool,
    pub at: Instant,
}

/// Maps `twitch_id` → `Entity` for O(1) Player lookup without scanning every
/// entity on every chat message. Kept in sync by `upsert_player`.
#[derive(Resource, Default)]
pub struct PlayerRegistry {
    by_id: HashMap<String, Entity>,
}

impl PlayerRegistry {
    pub fn get(&self, twitch_id: &str) -> Option<Entity> {
        self.by_id.get(twitch_id).copied()
    }
}

/// Look up an existing Player entity by `twitch_id`, or spawn a new one.
/// Also refreshes the stored username (people can change their display name).
pub fn upsert_player(
    commands: &mut Commands,
    registry: &mut PlayerRegistry,
    players: &mut Query<&mut Player>,
    twitch_id: &str,
    username: &str,
) -> Entity {
    if let Some(entity) = registry.by_id.get(twitch_id).copied() {
        if let Ok(mut player) = players.get_mut(entity) {
            if player.username != username {
                player.username = username.to_string();
            }
        }
        return entity;
    }

    let entity = commands
        .spawn(Player {
            twitch_id: twitch_id.to_string(),
            username: username.to_string(),
            joined_at: Instant::now(),
            timeout_until: None,
            answer_history: Vec::new(),
        })
        .id();
    registry.by_id.insert(twitch_id.to_string(), entity);
    entity
}

/// Derive a stable identifier from a chat message. Prefers the Twitch user
/// ID (stable even if a viewer changes their display name). Falls back to
/// the username when the provider didn't surface an ID — this means every
/// mock/test harness that omits `user_id` gets consistent Player entities
/// keyed by username.
pub fn stable_id(user_id: Option<&str>, username: &str) -> String {
    user_id
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("name:{}", username))
}
