use bevy::prelude::*;
use crate::constants::{CARD_GRID_X_SPACING, CARD_GRID_Y_SPACING, MAX_DISPLAYED_CARDS};
use super::components::{Card3D, CardsCamera, CardRenderTexture, SpawnedCards};
use super::mesh::spawn_card_3d;

/// Setup the 3D cards rendering system with render-to-texture
pub fn setup_3d_cards(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    use bevy::render::view::RenderLayers;
    use bevy::render::camera::{OrthographicProjection, RenderTarget};
    use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

    // Create a render target texture (512x512 for card portal)
    let size = Extent3d {
        width: 512,
        height: 512,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);

    // Store the handle in a resource for UI to access
    commands.insert_resource(CardRenderTexture {
        image_handle: image_handle.clone(),
    });

    // Orthographic camera that renders to the texture
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: -1, // Render before main camera
                target: RenderTarget::Image(image_handle),
                ..default()
            },
            projection: bevy::render::camera::Projection::Orthographic(OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::FixedHorizontal(2.0),
                ..default()
            }),
            // Camera faces cards straight-on
            transform: Transform::from_xyz(0.0, 0.0, 5.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
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
            transform: Transform::from_xyz(0.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
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
            transform: Transform::from_xyz(0.0, 0.0, 4.0),
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

/// System to spawn 3D cards when they are loaded
pub fn spawn_cards_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    card_manager: Res<crate::resources::CardManager>,
    mut spawned_cards: ResMut<SpawnedCards>,
) {
    for (index, card) in card_manager.available_cards.iter().take(MAX_DISPLAYED_CARDS).enumerate() {
        if !spawned_cards.card_ids.contains(&card.id) {
            // 2x2 grid layout centered at origin
            let row = index / 2;
            let col = index % 2;

            // Grid centered at (0, 0, 0)
            let x_offset = (col as f32 - 0.5) * CARD_GRID_X_SPACING;
            let y_offset = (0.5 - row as f32) * CARD_GRID_Y_SPACING;

            let position = Vec3::new(x_offset, y_offset, 0.0);

            spawn_card_3d(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
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
