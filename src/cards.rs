use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::components::CardType;
use crate::resources::CardDefinition;

#[derive(Debug, Serialize, Deserialize)]
pub struct CardSet {
    pub metadata: CardMetadata,
    pub cards: Vec<Card>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardMetadata {
    pub title: String,
    pub description: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub card_type: CardType,
    pub permanence: String,
    pub vote_requirement: usize,
    pub cost: i32,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub effects: Vec<CardEffect>,
    #[serde(default)]
    pub visual: CardVisual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardEffect {
    pub id: String,
    #[serde(rename = "type")]
    pub effect_type: String,
    #[serde(flatten)]
    pub parameters: serde_json::Value,
    pub intercepts: Vec<InterceptPattern>,
    pub timing: String,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptPattern {
    pub component: String,
    pub operation: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardVisual {
    pub image: Option<String>,
    pub sound: Option<String>,
}

impl CardSet {
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Ok(serde_yaml::from_str(&content)?)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }
}

/// Load all card sets from the content directory
pub async fn load_all_cards() -> Result<Vec<CardDefinition>> {
    let mut all_cards = Vec::new();

    // Load resistance cards
    if let Ok(resistance) = CardSet::from_file("content/palestinian-quiz/cards/resistance.yml").await {
        all_cards.extend(resistance.cards.into_iter().map(card_to_definition));
    }

    // Load palestinian cards
    if let Ok(palestinian) = CardSet::from_file("content/palestinian-quiz/cards/palestinian.yml").await {
        all_cards.extend(palestinian.cards.into_iter().map(card_to_definition));
    }

    // Load negative cards
    if let Ok(negative) = CardSet::from_file("content/palestinian-quiz/cards/negative.yml").await {
        all_cards.extend(negative.cards.into_iter().map(card_to_definition));
    }

    Ok(all_cards)
}

fn card_to_definition(card: Card) -> CardDefinition {
    use crate::resources::CardEffectDefinition;

    CardDefinition {
        id: card.id,
        name: card.name,
        card_type: card.card_type,
        description: card.description,
        cost: card.cost,
        vote_requirement: card.vote_requirement,
        effects: card.effects.into_iter().map(|e| CardEffectDefinition {
            effect_type: e.effect_type,
            priority: e.priority,
            parameters: e.parameters,
        }).collect(),
    }
}

/// Load questions from YAML
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionSet {
    pub metadata: Option<QuestionMetadata>,
    pub questions: Vec<crate::components::Question>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
}

impl QuestionSet {
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Ok(serde_yaml::from_str(&content)?)
    }
}
