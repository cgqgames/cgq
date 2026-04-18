mod screens;
mod game_ui;
mod cards_ui;

use bevy::prelude::*;

use crate::card_3d::CardRenderTexture;
#[cfg(not(target_arch = "wasm32"))]
use crate::chat_plugin::ChatCardVoteTracker;
use crate::components::{ActiveQuestion, Question};
use crate::modes::SessionMode;
use crate::resources::{CardDefinition, CardManager, GameTimer, QuizState, Score};
use crate::ui_config::UiConfig;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system);
    }
}

/// Marker component for UI entities
#[derive(Component)]
pub struct QuizUI;

#[allow(unused_variables)]
fn ui_system(
    mut commands: Commands,
    session_mode: Res<State<SessionMode>>,
    quiz_state: Res<QuizState>,
    score: Res<Score>,
    timer: Res<GameTimer>,
    ui_config: Res<UiConfig>,
    card_manager: Res<CardManager>,
    #[cfg(not(target_arch = "wasm32"))]
    _card_vote_tracker: Option<Res<ChatCardVoteTracker>>,
    card_render_texture: Option<Res<CardRenderTexture>>,
    args: Res<crate::Args>,
    questions: Query<&Question, With<ActiveQuestion>>,
    existing_ui: Query<Entity, With<QuizUI>>,
) {
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    match session_mode.get() {
        SessionMode::Startup => {
            screens::render_start_screen(&mut commands, &quiz_state, args.live);
            return;
        }
        SessionMode::GameOver => {
            screens::render_game_over_screen(&mut commands, &score, args.live);
            return;
        }
        _ => {}
    }

    let Ok(question) = questions.get_single() else { return };

    let deployed_cards: Vec<CardDefinition> = card_manager
        .available_cards
        .iter()
        .filter(|card| card_manager.deployed_card_ids.contains(&card.id))
        .cloned()
        .collect();

    commands
        .spawn((
            NodeBundle {
                style: bevy::ui::Style {
                    width: bevy::ui::Val::Percent(100.0),
                    height: bevy::ui::Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
            QuizUI,
        ))
        .with_children(|parent| {
            game_ui::render_question_box(parent, &ui_config, &quiz_state, question);
            game_ui::render_timer_box(parent, &ui_config, &timer, &score);
            cards_ui::render_cards_section(
                parent,
                &ui_config.cards_grid,
                &deployed_cards,
                card_render_texture.as_deref(),
            );
        });
}
