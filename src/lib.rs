// CGQ - Card Game Quiz Framework
// A Bevy-based generic game engine with data-driven card effects

pub mod card_templates;
pub mod cards;
pub mod collections;
pub mod components;
pub mod content_config;
pub mod effect;
pub mod effect_executor;
pub mod game_state;
pub mod resources;
pub mod ui_config;

#[cfg(not(target_arch = "wasm32"))]
pub mod chat;
#[cfg(not(target_arch = "wasm32"))]
pub mod twitch;

// Re-export commonly used types
pub use card_templates::{expand, YamlCardEffect};
pub use effect::{CardEffect, EffectOperation, Predicate, Value, EffectContext, EffectTiming};
pub use effect_executor::EffectExecutor;
pub use game_state::GameState;
pub use collections::{Collection, CollectionManager};
pub use resources::{GameTimer, Score, QuizState, CardManager};
pub use components::{Question, QuestionOption, CardType};
