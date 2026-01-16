use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct UiConfig {
    pub layout: LayoutConfig,
    pub question_box: QuestionBoxConfig,
    pub timer_box: TimerBoxConfig,
    pub cards_grid: CardsGridConfig,
    pub colors: ColorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub preset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionBoxConfig {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub font_size: f32,
    pub header_font_size: f32,
    pub option_font_size: f32,
    pub background_alpha: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerBoxConfig {
    pub left: f32,
    pub bottom: f32,
    pub timer_font_size: f32,
    pub status_font_size: f32,
    pub background_alpha: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardsGridConfig {
    pub right: f32,
    pub bottom: f32,
    pub width: f32,
    pub height: f32,
    pub background_alpha: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub background: [f32; 3],
    pub accent: [f32; 3],
    pub timer: [f32; 3],
    pub text_primary: [f32; 3],
    pub text_secondary: [f32; 3],
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            layout: LayoutConfig {
                preset: "stream-overlay".to_string(),
            },
            question_box: QuestionBoxConfig {
                left: 20.0,
                top: 20.0,
                width: 650.0,
                font_size: 20.0,
                header_font_size: 24.0,
                option_font_size: 18.0,
                background_alpha: 0.95,
            },
            timer_box: TimerBoxConfig {
                left: 20.0,
                bottom: 20.0,
                timer_font_size: 60.0,
                status_font_size: 16.0,
                background_alpha: 0.95,
            },
            cards_grid: CardsGridConfig {
                right: 20.0,
                bottom: 20.0,
                width: 500.0,
                height: 400.0,
                background_alpha: 0.8,
            },
            colors: ColorConfig {
                background: [0.08, 0.05, 0.12],
                accent: [0.3, 0.9, 0.8],
                timer: [0.2, 0.9, 0.9],
                text_primary: [1.0, 1.0, 1.0],
                text_secondary: [0.9, 0.9, 0.95],
            },
        }
    }
}

impl UiConfig {
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: UiConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn background_color(&self) -> Color {
        Color::srgb(
            self.colors.background[0],
            self.colors.background[1],
            self.colors.background[2],
        )
    }

    pub fn accent_color(&self) -> Color {
        Color::srgb(
            self.colors.accent[0],
            self.colors.accent[1],
            self.colors.accent[2],
        )
    }

    pub fn timer_color(&self) -> Color {
        Color::srgb(
            self.colors.timer[0],
            self.colors.timer[1],
            self.colors.timer[2],
        )
    }

    pub fn text_primary_color(&self) -> Color {
        Color::srgb(
            self.colors.text_primary[0],
            self.colors.text_primary[1],
            self.colors.text_primary[2],
        )
    }

    pub fn text_secondary_color(&self) -> Color {
        Color::srgb(
            self.colors.text_secondary[0],
            self.colors.text_secondary[1],
            self.colors.text_secondary[2],
        )
    }

    pub fn question_box_background(&self) -> Color {
        Color::srgba(0.05, 0.1, 0.15, self.question_box.background_alpha)
    }

    pub fn timer_box_background(&self) -> Color {
        Color::srgba(0.05, 0.1, 0.15, self.timer_box.background_alpha)
    }

    pub fn cards_grid_background(&self) -> Color {
        Color::srgba(0.05, 0.1, 0.15, self.cards_grid.background_alpha)
    }
}
