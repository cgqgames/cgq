use bevy::prelude::*;
use bevy::ui::{Style, Val, UiRect, FlexDirection};
use bevy::text::{Text, TextStyle};
use crate::components::Question;
use crate::resources::{QuizState, Score, GameTimer};
use crate::ui_config::UiConfig;

/// Render the question box (top-left)
pub fn render_question_box(
    parent: &mut ChildBuilder,
    ui_config: &UiConfig,
    quiz_state: &QuizState,
    question: &Question,
) {
    let qbox = &ui_config.question_box;

    parent.spawn(NodeBundle {
        style: Style {
            position_type: bevy::ui::PositionType::Absolute,
            left: Val::Px(qbox.left),
            top: Val::Px(qbox.top),
            width: Val::Px(qbox.width),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: ui_config.question_box_background().into(),
        ..default()
    }).with_children(|question_box| {
        // Question header
        question_box.spawn(TextBundle {
            text: Text::from_section(
                format!("QUESTION #{}", quiz_state.current_question_index + 1),
                TextStyle {
                    font_size: qbox.header_font_size,
                    color: ui_config.accent_color(),
                    ..default()
                },
            ),
            style: Style {
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            ..default()
        });

        // Question text
        question_box.spawn(TextBundle {
            text: Text::from_section(
                &question.text,
                TextStyle {
                    font_size: qbox.font_size,
                    color: ui_config.text_primary_color(),
                    ..default()
                },
            ),
            style: Style {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
            ..default()
        });

        // Options
        for option in &question.options {
            question_box.spawn(TextBundle {
                text: Text::from_section(
                    format!("{}) {}", option.id.to_uppercase(), option.text),
                    TextStyle {
                        font_size: qbox.option_font_size,
                        color: ui_config.text_secondary_color(),
                        ..default()
                    },
                ),
                style: Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                ..default()
            });
        }
    });
}

/// Render the timer and status bar (bottom-left)
pub fn render_timer_box(
    parent: &mut ChildBuilder,
    ui_config: &UiConfig,
    timer: &GameTimer,
    score: &Score,
) {
    let tbox = &ui_config.timer_box;

    parent.spawn(NodeBundle {
        style: Style {
            position_type: bevy::ui::PositionType::Absolute,
            left: Val::Px(tbox.left),
            bottom: Val::Px(tbox.bottom),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            ..default()
        },
        background_color: ui_config.timer_box_background().into(),
        ..default()
    }).with_children(|timer_box| {
        // Timer display
        let mins = (timer.timer.remaining_secs() / 60.0) as i32;
        let secs = (timer.timer.remaining_secs() % 60.0) as i32;
        timer_box.spawn(TextBundle {
            text: Text::from_section(
                format!("{:02}:{:02}", mins, secs),
                TextStyle {
                    font_size: tbox.timer_font_size,
                    color: ui_config.timer_color(),
                    ..default()
                },
            ),
            style: Style {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            ..default()
        });

        // Status bar
        timer_box.spawn(TextBundle {
            text: Text::from_section(
                format!(
                    "Palestine Quiz | Time: {}min | Passing Grade: {} | Questions Answered: {} | Current Grade: {}",
                    (timer.timer.duration().as_secs() / 60),
                    score.passing_grade,
                    score.total_answered,
                    score.current
                ),
                TextStyle {
                    font_size: tbox.status_font_size,
                    color: ui_config.timer_color(),
                    ..default()
                },
            ),
            ..default()
        });
    });
}
