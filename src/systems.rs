//! The quiz game mode. Systems gated on `QuizMode::Active`.

use bevy::prelude::*;

use crate::components::{ActiveQuestion, Question};
use crate::deploy::AnswerSubmittedEvent;
use crate::modes::{CardMode, QuizMode};
use crate::resources::{CardManager, GameTimer, QuizState, Score};

pub struct QuizPlugin;

impl Plugin for QuizPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuizState>()
            .init_resource::<GameTimer>()
            .init_resource::<Score>()
            .add_systems(
                Update,
                (quiz_system, timer_system, input_system)
                    .run_if(in_state(QuizMode::Active)),
            );
    }
}

/// Keeps exactly one `Question` entity tagged `ActiveQuestion`, matching
/// `QuizState::current_question_index`.
fn quiz_system(
    mut commands: Commands,
    quiz_state: Res<QuizState>,
    active_questions: Query<Entity, With<ActiveQuestion>>,
    all_questions: Query<(Entity, &Question)>,
) {
    if !quiz_state.is_changed() {
        return;
    }

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

fn timer_system(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    quiz_state: Res<QuizState>,
) {
    if quiz_state.paused || game_timer.paused {
        return;
    }
    game_timer.timer.tick(time.delta());
    if game_timer.timer.finished() {
        info!("Time's up!");
    }
}

/// Handles pause, local card deploy (1-4), and answer submission (A-D).
fn input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut quiz_state: ResMut<QuizState>,
    mut score: ResMut<Score>,
    mut card_manager: ResMut<CardManager>,
    mut answer_events: EventWriter<AnswerSubmittedEvent>,
    card_mode: Res<State<CardMode>>,
    questions: Query<&Question, With<ActiveQuestion>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        quiz_state.paused = !quiz_state.paused;
        info!("Game {}", if quiz_state.paused { "paused" } else { "resumed" });
    }

    if quiz_state.paused {
        return;
    }

    if *card_mode.get() == CardMode::Active {
        deploy_card_from_slot(&keyboard, &mut card_manager);
    }

    submit_answer(&keyboard, &mut quiz_state, &mut score, &mut answer_events, &questions);
}

fn deploy_card_from_slot(keyboard: &ButtonInput<KeyCode>, card_manager: &mut CardManager) {
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
}

fn submit_answer(
    keyboard: &ButtonInput<KeyCode>,
    quiz_state: &mut QuizState,
    score: &mut Score,
    answer_events: &mut EventWriter<AnswerSubmittedEvent>,
    questions: &Query<&Question, With<ActiveQuestion>>,
) {
    let answer = if keyboard.just_pressed(KeyCode::KeyA) {
        "a"
    } else if keyboard.just_pressed(KeyCode::KeyB) {
        "b"
    } else if keyboard.just_pressed(KeyCode::KeyC) {
        "c"
    } else if keyboard.just_pressed(KeyCode::KeyD) {
        "d"
    } else {
        return;
    };

    let Ok(question) = questions.get_single() else { return };

    let correct = question.is_correct(answer);
    if correct {
        score.current += question.points;
        score.correct_answers += 1;
        info!("✅ Correct! +{} points. Score: {}", question.points, score.current);
    } else {
        info!(
            "❌ Wrong! Correct answer: {:?}",
            question.correct_answer().map(|o| &o.id)
        );
    }

    answer_events.send(AnswerSubmittedEvent {
        correct,
        question_id: question.id.clone(),
    });

    score.total_answered += 1;
    quiz_state.current_question_index += 1;

    if quiz_state.current_question_index < quiz_state.total_questions {
        info!("Moving to question {}", quiz_state.current_question_index + 1);
    }
}
