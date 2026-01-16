use bevy::{
    prelude::*,
    ui::{Style, Val, UiRect, FlexDirection, JustifyContent, AlignItems},
    text::{Text, TextStyle},
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
mod collections;
mod ui_config;
mod chat;
mod twitch;
mod chat_plugin;
mod card_3d;

use components::*;
use resources::*;
use systems::*;
use game_state::GameState;
use collections::CollectionManager;
use ui_config::UiConfig;

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

    /// Path to UI config TOML file (optional, uses built-in defaults if not provided)
    #[arg(short = 'u', long)]
    ui_config: Option<PathBuf>,

    /// Twitch channel to connect to for chat integration (optional)
    #[arg(short = 't', long)]
    twitch_channel: Option<String>,

    /// Minimum votes required for chat consensus (default: 3)
    #[arg(long, default_value = "3")]
    chat_threshold: usize,

    /// Enable green screen background for streaming/recording
    #[arg(short, long)]
    live: bool,
}

fn main() {
    let args = Args::parse();

    // Load UI config (use defaults if not provided)
    let ui_config = if let Some(ref config_path) = args.ui_config {
        match UiConfig::from_file(config_path) {
            Ok(config) => {
                info!("Loaded UI config from {:?}", config_path);
                config
            }
            Err(e) => {
                warn!("Failed to load UI config from {:?}: {}. Using built-in defaults.", config_path, e);
                UiConfig::default()
            }
        }
    } else {
        info!("No UI config provided, using built-in defaults");
        UiConfig::default()
    };

    // Use green screen background if --live flag is set
    let background_color = if args.live {
        Color::srgb(0.0, 1.0, 0.0) // Chroma key green
    } else {
        ui_config.background_color()
    };
    let twitch_channel = args.twitch_channel.clone();
    let chat_threshold = args.chat_threshold;

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CGQ - Card Game Quiz".to_string(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(background_color))
        // Resources
        .insert_resource(args)
        .insert_resource(ui_config)
        .init_resource::<QuizState>()
        .init_resource::<GameTimer>()
        .init_resource::<Score>()
        .init_resource::<CardManager>()
        .init_resource::<GameState>()
        .init_resource::<CollectionManager>()
        .init_resource::<card_3d::SpawnedCards>()
        // Systems
        .add_systems(Startup, (setup, load_quiz, load_cards, card_3d::setup_3d_cards))
        .add_systems(Update, (
            quiz_system,
            card_effect_system,
            timer_system,
            input_system,
            ui_system,
            card_3d::spawn_cards_system,
            card_3d::update_card_positions,
        ));

    // Conditionally add chat integration
    if let Some(channel) = twitch_channel {
        info!("Enabling Twitch chat integration for channel: {} (threshold: {} votes)", channel, chat_threshold);
        app.add_plugins(chat_plugin::ChatPlugin {
            channel,
            answer_threshold: chat_threshold,
            card_threshold: chat_threshold * 2, // Card threshold is 2x answer threshold
        });
    }

    app.run();
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
    for (index, mut question) in question_set.questions.into_iter().enumerate() {
        question.question_index = index;
        let mut entity = commands.spawn(question);

        // Mark the first question as active
        if index == 0 {
            entity.insert(ActiveQuestion);
        }
    }
}

