use bevy::prelude::*;
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
mod constants;
mod ui;

use components::*;
use resources::*;
use systems::*;
use game_state::GameState;
use collections::CollectionManager;
use ui_config::UiConfig;

#[derive(Parser, Debug, Resource, Clone)]
#[command(name = "cgq")]
#[command(about = "Card Game Quiz Framework - A Bevy-based quiz game engine", long_about = None)]
pub struct Args {
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
    pub live: bool,
}

fn main() {
    let args = Args::parse();

    // Load UI config (use defaults if not provided)
    let ui_config = load_ui_config(&args);

    // Use green screen background if --live flag is set
    let background_color = if args.live {
        let (r, g, b) = constants::CHROMA_KEY_GREEN;
        Color::srgb(r, g, b)
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
            ui::ui_system,
            card_3d::spawn_cards_system,
            card_3d::update_card_positions,
        ));

    // Conditionally add chat integration
    if let Some(channel) = twitch_channel {
        info!("Enabling Twitch chat integration for channel: {} (threshold: {} votes)", channel, chat_threshold);
        app.add_plugins(chat_plugin::ChatPlugin {
            channel,
            answer_threshold: chat_threshold,
            card_threshold: chat_threshold * 2,
        });
    }

    app.run();
}

fn load_ui_config(args: &Args) -> UiConfig {
    if let Some(ref config_path) = args.ui_config {
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
    }
}

fn load_quiz(
    mut commands: Commands,
    mut quiz_state: ResMut<QuizState>,
    args: Res<Args>,
) {
    let yaml_content = std::fs::read_to_string(&args.quiz)
        .unwrap_or_else(|e| panic!("Failed to load questions from {:?}: {}", args.quiz, e));

    let question_set: cards::QuestionSet = serde_yaml::from_str(&yaml_content)
        .unwrap_or_else(|e| panic!("Failed to parse questions YAML: {}", e));

    quiz_state.total_questions = question_set.questions.len();

    info!("Loaded {} questions from {:?}", question_set.questions.len(), args.quiz);

    for (index, mut question) in question_set.questions.into_iter().enumerate() {
        question.question_index = index;
        let mut entity = commands.spawn(question);

        if index == 0 {
            entity.insert(ActiveQuestion);
        }
    }
}

fn load_cards(mut card_manager: ResMut<CardManager>) {
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
    commands.spawn(Camera2dBundle::default());
    info!("CGQ Game Started");
}
