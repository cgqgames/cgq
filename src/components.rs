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
        self.options
            .iter()
            .any(|opt| opt.id == answer.to_lowercase() && opt.correct)
    }
}

/// Marker component for the current active question
#[derive(Component)]
pub struct ActiveQuestion;

/// How long a deployed card stays on the board.
///
/// - `Permanent`: stays until explicitly removed (e.g. countered by another card).
/// - `OneShot`: removed on the next question change.
/// - `Turns { count }`: removed after N question changes. `count = 1` is
///   equivalent to `OneShot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permanence {
    Permanent,
    OneShot,
    Turns { count: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CardType {
    Resistance,
    Palestinian,
    Politics,
    Negative,
    #[allow(clippy::upper_case_acronyms)]
    IDF,
    Hasbara,
    Ceasefire,
    Other,
}
