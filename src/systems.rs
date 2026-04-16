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

    if quiz_state.is_changed() {
        for entity in active_questions.iter() {
            commands.entity(entity).remove::<ActiveQuestion>();
        }

        for (entity, question) in all_questions.iter() {
            if question.question_index == quiz_state.current_question_index {
                commands.entity(entity).insert(ActiveQuestion);
                break;
            }
        }
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
    mut card_manager: ResMut<CardManager>,
    questions: Query<&Question, With<ActiveQuestion>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        quiz_state.paused = !quiz_state.paused;
        info!("Game {}", if quiz_state.paused { "paused" } else { "resumed" });
    }

    if keyboard.just_pressed(KeyCode::Enter) && !quiz_state.game_started {
        quiz_state.game_started = true;
        info!("Game started!");
    }

    if !quiz_state.game_started || quiz_state.paused {
        return;
    }

    // Local card deployment: keys 1..=4 deploy the card at that board slot.
    // Remote deployment happens via the Twitch chat path (chat_plugin).
    for (slot, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4]
        .into_iter()
        .enumerate()
    {
        if !keyboard.just_pressed(key) {
            continue;
        }
        let Some(card) = card_manager.available_cards.get(slot) else {
            continue;
        };
        if card_manager.deployed_card_ids.contains(&card.id) {
            continue;
        }
        info!("Deploying card from slot {}: {}", slot + 1, card.name);
        let id = card.id.clone();
        card_manager.deployed_card_ids.push(id);
    }

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

    let Some(answer) = answer_key else { return };
    let Ok(question) = questions.get_single() else { return };

    if question.is_correct(answer) {
        score.current += question.points;
        score.correct_answers += 1;
        info!("✅ Correct! +{} points. Score: {}", question.points, score.current);
    } else {
        info!(
            "❌ Wrong! Correct answer: {:?}",
            question.correct_answer().map(|o| &o.id)
        );
    }

    score.total_answered += 1;
    quiz_state.current_question_index += 1;

    if quiz_state.current_question_index >= quiz_state.total_questions {
        quiz_state.game_complete = true;
        info!(
            "🏁 Quiz complete! Final score: {} / {}",
            score.current, score.passing_grade
        );
        info!("Correct: {} / {}", score.correct_answers, score.total_answered);
    } else {
        info!("Moving to question {}", quiz_state.current_question_index + 1);
    }
}
