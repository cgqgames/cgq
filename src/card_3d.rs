use bevy::prelude::*;
use crate::resources::CardDefinition;

/// Marker component for 3D card entities
#[derive(Component)]
pub struct Card3D {
    pub card_id: String,
}

/// Marker component for the cards 3D viewport camera
#[derive(Component)]
pub struct CardsCamera;

/// Setup the 3D cards rendering system
pub fn setup_3d_cards(
    mut commands: Commands,
) {
    use bevy::render::view::RenderLayers;

    // Add a 3D camera for the cards
    // Position it to look at cards in 3D space
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 1, // Render after the main 2D camera
                clear_color: bevy::render::camera::ClearColorConfig::None, // Don't clear, overlay on 2D
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.5, 4.0)
                .looking_at(Vec3::new(0.0, -0.5, 0.0), Vec3::Y),
            ..default()
        },
        CardsCamera,
        RenderLayers::layer(1), // Render layer 1 for cards
    ));

    // Add lighting for the cards
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 2000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 4.0, 4.0),
            ..default()
        },
        RenderLayers::layer(1), // Illuminate cards on layer 1
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });
}

/// Spawn a 3D card mesh
pub fn spawn_card_3d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    card: &CardDefinition,
    position: Vec3,
) {
    // Create a thin card mesh (like a playing card)
    let card_mesh = meshes.add(Cuboid::new(
        0.63,  // Width (standard playing card ratio)
        0.88,  // Height
        0.01,  // Thickness (very thin like paper)
    ));

    // Load texture if available
    let texture_handle = if let Some(ref image_path) = card.image_path {
        Some(asset_server.load(image_path.clone()))
    } else {
        None
    };

    // Create PBR material for paper-like appearance
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.95, 0.98), // Slight off-white for paper
        base_color_texture: texture_handle,
        perceptual_roughness: 0.8, // Paper is fairly rough
        metallic: 0.0,              // Paper is not metallic
        reflectance: 0.1,           // Very low reflectance
        ..default()
    });

    use bevy::render::view::RenderLayers;

    commands.spawn((
        PbrBundle {
            mesh: card_mesh,
            material,
            transform: Transform::from_translation(position),
            ..default()
        },
        Card3D {
            card_id: card.id.clone(),
        },
        RenderLayers::layer(1), // Same layer as cards camera
    ));
}

/// Resource to track which cards have been spawned
#[derive(Resource, Default)]
pub struct SpawnedCards {
    pub card_ids: Vec<String>,
}

/// System to spawn 3D cards when they are loaded
pub fn spawn_cards_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    card_manager: Res<crate::resources::CardManager>,
    mut spawned_cards: ResMut<SpawnedCards>,
) {
    // Check if there are new cards to spawn
    for (index, card) in card_manager.available_cards.iter().enumerate() {
        if !spawned_cards.card_ids.contains(&card.id) {
            // Calculate position in a grid layout
            let x_offset = (index % 3) as f32 * 0.8 - 0.8; // 3 cards per row
            let y_offset = -((index / 3) as f32 * 1.0);

            let position = Vec3::new(x_offset, y_offset, 0.0);

            spawn_card_3d(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                card,
                position,
            );

            spawned_cards.card_ids.push(card.id.clone());
        }
    }
}

/// Update card positions based on game state
pub fn update_card_positions(
    mut card_query: Query<(&Card3D, &mut Transform)>,
    card_manager: Res<crate::resources::CardManager>,
) {
    // Update positions for deployed cards (e.g., glow effect, raise them slightly)
    for (card_3d, mut transform) in card_query.iter_mut() {
        if card_manager.deployed_card_ids.contains(&card_3d.card_id) {
            // Raise deployed cards slightly
            transform.translation.z = 0.1;
        } else {
            transform.translation.z = 0.0;
        }
    }
}
