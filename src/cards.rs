//! Bridges parsed content configuration (`CardConfig`, `QuestionConfig`) into
//! the runtime types used by the engine (`CardDefinition`, `Question`).

use bevy::log::info;
use indexmap::IndexMap;

use crate::card_templates::expand;
use crate::components::Question;
use crate::content_config::{CardConfig, QuestionConfig};
use crate::resources::CardDefinition;

pub fn cards_from_configs(configs: IndexMap<String, CardConfig>) -> Vec<CardDefinition> {
    let mut defs: Vec<CardDefinition> = configs
        .into_iter()
        .map(|(id, c)| CardDefinition {
            id,
            name: c.name,
            card_type: c.card_type,
            permanence: c.permanence,
            description: c.description,
            cost: c.cost,
            vote_requirement: c.vote_requirement,
            effects: c.effects.into_iter().map(expand).collect(),
        })
        .collect();
    defs.sort_by(|a, b| a.id.cmp(&b.id));
    info!("Loaded {} cards from config", defs.len());
    defs
}

pub fn questions_from_configs(configs: IndexMap<String, QuestionConfig>) -> Vec<Question> {
    let questions: Vec<Question> = configs
        .into_iter()
        .enumerate()
        .map(|(index, (id, c))| Question {
            id,
            text: c.text,
            options: c.options,
            points: c.points,
            explanation: c.explanation,
            source: c.source,
            tags: c.tags,
            question_index: index,
        })
        .collect();
    info!("Loaded {} questions from config", questions.len());
    questions
}
