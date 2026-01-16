// CGQ - Card Game Quiz Framework
// A Bevy-based generic game engine with data-driven card effects

pub mod effect;
pub mod effect_executor;
pub mod game_state;
pub mod collections;
pub mod components;
pub mod resources;
pub mod cards;
pub mod chat;
pub mod twitch;

// Re-export commonly used types
pub use effect::{CardEffect, EffectOperation, Predicate, Value, EffectContext, EffectTiming};
pub use effect_executor::EffectExecutor;
pub use game_state::GameState;
pub use collections::{Collection, CollectionManager};
pub use resources::{GameTimer, Score, QuizState, CardManager};
pub use components::{Question, QuestionOption, CardType};