fn load_cards(
    mut card_manager: ResMut<CardManager>,
) {
    // Load all cards synchronously at startup
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    match runtime.block_on(cards::load_all_cards()) {
        Ok(cards) => {
            info!("Loaded {} cards", cards.len());
            card_manager.available_cards = cards;
        }
        Err(e) => {
            warn!("Failed to load cards: {}. Game will continue without cards.", e);
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
    asset_server: Res<AssetServer>,
    quiz_state: Res<QuizState>,
    score: Res<Score>,
    timer: Res<GameTimer>,
    ui_config: Res<UiConfig>,
    card_manager: Res<CardManager>,
    card_vote_tracker: Option<Res<chat_plugin::ChatCardVoteTracker>>,
    card_render_texture: Option<Res<card_3d::CardRenderTexture>>,
    args: Res<Args>,
    questions: Query<&Question, With<ActiveQuestion>>,
    existing_ui: Query<Entity, With<QuizUI>>,
) {
    // Clear existing UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Start screen
    if !quiz_state.game_started {
        let screen_bg = if args.live {
            Color::NONE // Transparent for green screen
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
        return;
    }

    // Game over screen
    if quiz_state.game_complete {
        let passed = score.current >= score.passing_grade;
        let screen_bg = if args.live {
            Color::NONE // Transparent for green screen
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
        return;
    }

    if let Ok(question) = questions.get_single() {
        // Collect card data once for the entire UI render
        let deployed_cards: Vec<CardDefinition> = card_manager.available_cards
            .iter()
            .filter(|card| card_manager.deployed_card_ids.contains(&card.id))
            .cloned()
            .collect();

        let all_cards = card_manager.available_cards.clone();
        let deployed_ids = card_manager.deployed_card_ids.clone();

        // Root container (transparent background)
        commands.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
            QuizUI,
        ))
        .with_children(|parent| {
            // Question box (top-left)
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

            // Timer and status bar (bottom-left)
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

            // Cards grid (bottom-right)
            let cbox = &ui_config.cards_grid;

            parent.spawn(NodeBundle {
                style: Style {
                    position_type: bevy::ui::PositionType::Absolute,
                    right: Val::Px(cbox.right),
                    bottom: Val::Px(cbox.bottom),
                    width: Val::Px(cbox.width),
                    height: Val::Px(cbox.height),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                ..default()
            }).with_children(|cards_box| {
                // Active Cards Section

                if !deployed_cards.is_empty() {
                    cards_box.spawn(TextBundle {
                        text: Text::from_section(
                            "ACTIVE EFFECTS",
                            TextStyle {
                                font_size: 18.0,
                                color: Color::srgb(0.9, 0.9, 1.0),
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                        ..default()
                    });

                    // Show deployed cards compactly
                    for card in &deployed_cards {
                        cards_box.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            background_color: Color::srgba(0.25, 0.25, 0.3, 0.8).into(),
                            border_color: Color::srgb(0.8, 0.8, 0.9).into(),
                            ..default()
                        }).with_children(|active_card| {
                            active_card.spawn(TextBundle {
                                text: Text::from_section(
                                    format!("✓ {}", card.name),
                                    TextStyle {
                                        font_size: 13.0,
                                        color: Color::srgb(0.95, 0.95, 1.0),
                                        ..default()
                                    },
                                ),
                                ..default()
                            });
                        });
                    }

                    // Separator
                    cards_box.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(4.0)),
                            ..default()
                        },
                        background_color: Color::srgba(0.4, 0.4, 0.5, 0.5).into(),
                        ..default()
                    });
                }

                // 3D Cards Portal - display the render texture with text overlays
                if let Some(render_tex) = card_render_texture {
                    cards_box.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: bevy::ui::PositionType::Relative,
                            ..default()
                        },
                        ..default()
                    }).with_children(|portal| {
                        // Background image
                        portal.spawn(ImageBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                position_type: bevy::ui::PositionType::Absolute,
                                ..default()
                            },
                            image: UiImage::new(render_tex.image_handle.clone()),
                            ..default()
                        });

                        // Text overlays for each card in 2x2 grid
                        // Positions based on 3D card layout mapped to UI space
                        // Adjusted for overlay size (35% wide, 45% tall) to center on cards
                        let card_positions = [
                            (13.75, 2.5),   // Top-left (index 0)
                            (51.25, 2.5),   // Top-right (index 1)
                            (13.75, 52.5),  // Bottom-left (index 2)
                            (51.25, 52.5),  // Bottom-right (index 3)
                        ];

                        for (index, card) in card_manager.available_cards.iter().take(4).enumerate() {
                            let (left_pct, top_pct) = card_positions[index];

                            portal.spawn(NodeBundle {
                                style: Style {
                                    position_type: bevy::ui::PositionType::Absolute,
                                    left: Val::Percent(left_pct),
                                    top: Val::Percent(top_pct),
                                    width: Val::Percent(35.0),  // Card width in UI space
                                    height: Val::Percent(45.0), // Card height in UI space
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::all(Val::Px(4.0)),
                                    ..default()
                                },
                                // Semi-transparent background for text readability
                                background_color: Color::srgba(0.0, 0.0, 0.0, 0.6).into(),
                                ..default()
                            }).with_children(|card_overlay| {
                                // Card name
                                card_overlay.spawn(TextBundle {
                                    text: Text::from_section(
                                        &card.name,
                                        TextStyle {
                                            font_size: 12.0,
                                            color: Color::srgb(1.0, 1.0, 1.0),
                                            ..default()
                                        },
                                    ),
                                    style: Style {
                                        margin: UiRect::bottom(Val::Px(2.0)),
                                        ..default()
                                    },
                                    ..default()
                                });

                                // Card description
                                if let Some(desc) = &card.description {
                                    card_overlay.spawn(TextBundle {
                                        text: Text::from_section(
                                            desc,
                                            TextStyle {
                                                font_size: 9.0,
                                                color: Color::srgb(0.9, 0.9, 0.9),
                                                ..default()
                                            },
                                        ),
                                        ..default()
                                    });
                                }
                            });
                        }
                    });
                }
            });
        });
    }
}

/// Marker component for UI
#[derive(Component)]
struct QuizUI;
