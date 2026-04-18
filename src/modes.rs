//! Top-level state machinery.
//!
//! The engine is organised as a set of concurrent Bevy `States` — each one
//! an independent state machine. `SessionMode` is the governing mode; it
//! orchestrates the child modes (`QuizMode`, `CardMode`, `ShopMode`) by
//! driving their `NextState<T>` from its own transitions.
//!
//! Each subsystem is packaged as a Bevy Plugin that gates its systems on
//! `run_if(in_state(...))`. Plugins are mode-agnostic: they describe their
//! own lifecycle, and whichever session mode is active decides when their
//! states are `Active`.

use bevy::prelude::*;

/// Governing mode for a playthrough. A higher-level driver (Campaign /
/// Tournament / Standalone) decides when child modes activate.
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum SessionMode {
    #[default]
    Startup,
    Standalone,
    /// Placeholder — no driver yet; will orchestrate Quiz ↔ Shop ↔ Map.
    Campaign,
    /// Placeholder — no driver yet; multi-round leaderboard format.
    Tournament,
    GameOver,
}

/// Whether the quiz loop is running. Governed by `SessionMode`.
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum QuizMode {
    #[default]
    Inactive,
    Active,
}

/// Whether the card subsystem is accepting deployments and running effects.
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum CardMode {
    #[default]
    Inactive,
    Active,
}

/// Whether the shop is accepting purchases. Placeholder until the shop
/// subsystem is built.
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum ShopMode {
    #[default]
    Inactive,
    Open,
}

/// Governs which chat messages the chat plugin will accept as votes.
/// Cards that change chat behaviour manipulate this state.
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum ChatMode {
    #[default]
    Normal,
    EmojiOnly,
    SubOnly,
    FirstAnswerOnly,
}

/// Registers every state type the engine uses. Plugins gate on these.
pub struct ModesPlugin;

impl Plugin for ModesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SessionMode>()
            .init_state::<QuizMode>()
            .init_state::<CardMode>()
            .init_state::<ShopMode>()
            .init_state::<ChatMode>();
    }
}
