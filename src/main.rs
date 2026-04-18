use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;

mod card_3d;
mod card_templates;
mod cards;
mod collections;
mod components;
mod constants;
mod content_config;
mod deploy;
mod effect;
mod effect_executor;
mod game_state;
mod modes;
mod players;
mod resources;
mod session;
mod systems;
mod ui;
mod ui_config;
#[cfg(not(target_arch = "wasm32"))]
mod chat;
#[cfg(not(target_arch = "wasm32"))]
mod chat_plugin;
#[cfg(not(target_arch = "wasm32"))]
mod twitch;

use card_3d::Card3dPlugin;
use components::*;
use content_config::{load_app_config, AppConfig};
use deploy::CardPlugin;
use modes::ModesPlugin;
use players::PlayersPlugin;
use resources::*;
use session::StandaloneSessionPlugin;
use systems::QuizPlugin;
use ui::UiPlugin;

#[derive(Parser, Debug, Resource, Clone)]
#[command(name = "cgq")]
#[command(about = "Card Game Quiz Framework - A Bevy-based quiz game engine", long_about = None)]
pub struct Args {
    /// Directory containing the merged configuration tree
    #[arg(short = 'C', long, default_value = "examples/sample-quiz/etc")]
    pub config_dir: PathBuf,

    #[arg(short = 't', long)]
    twitch_channel: Option<String>,

    #[arg(long, default_value = "3")]
    chat_threshold: usize,

    #[arg(short, long)]
    pub live: bool,
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let args = Args::parse();
    let app_config = load_app_config(&args.config_dir).unwrap_or_else(|e| {
        panic!(
            "Failed to load configuration from {}: {:#}",
            args.config_dir.display(),
            e
        )
    });

    let window_title = app_config
        .game
        .title
        .clone()
        .unwrap_or_else(|| "CGQ - Card Game Quiz".to_string());

    let background_color = if args.live {
        let (r, g, b) = constants::CHROMA_KEY_GREEN;
        Color::srgb(r, g, b)
    } else {
        app_config.ui.background_color()
    };

    let quiz_state = QuizState {
        total_questions: app_config.questions.len(),
        ..default()
    };
    let score = Score {
        passing_grade: app_config
            .game
            .passing_grade
            .unwrap_or(Score::default().passing_grade),
        ..default()
    };
    let timer_duration = Duration::from_secs(
        app_config
            .game
            .timer_seconds
            .unwrap_or(GameTimer::default().timer.duration().as_secs()),
    );
    let game_timer = GameTimer {
        timer: Timer::new(timer_duration, TimerMode::Once),
        paused: false,
    };

    #[cfg(not(target_arch = "wasm32"))]
    let twitch_channel = args.twitch_channel.clone();
    #[cfg(not(target_arch = "wasm32"))]
    let chat_threshold = args.chat_threshold;

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: window_title,
            resolution: (1920.0, 1080.0).into(),
            ..default()
        }),
        ..default()
    }))
        .insert_resource(ClearColor(background_color))
        .insert_resource(args)
        .insert_resource(app_config.ui.clone())
        .insert_resource(AppConfigResource(app_config))
        .add_plugins((
            ModesPlugin,
            StandaloneSessionPlugin,
            QuizPlugin,
            CardPlugin,
            Card3dPlugin,
            UiPlugin,
            PlayersPlugin,
        ))
        .add_systems(Startup, (setup, load_content));

    // Seed the timer / score resources that QuizPlugin also init_resources —
    // these specific instances come from the quiz config. Insert them after
    // the plugin so we override the defaults.
    app.insert_resource(quiz_state)
        .insert_resource(game_timer)
        .insert_resource(score);

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(channel) = twitch_channel {
        info!(
            "Enabling Twitch chat integration for channel: {} (threshold: {} votes)",
            channel, chat_threshold
        );
        app.add_plugins(chat_plugin::ChatPlugin {
            channel,
            answer_threshold: chat_threshold,
            card_threshold: chat_threshold * 2,
        });
    }

    app.run();
}

#[derive(Resource)]
struct AppConfigResource(AppConfig);

fn load_content(
    mut commands: Commands,
    mut card_manager: ResMut<CardManager>,
    config: Res<AppConfigResource>,
) {
    let config = &config.0;
    card_manager.available_cards = cards::cards_from_configs(config.cards.clone());

    for (index, question) in cards::questions_from_configs(config.questions.clone())
        .into_iter()
        .enumerate()
    {
        let mut entity = commands.spawn(question);
        if index == 0 {
            entity.insert(ActiveQuestion);
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    info!("CGQ Game Started");
}
