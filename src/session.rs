//! Standalone session orchestrator.
//!
//! Drives `SessionMode` transitions for a standard one-off playthrough
//! (not Campaign, not Tournament — those have their own drivers that
//! will live beside this one).
//!
//! Flow:
//!   Startup -> (Enter pressed) -> Standalone
//!   Standalone -> (quiz complete) -> GameOver

use bevy::prelude::*;

use crate::modes::{CardMode, QuizMode, SessionMode};
use crate::resources::{QuizState, Score};

pub struct StandaloneSessionPlugin;

impl Plugin for StandaloneSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            start_on_enter.run_if(in_state(SessionMode::Startup)),
        )
        .add_systems(
            Update,
            detect_game_over.run_if(in_state(SessionMode::Standalone)),
        )
        .add_systems(OnEnter(SessionMode::Standalone), activate_children)
        .add_systems(OnEnter(SessionMode::GameOver), deactivate_children);
    }
}

fn start_on_enter(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_session: ResMut<NextState<SessionMode>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        info!("Game started");
        next_session.set(SessionMode::Standalone);
    }
}

fn activate_children(
    mut next_quiz: ResMut<NextState<QuizMode>>,
    mut next_cards: ResMut<NextState<CardMode>>,
) {
    next_quiz.set(QuizMode::Active);
    next_cards.set(CardMode::Active);
}

fn deactivate_children(
    mut next_quiz: ResMut<NextState<QuizMode>>,
    mut next_cards: ResMut<NextState<CardMode>>,
) {
    next_quiz.set(QuizMode::Inactive);
    next_cards.set(CardMode::Inactive);
}

fn detect_game_over(
    quiz_state: Res<QuizState>,
    score: Res<Score>,
    mut next_session: ResMut<NextState<SessionMode>>,
) {
    if quiz_state.current_question_index >= quiz_state.total_questions && quiz_state.total_questions > 0 {
        info!(
            "Quiz complete. Final score: {} / {}",
            score.current, score.passing_grade
        );
        next_session.set(SessionMode::GameOver);
    }
}
