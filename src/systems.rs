use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Main quiz logic system
pub fn quiz_system(
    quiz_state: Res<QuizState>,
    _questions: Query<&Question, With<ActiveQuestion>>,
) {
    if !quiz_state.game_started || quiz_state.game_complete {
        return;
    }

    // Quiz logic will go here
}

/// Processes card effects on game state
/// This is where cards modify questions, timer, score, etc.
pub fn card_effect_system(
    mut questions: Query<&mut Question, With<ActiveQuestion>>,
    eliminate_effects: Query<&EliminateWrongAnswers>,
    time_effects: Query<&ModifyTime>,
    mut game_timer: ResMut<GameTimer>,
) {
    // Apply eliminate wrong answer effects
    if let Ok(mut question) = questions.get_single_mut() {
        for effect in eliminate_effects.iter() {
            // Remove wrong answers based on card effect
            let mut incorrect_indices: Vec<usize> = question.options
                .iter()
                .enumerate()
                .filter(|(_, opt)| !opt.correct)
                .map(|(i, _)| i)
                .collect();

            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            incorrect_indices.shuffle(&mut rng);

            // Remove up to 'count' incorrect options
            let to_remove = effect.count.min(incorrect_indices.len());
            for _ in 0..to_remove {
                if let Some(idx) = incorrect_indices.pop() {
                    question.options.remove(idx);
                }
            }
        }
    }

    // Apply time modification effects
    for effect in time_effects.iter() {
        let current_remaining = game_timer.timer.remaining_secs();
        let new_time = (current_remaining as i64 + effect.seconds).max(0) as f32;
        let duration_secs = game_timer.timer.duration().as_secs_f32();
        game_timer.timer.set_elapsed(
            std::time::Duration::from_secs_f32(duration_secs - new_time)
        );
    }
}

/// Updates the game timer
pub fn timer_system(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    quiz_state: Res<QuizState>,
) {
    if quiz_state.game_started && !quiz_state.paused && !game_timer.paused {
        game_timer.timer.tick(time.delta());

        if game_timer.timer.finished() {
            info!("Time's up!");
        }
    }
}

/// Handles player input
pub fn input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut quiz_state: ResMut<QuizState>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        quiz_state.paused = !quiz_state.paused;
        info!("Game {}", if quiz_state.paused { "paused" } else { "resumed" });
    }
}
