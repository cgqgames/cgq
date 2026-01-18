use bevy::prelude::*;
use bevy::ui::{Style, Val, UiRect, FlexDirection, JustifyContent, AlignItems};
use bevy::text::{Text, TextStyle};
use crate::resources::CardDefinition;
use crate::card_3d::CardRenderTexture;
use crate::ui_config::CardsGridConfig;
use crate::constants::{CARD_OVERLAY_POSITIONS, MAX_DISPLAYED_CARDS};

/// Render the cards grid section (bottom-right)
pub fn render_cards_section(
    parent: &mut ChildBuilder,
    cbox: &CardsGridConfig,
    deployed_cards: &[CardDefinition],
    available_cards: &[CardDefinition],
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
        render_3d_portal(cards_box, available_cards, card_render_texture);
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

/// Render the 3D cards portal with text overlays
fn render_3d_portal(
    cards_box: &mut ChildBuilder,
    available_cards: &[CardDefinition],
    card_render_texture: Option<&CardRenderTexture>,
) {
    let Some(render_tex) = card_render_texture else {
        return;
    };

    cards_box.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: bevy::ui::PositionType::Relative,
            ..default()
        },
        ..default()
    }).with_children(|portal| {
        // Background image from 3D render
        portal.spawn(ImageBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: bevy::ui::PositionType::Absolute,
                ..default()
            },
            image: UiImage::new(render_tex.image_handle.clone()),
            ..default()
        });

        // Text overlays for each card in 2x2 grid
        for (index, card) in available_cards.iter().take(MAX_DISPLAYED_CARDS).enumerate() {
            let (left_pct, top_pct) = CARD_OVERLAY_POSITIONS[index];
            render_card_overlay(portal, card, left_pct, top_pct);
        }
    });
}

/// Render a single card's text overlay
fn render_card_overlay(
    portal: &mut ChildBuilder,
    card: &CardDefinition,
    left_pct: f32,
    top_pct: f32,
) {
    portal.spawn(NodeBundle {
        style: Style {
            position_type: bevy::ui::PositionType::Absolute,
            left: Val::Percent(left_pct),
            top: Val::Percent(top_pct),
            width: Val::Percent(35.0),
            height: Val::Percent(45.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        background_color: Color::srgba(0.0, 0.0, 0.0, 0.6).into(),
        ..default()
    }).with_children(|card_overlay| {
        // Card name
        card_overlay.spawn(TextBundle {
            text: Text::from_section(
                &card.name,
                TextStyle {
                    font_size: 12.0,
                    color: Color::srgb(1.0, 1.0, 1.0),
                    ..default()
                },
            ),
            style: Style {
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            ..default()
        });

        // Card description
        if let Some(desc) = &card.description {
            card_overlay.spawn(TextBundle {
                text: Text::from_section(
                    desc,
                    TextStyle {
                        font_size: 9.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
                ..default()
            });
        }
    });
}
