use bevy::prelude::*;
use bevy::text::{Text, TextStyle};
use bevy::ui::{FlexDirection, Style, UiRect, Val};

use crate::card_3d::CardRenderTexture;
use crate::resources::CardDefinition;
use crate::ui_config::CardsGridConfig;

/// Render the cards grid section (bottom-right)
pub fn render_cards_section(
    parent: &mut ChildBuilder,
    cbox: &CardsGridConfig,
    deployed_cards: &[CardDefinition],
    card_render_texture: Option<&CardRenderTexture>,
) {
    parent.spawn(NodeBundle {
        style: Style {
            position_type: bevy::ui::PositionType::Absolute,
            right: Val::Px(cbox.right),
            bottom: Val::Px(cbox.bottom),
            width: Val::Px(cbox.width),
            height: Val::Px(cbox.height),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            ..default()
        },
        ..default()
    }).with_children(|cards_box| {
        render_active_effects(cards_box, deployed_cards);
        render_3d_portal(cards_box, card_render_texture);
    });
}

/// Render the active effects section
fn render_active_effects(
    cards_box: &mut ChildBuilder,
    deployed_cards: &[CardDefinition],
) {
    if deployed_cards.is_empty() {
        return;
    }

    cards_box.spawn(TextBundle {
        text: Text::from_section(
            "ACTIVE EFFECTS",
            TextStyle {
                font_size: 18.0,
                color: Color::srgb(0.9, 0.9, 1.0),
                ..default()
            },
        ),
        style: Style {
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
        ..default()
    });

    // Show deployed cards compactly
    for card in deployed_cards {
        cards_box.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::srgba(0.25, 0.25, 0.3, 0.8).into(),
            border_color: Color::srgb(0.8, 0.8, 0.9).into(),
            ..default()
        }).with_children(|active_card| {
            active_card.spawn(TextBundle {
                text: Text::from_section(
                    format!("✓ {}", card.name),
                    TextStyle {
                        font_size: 13.0,
                        color: Color::srgb(0.95, 0.95, 1.0),
                        ..default()
                    },
                ),
                ..default()
            });
        });
    }

    // Separator
    cards_box.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        },
        background_color: Color::srgba(0.4, 0.4, 0.5, 0.5).into(),
        ..default()
    });
}

/// Render the 3D cards portal. Card art/text is fully baked into the prebaked
/// textures, so this is just a single image pulled from the render target.
fn render_3d_portal(
    cards_box: &mut ChildBuilder,
    card_render_texture: Option<&CardRenderTexture>,
) {
    let Some(render_tex) = card_render_texture else { return };

    cards_box.spawn(ImageBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        image: UiImage::new(render_tex.image_handle.clone()),
        ..default()
    });
}
