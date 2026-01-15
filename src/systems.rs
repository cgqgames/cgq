use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

/// Main quiz logic system - manages which question is active
pub fn quiz_system(
    mut commands: Commands,
    quiz_state: Res<QuizState>,
    active_questions: Query<Entity, With<ActiveQuestion>>,
    all_questions: Query<(Entity, &Question)>,
) {
    if !quiz_state.game_started || quiz_state.game_complete {
        return;
    }

    // If current question index changed, update active question marker
    if quiz_state.is_changed() {
        // Remove ActiveQuestion from all
        for entity in active_questions.iter() {
            commands.entity(entity).remove::<ActiveQuestion>();
        }

        // Add ActiveQuestion to the current index
        let mut questions_vec: Vec<_> = all_questions.iter().collect();

        if let Some((entity, _)) = questions_vec.get(quiz_state.current_question_index) {
            commands.entity(*entity).insert(ActiveQuestion);
        }
    }
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
    mut score: ResMut<Score>,
    questions: Query<&Question, With<ActiveQuestion>>,
) {
    // Pause/resume
    if keyboard.just_pressed(KeyCode::Space) {
        quiz_state.paused = !quiz_state.paused;
        info!("Game {}", if quiz_state.paused { "paused" } else { "resumed" });
    }

    // Start game with Enter
    if keyboard.just_pressed(KeyCode::Enter) && !quiz_state.game_started {
        quiz_state.game_started = true;
        info!("Game started!");
    }

    // Answer input (A/B/C/D)
    if quiz_state.game_started && !quiz_state.paused {
        let answer_key = if keyboard.just_pressed(KeyCode::KeyA) {
            Some("a")
        } else if keyboard.just_pressed(KeyCode::KeyB) {
            Some("b")
        } else if keyboard.just_pressed(KeyCode::KeyC) {
            Some("c")
        } else if keyboard.just_pressed(KeyCode::KeyD) {
            Some("d")
        } else {
            None
        };

        if let Some(answer) = answer_key {
            if let Ok(question) = questions.get_single() {
                let correct = question.is_correct(answer);

                if correct {
                    score.current += question.points;
                    score.correct_answers += 1;
                    info!("✅ Correct! +{} points. Score: {}", question.points, score.current);
                } else {
                    info!("❌ Wrong! Correct answer: {:?}",
                        question.correct_answer().map(|o| &o.id));
                }

                score.total_answered += 1;

                // Signal to move to next question
                quiz_state.paused = true;
            }
        }

        // Next question with N key
        if keyboard.just_pressed(KeyCode::KeyN) {
            quiz_state.current_question_index += 1;
            quiz_state.paused = false;

            if quiz_state.current_question_index >= quiz_state.total_questions {
                quiz_state.game_complete = true;
                info!("🏁 Quiz complete! Final score: {} / {}", score.current, score.passing_grade);
                info!("Correct: {} / {}", score.correct_answers, score.total_answered);
            } else {
                info!("Moving to question {}", quiz_state.current_question_index + 1);
            }
        }
    }
}
