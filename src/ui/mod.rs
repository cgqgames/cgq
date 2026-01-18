mod screens;
mod game_ui;
mod cards_ui;

use bevy::prelude::*;
use crate::components::{Question, ActiveQuestion};
use crate::resources::{QuizState, Score, GameTimer, CardManager, CardDefinition};
use crate::ui_config::UiConfig;
use crate::card_3d::CardRenderTexture;
use crate::chat_plugin::ChatCardVoteTracker;

/// Marker component for UI entities
#[derive(Component)]
pub struct QuizUI;

/// Main UI system that renders all game UI
pub fn ui_system(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    quiz_state: Res<QuizState>,
    score: Res<Score>,
    timer: Res<GameTimer>,
    ui_config: Res<UiConfig>,
    card_manager: Res<CardManager>,
    _card_vote_tracker: Option<Res<ChatCardVoteTracker>>,
    card_render_texture: Option<Res<CardRenderTexture>>,
    args: Res<crate::Args>,
    questions: Query<&Question, With<ActiveQuestion>>,
    existing_ui: Query<Entity, With<QuizUI>>,
) {
    // Clear existing UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Start screen
    if !quiz_state.game_started {
        screens::render_start_screen(&mut commands, &quiz_state, args.live);
        return;
    }

    // Game over screen
    if quiz_state.game_complete {
        screens::render_game_over_screen(&mut commands, &score, args.live);
        return;
    }

    if let Ok(question) = questions.get_single() {
        // Collect card data once for the entire UI render
        let deployed_cards: Vec<CardDefinition> = card_manager.available_cards
            .iter()
            .filter(|card| card_manager.deployed_card_ids.contains(&card.id))
            .cloned()
            .collect();

        // Root container (transparent background)
        commands.spawn((
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
                &card_manager.available_cards,
                card_render_texture.as_deref(),
            );
        });
    }
}
