use bevy::prelude::*;
use crate::constants::{CARD_3D_WIDTH, CARD_3D_HEIGHT, CARD_3D_THICKNESS};
use crate::resources::CardDefinition;
use super::components::Card3D;
use super::texture::generate_card_texture;

/// Spawn a 3D card mesh
pub fn spawn_card_3d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    _asset_server: &Res<AssetServer>,
    card: &CardDefinition,
    position: Vec3,
) {
    use bevy::render::mesh::{Indices, PrimitiveTopology};
    use bevy::render::render_asset::RenderAssetUsages;

    // Create a thin 3D card mesh (for flip animations and back texturing)
    let width = CARD_3D_WIDTH;
    let height = CARD_3D_HEIGHT;
    let thickness = CARD_3D_THICKNESS;

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

    // UVs - front face texture maps the card texture
    let uvs = vec![
        // Front face
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        // Back face - flipped for card back texture
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
    let artwork_path = card.image_path.as_deref();
    let composite_image = generate_card_texture(card, artwork_path);

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
