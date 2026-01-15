use bevy::prelude::*;

mod components;
mod resources;
mod systems;
mod cards;

use components::*;
use resources::*;
use systems::*;

fn main() {
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
        .init_resource::<QuizState>()
        .init_resource::<GameTimer>()
        .init_resource::<Score>()
        .init_resource::<CardManager>()
        // Systems
        .add_systems(Startup, setup)
        .add_systems(Update, (
            quiz_system,
            card_effect_system,
            timer_system,
            input_system,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Load quiz from YAML
    // TODO: Load questions and cards from content/ directory

    info!("CGQ Game Started");
}
