use anyhow::{Context, Result};
use bevy::log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::card_templates::{expand, YamlCardEffect};
use crate::components::CardType;
use crate::resources::CardDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permanence {
    Permanent,
    OneShot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub card_type: CardType,
    pub permanence: Permanence,
    pub vote_requirement: usize,
    pub cost: i32,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub effects: Vec<YamlCardEffect>,
    #[serde(default)]
    pub visual: CardVisual,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardVisual {
    pub image: Option<String>,
    pub sound: Option<String>,
}

/// Discover and load all `*.toml` card files in a directory.
pub fn load_cards_from_dir(dir: &Path) -> Result<Vec<CardDefinition>> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Cannot read card directory: {}", dir.display()))?;

    let mut cards = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            match load_card_file(&path) {
                Ok(card) => {
                    info!("Loaded card '{}' from {}", card.name, path.display());
                    cards.push(card);
                }
                Err(e) => {
                    warn!("Failed to load card from {}: {}", path.display(), e);
                }
            }
        }
    }

    cards.sort_by(|a, b| a.id.cmp(&b.id));
    info!("Loaded {} cards from {}", cards.len(), dir.display());
    Ok(cards)
}

fn load_card_file(path: &Path) -> Result<CardDefinition> {
    let content = std::fs::read_to_string(path)?;
    let card: Card = toml::from_str(&content)?;
    Ok(card_to_definition(card))
}

fn card_to_definition(card: Card) -> CardDefinition {
    CardDefinition {
        id: card.id,
        name: card.name,
        card_type: card.card_type,
        permanence: card.permanence,
        description: card.description,
        cost: card.cost,
        vote_requirement: card.vote_requirement,
        effects: card.effects.into_iter().map(expand).collect(),
    }
}

// -- Questions (still YAML) --------------------------------------------------

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
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&content)?)
    }
}
