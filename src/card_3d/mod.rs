mod components;
mod mesh;
mod systems;

use bevy::prelude::*;

pub use components::{CardRenderTexture, SpawnedCards};

pub struct Card3dPlugin;

impl Plugin for Card3dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnedCards>()
            .add_systems(Startup, systems::setup_3d_cards)
            .add_systems(Update, (systems::spawn_cards_system, systems::update_card_positions));
    }
}
