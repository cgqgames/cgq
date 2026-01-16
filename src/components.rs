use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a quiz question entity
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub text: String,
    pub options: Vec<QuestionOption>,
    pub points: i32,
    pub explanation: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip, default)]
    pub question_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestionOption {
    pub id: String,
    pub text: String,
    pub correct: bool,
}

impl Question {
    pub fn correct_answer(&self) -> Option<&QuestionOption> {
        self.options.iter().find(|opt| opt.correct)
    }

    pub fn is_correct(&self, answer: &str) -> bool {
        self.options.iter()
            .any(|opt| opt.id == answer.to_lowercase() && opt.correct)
    }
}

/// Marker component for the current active question
#[derive(Component)]
pub struct ActiveQuestion;

/// Represents a deployed card in the game
#[derive(Component, Clone, Debug)]
pub struct DeployedCard {
    pub card_id: String,
    pub name: String,
    pub card_type: CardType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CardType {
    Resistance,
    Palestinian,
    Politics,
    Negative,
}

/// Card effect components - each card effect type is its own component
/// This allows Bevy's ECS to efficiently query for active effects

#[derive(Component, Clone, Debug)]
pub struct EliminateWrongAnswers {
    pub count: usize,
    pub priority: i32,
}

#[derive(Component, Clone, Debug)]
pub struct ModifyTime {
    pub seconds: i64,
    pub priority: i32,
}

#[derive(Component, Clone, Debug)]
pub struct ModifyPoints {
    pub points: i32,
    pub priority: i32,
}

#[derive(Component, Clone, Debug)]
pub struct MultiplyPoints {
    pub multiplier: f32,
    pub priority: i32,
}

/// Player answer submission
#[derive(Component)]
pub struct PlayerAnswer {
    pub player_id: String,
    pub username: String,
    pub answer: String,
}
