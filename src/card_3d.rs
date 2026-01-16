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
    use bevy::render::camera::OrthographicProjection;

    // Orthographic camera positioned to project cards to bottom-right
    // Using asymmetric frustum to offset the view
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 1,
                clear_color: bevy::render::camera::ClearColorConfig::None,
                ..default()
            },
            projection: bevy::render::camera::Projection::Orthographic(OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal(8.0),
                ..default()
            }),
            // Position camera to the right and down so cards appear bottom-right
            transform: Transform::from_xyz(3.0, -2.0, 5.0)
                .looking_at(Vec3::new(3.0, -2.0, 0.0), Vec3::Y),
            ..default()
        },
        CardsCamera,
        RenderLayers::layer(1),
    ));

    // Add strong directional lighting for the cards
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(3.0, 0.0, 1.0).looking_at(Vec3::new(3.0, -2.0, 0.0), Vec3::Y),
            ..default()
        },
        RenderLayers::layer(1),
    ));

    // Additional point light for extra brightness
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 8000.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(3.0, -2.0, 4.0),
            ..default()
        },
        RenderLayers::layer(1),
    ));

    // Strong ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
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

    // Create PBR material for paper-like appearance with emissive lighting
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: texture_handle.clone(),
        emissive: Color::srgb(0.5, 0.5, 0.5).into(), // Make cards emit light so they're visible
        emissive_texture: texture_handle, // Use the same texture for emissive
        perceptual_roughness: 0.8, // Paper is fairly rough
        metallic: 0.0,              // Paper is not metallic
        reflectance: 0.1,           // Very low reflectance
        unlit: false,
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
    // Only show first 4 cards in a 2x2 grid
    let max_cards = 4;

    for (index, card) in card_manager.available_cards.iter().take(max_cards).enumerate() {
        if !spawned_cards.card_ids.contains(&card.id) {
            // 2x2 grid layout centered where camera is looking
            let row = index / 2;
            let col = index % 2;

            // Grid centered around (3.0, -2.0) where camera is looking
            let x_offset = 3.0 + (col as f32 - 0.5) * 0.9;
            let y_offset = -2.0 + (0.5 - row as f32) * 1.2;

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
