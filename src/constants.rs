//! Constants used throughout the CGQ codebase.

// Card texture dimensions (pixels)
pub const CARD_WIDTH: u32 = 512;
pub const CARD_HEIGHT: u32 = 716;
pub const CARD_BORDER: u32 = 10;
pub const CARD_HEADER_HEIGHT: u32 = 70;
pub const CARD_DESC_HEIGHT: u32 = 160;

// 3D card mesh dimensions
pub const CARD_3D_WIDTH: f32 = 0.83;
pub const CARD_3D_HEIGHT: f32 = 1.16;
pub const CARD_3D_THICKNESS: f32 = 0.02;

// Grid layout
pub const CARD_GRID_X_SPACING: f32 = 0.95;
pub const CARD_GRID_Y_SPACING: f32 = 1.25;
pub const MAX_DISPLAYED_CARDS: usize = 4;

// UI card overlay positions (percentages)
pub const CARD_OVERLAY_POSITIONS: [(f32, f32); 4] = [
    (13.75, 2.5),   // Top-left
    (51.25, 2.5),   // Top-right
    (13.75, 52.5),  // Bottom-left
    (51.25, 52.5),  // Bottom-right
];

// Colors
pub const CHROMA_KEY_GREEN: (f32, f32, f32) = (0.0, 1.0, 0.0);
