use bevy::prelude::*;

/// Marker component for 3D card entities
#[derive(Component)]
pub struct Card3D {
    pub card_id: String,
}

/// Marker component for the cards 3D viewport camera
#[derive(Component)]
pub struct CardsCamera;

/// Resource to hold the render texture handle
#[derive(Resource)]
pub struct CardRenderTexture {
    pub image_handle: Handle<Image>,
}

/// Resource to track which cards have been spawned
#[derive(Resource, Default)]
pub struct SpawnedCards {
    pub card_ids: Vec<String>,
}
