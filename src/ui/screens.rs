use bevy::prelude::*;
use bevy::ui::{Style, Val, UiRect, FlexDirection, JustifyContent, AlignItems};
use bevy::text::{Text, TextStyle};
use crate::resources::{QuizState, Score};
use super::QuizUI;

/// Render the start screen
pub fn render_start_screen(
    commands: &mut Commands,
    quiz_state: &QuizState,
    live_mode: bool,
) {
    let screen_bg = if live_mode {
        Color::NONE
    } else {
        Color::srgb(0.1, 0.1, 0.15)
    };

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: screen_bg.into(),
            ..default()
        },
        QuizUI,
    )).with_children(|parent| {
        parent.spawn((
            TextBundle {
                text: Text::from_section(
                    "CGQ - Palestinian History Quiz",
                    TextStyle {
                        font_size: 60.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
                ..default()
            },
        ));

        parent.spawn(TextBundle {
            text: Text::from_section(
                "Press ENTER to start",
                TextStyle {
                    font_size: 40.0,
                    color: Color::srgb(0.7, 0.7, 0.8),
                    ..default()
                },
            ),
            ..default()
        });

        parent.spawn((
            TextBundle {
                text: Text::from_section(
                    format!("{} questions loaded", quiz_state.total_questions),
                    TextStyle {
                        font_size: 30.0,
                        color: Color::srgb(0.5, 0.5, 0.6),
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
                ..default()
            },
        ));
    });
}

/// Render the game over screen
pub fn render_game_over_screen(
    commands: &mut Commands,
    score: &Score,
    live_mode: bool,
) {
    let passed = score.current >= score.passing_grade;
    let screen_bg = if live_mode {
        Color::NONE
    } else {
        Color::srgb(0.1, 0.1, 0.15)
    };

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: screen_bg.into(),
            ..default()
        },
        QuizUI,
    )).with_children(|parent| {
        parent.spawn((
            TextBundle {
                text: Text::from_section(
                    if passed { "🎉 YOU WIN!" } else { "📚 Keep Learning!" },
                    TextStyle {
                        font_size: 60.0,
                        color: if passed { Color::srgb(0.2, 0.8, 0.2) } else { Color::srgb(0.8, 0.5, 0.2) },
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
                ..default()
            },
        ));

        parent.spawn(TextBundle {
            text: Text::from_section(
                format!("Final Score: {} / {}", score.current, score.passing_grade),
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            ..default()
        });

        parent.spawn(TextBundle {
            text: Text::from_section(
                format!("Correct: {} / {} ({:.1}%)",
                    score.correct_answers,
                    score.total_answered,
                    (score.correct_answers as f32 / score.total_answered as f32) * 100.0
                ),
                TextStyle {
                    font_size: 35.0,
                    color: Color::srgb(0.8, 0.8, 0.9),
                    ..default()
                },
            ),
            ..default()
        });
    });
}
