use bevy::prelude::*;
use std::time::Duration;

use crate::components::Permanence;
use crate::effect::CardEffect;

/// Global quiz state
#[derive(Resource, Default)]
pub struct QuizState {
    pub current_question_index: usize,
    pub total_questions: usize,
    pub game_started: bool,
    pub game_complete: bool,
    pub paused: bool,
}

/// Game timer resource
#[derive(Resource)]
pub struct GameTimer {
    pub timer: Timer,
    pub paused: bool,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(600), TimerMode::Once),
            paused: false,
        }
    }
}

/// Score tracking
#[derive(Resource)]
pub struct Score {
    pub current: i32,
    pub passing_grade: i32,
    pub correct_answers: usize,
    pub total_answered: usize,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            current: 0,
            passing_grade: 6,
            correct_answers: 0,
            total_answered: 0,
        }
    }
}

/// Manages all cards in the game
#[derive(Resource)]
pub struct CardManager {
    pub available_cards: Vec<CardDefinition>,
    pub deployed_card_ids: Vec<String>,
    /// Maximum number of concurrently-deployed permanent cards. Modified by
    /// effects that add/remove table slots.
    pub max_slots: i32,
    /// Per-card-type vote-requirement modifier. Added to a card's base
    /// `vote_requirement` when the chat-consensus system evaluates it. A
    /// "*" key applies to every card type.
    pub vote_req_modifiers: std::collections::HashMap<String, i32>,
    /// Turns remaining for deployed non-permanent cards. A card with an
    /// entry here is decremented on each question change and expired when
    /// the counter reaches zero. Permanent cards have no entry.
    pub turn_counters: std::collections::HashMap<String, u32>,
}

impl Default for CardManager {
    fn default() -> Self {
        Self {
            available_cards: Vec::new(),
            deployed_card_ids: Vec::new(),
            max_slots: 4,
            vote_req_modifiers: std::collections::HashMap::new(),
            turn_counters: std::collections::HashMap::new(),
        }
    }
}

/// Card definition loaded from YAML.
/// Effects are stored as fully-expanded operation trees — the YAML shorthand
/// is translated at load time by `card_templates::expand`.
#[derive(Clone, Debug)]
pub struct CardDefinition {
    pub id: String,
    pub name: String,
    pub card_type: crate::components::CardType,
    pub permanence: Permanence,
    pub description: Option<String>,
    pub cost: i32,
    pub vote_requirement: usize,
    pub effects: Vec<CardEffect>,
}
