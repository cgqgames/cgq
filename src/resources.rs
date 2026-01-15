use bevy::prelude::*;
use std::time::Duration;

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
            passing_grade: 6, // Default passing grade
            correct_answers: 0,
            total_answered: 0,
        }
    }
}

/// Manages all cards in the game
#[derive(Resource, Default)]
pub struct CardManager {
    pub available_cards: Vec<CardDefinition>,
    pub deployed_card_ids: Vec<String>,
}

/// Card definition loaded from YAML
#[derive(Clone, Debug)]
pub struct CardDefinition {
    pub id: String,
    pub name: String,
    pub card_type: crate::components::CardType,
    pub description: Option<String>,
    pub cost: i32,
    pub vote_requirement: usize,
    pub effects: Vec<CardEffectDefinition>,
}

#[derive(Clone, Debug)]
pub struct CardEffectDefinition {
    pub effect_type: String,
    pub priority: i32,
    pub parameters: serde_json::Value,
}
