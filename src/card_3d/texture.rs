use crate::components::CardType;
use crate::constants::{
    CARD_WIDTH, CARD_HEIGHT, CARD_BORDER, CARD_HEADER_HEIGHT, CARD_DESC_HEIGHT,
};
use crate::resources::CardDefinition;
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use ab_glyph::{FontRef, PxScale};

/// Get colors for a card based on its type
pub fn get_card_colors(card_type: &CardType) -> (Rgba<u8>, Rgba<u8>, Rgba<u8>) {
    let header_bg_color = match card_type {
        CardType::Resistance => Rgba([40u8, 80u8, 40u8, 255u8]),
        CardType::Palestinian => Rgba([80u8, 60u8, 40u8, 255u8]),
        CardType::Politics => Rgba([40u8, 40u8, 80u8, 255u8]),
        CardType::Negative => Rgba([100u8, 30u8, 30u8, 255u8]),
        CardType::IDF => Rgba([60u8, 40u8, 80u8, 255u8]),
        CardType::Hasbara => Rgba([80u8, 70u8, 50u8, 255u8]),
        CardType::Ceasefire => Rgba([40u8, 60u8, 100u8, 255u8]),
        CardType::Other => Rgba([60u8, 60u8, 60u8, 255u8]),
    };

    let border_color = match card_type {
        CardType::Resistance => Rgba([30u8, 60u8, 30u8, 255u8]),
        CardType::Palestinian => Rgba([60u8, 45u8, 30u8, 255u8]),
        CardType::Politics => Rgba([30u8, 30u8, 60u8, 255u8]),
        CardType::Negative => Rgba([80u8, 20u8, 20u8, 255u8]),
        CardType::IDF => Rgba([45u8, 30u8, 60u8, 255u8]),
        CardType::Hasbara => Rgba([60u8, 50u8, 35u8, 255u8]),
        CardType::Ceasefire => Rgba([30u8, 45u8, 80u8, 255u8]),
        CardType::Other => Rgba([45u8, 45u8, 45u8, 255u8]),
    };

    let desc_bg_color = Rgba([220u8, 240u8, 80u8, 255u8]);

    (header_bg_color, border_color, desc_bg_color)
}

/// Draw the colored background regions for a card
pub fn draw_card_backgrounds(
    img: &mut RgbaImage,
    card_width: u32,
    card_height: u32,
    border: u32,
    header_height: u32,
    desc_height: u32,
    header_bg_color: Rgba<u8>,
    border_color: Rgba<u8>,
    desc_bg_color: Rgba<u8>,
) {
    let header_y = border;
    let artwork_y = border + header_height;
    let artwork_height = card_height - (border * 2 + header_height + desc_height);
    let desc_y = artwork_y + artwork_height;

    // Fill entire card with border color
    draw_filled_rect_mut(img, Rect::at(0, 0).of_size(card_width, card_height), border_color);

    // Draw dark background for header area
    draw_filled_rect_mut(
        img,
        Rect::at(border as i32, header_y as i32).of_size(card_width - border * 2, header_height),
        header_bg_color,
    );

    // Draw bright yellow background for description area
    draw_filled_rect_mut(
        img,
        Rect::at(border as i32, desc_y as i32).of_size(card_width - border * 2, desc_height),
        desc_bg_color,
    );
}

/// Generate a composite card texture with text baked in at runtime
pub fn generate_card_texture(
    card: &CardDefinition,
    artwork_path: Option<&str>,
) -> bevy::render::texture::Image {
    // Card dimensions in pixels
    let card_width = CARD_WIDTH;
    let card_height = CARD_HEIGHT;

    // Layout regions
    let border = CARD_BORDER;
    let header_height = CARD_HEADER_HEIGHT;
    let desc_height = CARD_DESC_HEIGHT;

    let header_y = border;
    let artwork_y = border + header_height;
    let artwork_height = card_height - (border * 2 + header_height + desc_height);
    let desc_y = artwork_y + artwork_height;

    // Get colors for this card type
    let (header_bg_color, border_color, desc_bg_color) = get_card_colors(&card.card_type);

    // Create base image and draw backgrounds
    let mut img = RgbaImage::new(card_width, card_height);
    draw_card_backgrounds(
        &mut img,
        card_width,
        card_height,
        border,
        header_height,
        desc_height,
        header_bg_color,
        border_color,
        desc_bg_color,
    );

    // Load and composite character artwork in the middle section
    if let Some(path) = artwork_path {
        let full_path = format!("assets/{}", path);

        if let Ok(artwork_img) = image::open(&full_path) {
            let artwork_rgba = artwork_img.to_rgba8();

            // Resize artwork to fit in the middle section
            let resized = image::imageops::resize(
                &artwork_rgba,
                card_width - border * 2,
                artwork_height,
                image::imageops::FilterType::Lanczos3,
            );

            // Place the artwork in the middle section
            image::imageops::overlay(
                &mut img,
                &resized,
                border as i64,
                artwork_y as i64
            );
        }
    }

    // Text positioning
    let header_text_y = header_y + 18;
    let desc_text_y = desc_y + 12;

    // Load embedded font for text rendering
    let font_data = include_bytes!("../../assets/fonts/FiraSans-Bold.ttf");
    if let Ok(font) = FontRef::try_from_slice(font_data) {
        // Draw card name in header area with BLACK text
        let header_text_color = Rgba([20u8, 20u8, 20u8, 255u8]);
        let header_scale = PxScale::from(30.0);
        draw_text_mut(
            &mut img,
            header_text_color,
            (border + 8) as i32,
            header_text_y as i32,
            header_scale,
            &font,
            &card.name,
        );

        // Draw description text with BLACK text
        if let Some(desc) = &card.description {
            let desc_text_color = Rgba([20u8, 20u8, 20u8, 255u8]);
            let desc_scale = PxScale::from(16.0);
            let text_x = (border + 10) as i32;

            // Word wrap description to fit card width
            let wrapped = textwrap::wrap(desc, 48);
            for (i, line) in wrapped.iter().take(8).enumerate() {
                draw_text_mut(
                    &mut img,
                    desc_text_color,
                    text_x,
                    (desc_text_y + i as u32 * 19) as i32,
                    desc_scale,
                    &font,
                    line,
                );
            }
        }
    }

    // Convert to Bevy Image format
    bevy::render::texture::Image::new(
        bevy::render::render_resource::Extent3d {
            width: card_width,
            height: card_height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        img.into_raw(),
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
}
