use bevy::prelude::*;
use bevy::render::{
    mesh::{Indices, PrimitiveTopology},
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    view::RenderLayers,
};

use crate::constants::{CARD_3D_HEIGHT, CARD_3D_THICKNESS, CARD_3D_WIDTH};
use crate::resources::CardDefinition;

use super::components::Card3D;

const CARD_TEXTURE_DIR: &str = "share/textures/cards";

/// Spawn a 3D card mesh textured with the prebaked card PNG.
pub fn spawn_card_3d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    card: &CardDefinition,
    position: Vec3,
) {
    let mesh_handle = meshes.add(build_card_mesh());

    let texture_handle = load_card_texture(images, &card.name).unwrap_or_else(|| {
        warn!("No texture for card '{}', falling back to white", card.name);
        images.add(white_pixel())
    });

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture_handle),
        unlit: true,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material,
            transform: Transform::from_translation(position),
            ..default()
        },
        Card3D {
            card_id: card.id.clone(),
        },
        RenderLayers::layer(1),
    ));
}

fn build_card_mesh() -> Mesh {
    let hw = CARD_3D_WIDTH / 2.0;
    let hh = CARD_3D_HEIGHT / 2.0;
    let ht = CARD_3D_THICKNESS / 2.0;

    #[rustfmt::skip]
    let vertices = vec![
        [-hw, -hh,  ht], [ hw, -hh,  ht], [ hw,  hh,  ht], [-hw,  hh,  ht], // front
        [ hw, -hh, -ht], [-hw, -hh, -ht], [-hw,  hh, -ht], [ hw,  hh, -ht], // back
        [-hw,  hh,  ht], [ hw,  hh,  ht], [ hw,  hh, -ht], [-hw,  hh, -ht], // top
        [-hw, -hh, -ht], [ hw, -hh, -ht], [ hw, -hh,  ht], [-hw, -hh,  ht], // bottom
        [ hw, -hh,  ht], [ hw, -hh, -ht], [ hw,  hh, -ht], [ hw,  hh,  ht], // right
        [-hw, -hh, -ht], [-hw, -hh,  ht], [-hw,  hh,  ht], [-hw,  hh, -ht], // left
    ];

    #[rustfmt::skip]
    let uvs = vec![
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
    ];

    #[rustfmt::skip]
    let normals = vec![
        [0.0, 0.0,  1.0]; 4
    ].into_iter()
        .chain([[0.0, 0.0, -1.0]; 4])
        .chain([[0.0,  1.0, 0.0]; 4])
        .chain([[0.0, -1.0, 0.0]; 4])
        .chain([[ 1.0, 0.0, 0.0]; 4])
        .chain([[-1.0, 0.0, 0.0]; 4])
        .collect::<Vec<_>>();

    #[rustfmt::skip]
    let indices = vec![
        0, 1, 2,  0, 2, 3,
        4, 5, 6,  4, 6, 7,
        8, 9, 10, 8, 10, 11,
        12, 13, 14, 12, 14, 15,
        16, 17, 18, 16, 18, 19,
        20, 21, 22, 20, 22, 23,
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn load_card_texture(images: &mut Assets<Image>, card_name: &str) -> Option<Handle<Image>> {
    let path = format!("{}/{}.png", CARD_TEXTURE_DIR, card_name);
    let bytes = std::fs::read(&path)
        .inspect_err(|e| warn!("Failed to read {}: {}", path, e))
        .ok()?;
    let rgba = image::load_from_memory(&bytes)
        .inspect_err(|e| warn!("Failed to decode {}: {}", path, e))
        .ok()?
        .to_rgba8();

    let (width, height) = rgba.dimensions();
    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    Some(images.add(image))
}

fn white_pixel() -> Image {
    Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
