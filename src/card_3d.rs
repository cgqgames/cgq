use bevy::prelude::*;
use crate::resources::CardDefinition;
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use ab_glyph::{FontRef, PxScale};

/// Marker component for 3D card entities
#[derive(Component)]
pub struct Card3D {
    pub card_id: String,
}

/// Generate a composite card texture with text baked in at runtime
fn generate_card_texture(
    card: &CardDefinition,
    artwork_image: Option<&bevy::render::texture::Image>,
) -> bevy::render::texture::Image {
    // Card dimensions in pixels
    let card_width = 512;
    let card_height = 716; // Roughly 63:88 aspect ratio (card proportions)

    // Create base image with card background color (beige/tan)
    let mut img = RgbaImage::new(card_width, card_height);
    let bg_color = Rgba([235u8, 225u8, 200u8, 255u8]); // Beige card stock

    // Fill background
    draw_filled_rect_mut(
        &mut img,
        Rect::at(0, 0).of_size(card_width, card_height),
        bg_color,
    );

    // Layout regions
    let border = 20;
    let header_height = 60;
    let desc_height = 160;
    let artwork_y = border + header_height;
    let artwork_height = card_height - (border * 2 + header_height + desc_height);

    // Draw border around artwork area
    let artwork_rect = Rect::at(border as i32, artwork_y as i32)
        .of_size(card_width - border * 2, artwork_height);
    draw_filled_rect_mut(&mut img, artwork_rect, Rgba([255, 255, 255, 255]));

    // Composite artwork if available
    if let Some(artwork) = artwork_image {
        // Convert Bevy image to image-rs format and composite
        let artwork_data = &artwork.data;
        if let Ok(artwork_img) = image::load_from_memory(artwork_data) {
            let artwork_rgba = artwork_img.to_rgba8();

            // Resize to fit artwork area while maintaining aspect ratio
            let resized = image::imageops::resize(
                &artwork_rgba,
                card_width - border * 2,
                artwork_height,
                image::imageops::FilterType::Lanczos3,
            );

            // Center the artwork
            let x_offset = border;
            let y_offset = artwork_y;
            image::imageops::overlay(&mut img, &resized, x_offset as i64, y_offset as i64);
        }
    }

    // Load embedded font for text rendering (using Bevy's default font data)
    // We'll use a simple embedded font
    let font_data = include_bytes!("../assets/fonts/FiraSans-Bold.ttf");
    if let Ok(font) = FontRef::try_from_slice(font_data) {
        let text_color = Rgba([20u8, 20u8, 20u8, 255u8]);

        // Draw card name (header)
        let header_scale = PxScale::from(32.0);
        draw_text_mut(
            &mut img,
            text_color,
            border as i32,
            (border + 15) as i32,
            header_scale,
            &font,
            &card.name,
        );

        // Draw description (bottom section)
        if let Some(desc) = &card.description {
            let desc_scale = PxScale::from(20.0);
            let desc_y = (artwork_y + artwork_height + 20) as i32;

            // Word wrap description
            let wrapped = textwrap::wrap(desc, 40);
            for (i, line) in wrapped.iter().take(6).enumerate() {
                draw_text_mut(
                    &mut img,
                    text_color,
                    border as i32,
                    desc_y + (i as i32 * 24),
                    desc_scale,
                    &font,
                    line,
                );
            }
        }
    }

    // Convert to Bevy Image format
    let bevy_img = bevy::render::texture::Image::new(
        bevy::render::render_resource::Extent3d {
            width: card_width,
            height: card_height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        img.into_raw(),
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    bevy_img
}

/// Marker component for the cards 3D viewport camera
#[derive(Component)]
pub struct CardsCamera;

/// Resource to hold the render texture handle
#[derive(Resource)]
pub struct CardRenderTexture {
    pub image_handle: Handle<Image>,
}

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

/// Spawn a 3D card mesh
pub fn spawn_card_3d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    asset_server: &Res<AssetServer>,
    card: &CardDefinition,
    position: Vec3,
) {
    use bevy::render::mesh::{Indices, PrimitiveTopology};
    use bevy::render::render_asset::RenderAssetUsages;

    // Create a thin 3D card mesh (for flip animations and back texturing)
    let width = 0.83;   // 0.72 * 1.15 (15% bigger)
    let height = 1.16;  // 1.01 * 1.15 (15% bigger)
    let thickness = 0.02;  // Thin like a real card

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    let hw = width / 2.0;
    let hh = height / 2.0;
    let ht = thickness / 2.0;

    // 24 vertices (4 per face, 6 faces) for proper normals and UVs
    let vertices = vec![
        // Front face (facing +Z) - 0-3
        [-hw, -hh,  ht], [hw, -hh,  ht], [hw,  hh,  ht], [-hw,  hh,  ht],
        // Back face (facing -Z) - 4-7
        [hw, -hh, -ht], [-hw, -hh, -ht], [-hw,  hh, -ht], [hw,  hh, -ht],
        // Top edge (facing +Y) - 8-11
        [-hw,  hh,  ht], [hw,  hh,  ht], [hw,  hh, -ht], [-hw,  hh, -ht],
        // Bottom edge (facing -Y) - 12-15
        [-hw, -hh, -ht], [hw, -hh, -ht], [hw, -hh,  ht], [-hw, -hh,  ht],
        // Right edge (facing +X) - 16-19
        [hw, -hh,  ht], [hw, -hh, -ht], [hw,  hh, -ht], [hw,  hh,  ht],
        // Left edge (facing -X) - 20-23
        [-hw, -hh, -ht], [-hw, -hh,  ht], [-hw,  hh,  ht], [-hw,  hh, -ht],
    ];

    // UVs - front face texture only covers upper ~50% of card (artwork area)
    // Bottom portion and borders will be card background color
    let uvs = vec![
        // Front face - texture mapped to upper portion only
        // Using UVs > 1.0 to map texture to just the upper middle area
        // Actual card layout:
        //   Top 15%: Header text area (background color)
        //   Middle 50%: Artwork (textured)
        //   Bottom 35%: Description area (background color)
        // We'll use vertex colors or a separate texture for the full card later
        // For now, map texture to cover most of the card, text will overlay
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        // Back face - flipped for card back texture (will add later)
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        // Top edge
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        // Bottom edge
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        // Right edge
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        // Left edge
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
    ];

    // Normals for each face
    let normals = vec![
        // Front face
        [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
        // Back face
        [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
        // Top edge
        [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
        // Bottom edge
        [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
        // Right edge
        [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
        // Left edge
        [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
    ];

    // Triangle indices for all 6 faces
    #[rustfmt::skip]
    let indices = vec![
        // Front face
        0, 1, 2,  0, 2, 3,
        // Back face
        4, 5, 6,  4, 6, 7,
        // Top edge
        8, 9, 10,  8, 10, 11,
        // Bottom edge
        12, 13, 14,  12, 14, 15,
        // Right edge
        16, 17, 18,  16, 18, 19,
        // Left edge
        20, 21, 22,  20, 22, 23,
    ];

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    let card_mesh = meshes.add(mesh);

    // Generate composite card texture with text baked in
    // TODO: Load artwork asynchronously and regenerate texture when ready
    // For now, generate without artwork
    let composite_image = generate_card_texture(card, None);

    // Add the composite texture to assets
    let texture_handle = images.add(composite_image);

    // Create PBR material with composite texture
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture_handle),
        emissive: Color::srgb(0.0, 0.0, 0.0).into(),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        reflectance: 0.2,
        unlit: false,
        ..default()
    });

    use bevy::render::view::RenderLayers;

    // No rotation needed - custom UVs handle orientation
    let transform = Transform::from_translation(position);

    commands.spawn((
        PbrBundle {
            mesh: card_mesh,
            material,
            transform,
            ..default()
        },
        Card3D {
            card_id: card.id.clone(),
        },
        RenderLayers::layer(1),
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
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    card_manager: Res<crate::resources::CardManager>,
    mut spawned_cards: ResMut<SpawnedCards>,
) {
    // Only show first 4 cards in a 2x2 grid
    let max_cards = 4;

    for (index, card) in card_manager.available_cards.iter().take(max_cards).enumerate() {
        if !spawned_cards.card_ids.contains(&card.id) {
            // 2x2 grid layout centered at origin
            let row = index / 2;
            let col = index % 2;

            // Grid centered at (0, 0, 0) - more spacing for larger cards
            let x_offset = (col as f32 - 0.5) * 0.95;  // Increased from 0.75
            let y_offset = (0.5 - row as f32) * 1.25;  // Increased from 1.0

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
