use bevy::{
    prelude::*,
    ui::{node_bundles::*, Style, Val, UiRect, FlexDirection, JustifyContent, AlignItems},
    text::{Text, TextStyle, TextSection},
};
use clap::Parser;
use std::path::PathBuf;

mod components;
mod resources;
mod systems;
mod cards;
mod effect;
mod game_state;
mod effect_executor;

use components::*;
use resources::*;
use systems::*;
use game_state::GameState;

#[derive(Parser, Debug, Resource, Clone)]
#[command(name = "cgq")]
#[command(about = "Card Game Quiz Framework - A Bevy-based quiz game engine", long_about = None)]
struct Args {
    /// Path to the quiz YAML file
    #[arg(short, long, default_value = "content/palestinian-quiz/questions/test.yml")]
    quiz: PathBuf,

    /// Path to cards directory (optional)
    #[arg(short, long)]
    cards: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CGQ - Card Game Quiz".to_string(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Resources
        .insert_resource(args)
        .init_resource::<QuizState>()
        .init_resource::<GameTimer>()
        .init_resource::<Score>()
        .init_resource::<CardManager>()
        .init_resource::<GameState>()
        // Systems
        .add_systems(Startup, (setup, load_quiz))
        .add_systems(Update, (
            quiz_system,
            card_effect_system,
            timer_system,
            input_system,
            ui_system,
        ))
        .run();
}

fn load_quiz(
    mut commands: Commands,
    mut quiz_state: ResMut<QuizState>,
    args: Res<Args>,
) {
    // Load questions from CLI-specified path
    let yaml_content = std::fs::read_to_string(&args.quiz)
        .unwrap_or_else(|e| panic!("Failed to load questions from {:?}: {}", args.quiz, e));

    let question_set: cards::QuestionSet = serde_yaml::from_str(&yaml_content)
        .unwrap_or_else(|e| panic!("Failed to parse questions YAML: {}", e));

    quiz_state.total_questions = question_set.questions.len();

    info!("Loaded {} questions from {:?}", question_set.questions.len(), args.quiz);

    // Spawn all question entities
    for (index, question) in question_set.questions.into_iter().enumerate() {
        let mut entity = commands.spawn(question);

        // Mark the first question as active
        if index == 0 {
            entity.insert(ActiveQuestion);
        }
    }
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    info!("CGQ Game Started");
}

fn ui_system(
    mut commands: Commands,
    quiz_state: Res<QuizState>,
    score: Res<Score>,
    timer: Res<GameTimer>,
    questions: Query<&Question, With<ActiveQuestion>>,
    existing_ui: Query<Entity, With<QuizUI>>,
) {
    // Clear existing UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Start screen
    if !quiz_state.game_started {
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
                background_color: Color::srgb(0.1, 0.1, 0.15).into(),
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
        return;
    }

    // Game over screen
    if quiz_state.game_complete {
        let passed = score.current >= score.passing_grade;
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
                background_color: Color::srgb(0.1, 0.1, 0.15).into(),
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
        return;
    }

    if let Ok(question) = questions.get_single() {
        // Root UI container
        commands.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
                background_color: Color::srgb(0.1, 0.1, 0.15).into(),
                ..default()
            },
            QuizUI,
        ))
        .with_children(|parent| {
            // Header: Timer and Score
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
                ..default()
            }).with_children(|header| {
                header.spawn(TextBundle {
                    text: Text::from_section(
                        format!("Time: {:.0}s", timer.timer.remaining_secs()),
                        TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                });

                header.spawn(TextBundle {
                    text: Text::from_section(
                        format!("Score: {} / {}", score.current, score.passing_grade),
                        TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                });

                header.spawn(TextBundle {
                    text: Text::from_section(
                        format!("Question {} / {}", quiz_state.current_question_index + 1, quiz_state.total_questions),
                        TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                });
            });

            // Question text
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        &question.text,
                        TextStyle {
                            font_size: 50.0,
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

            // Options
            for option in &question.options {
                parent.spawn((
                    TextBundle {
                        text: Text::from_section(
                            format!("{}) {}", option.id.to_uppercase(), option.text),
                            TextStyle {
                                font_size: 36.0,
                                color: Color::srgb(0.8, 0.8, 0.9),
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        ..default()
                    },
                ));
            }
        });
    }
}

/// Marker component for UI
#[derive(Component)]
struct QuizUI;
