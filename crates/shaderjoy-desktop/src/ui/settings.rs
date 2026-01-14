//! Settings panel UI component.

#![allow(dead_code)]

use iced::widget::{column, container, row, text};
use iced::{Element, Length};

pub struct SettingsPanel {
    pub provider_name: String,
    pub model_name: String,
    pub grid_size: (u32, u32),
}

impl SettingsPanel {
    pub fn new(provider_name: String, model_name: String) -> Self {
        Self {
            provider_name,
            model_name,
            grid_size: (3, 3),
        }
    }

    pub fn view<'a, Message>(&'a self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let provider_row = row![
            text("Provider:").size(14),
            text(&self.provider_name).size(14),
        ]
        .spacing(8);

        let model_row = row![
            text("Model:").size(14),
            text(&self.model_name).size(14),
        ]
        .spacing(8);

        let grid_row = row![
            text("Grid:").size(14),
            text(format!("{}x{}", self.grid_size.0, self.grid_size.1)).size(14),
        ]
        .spacing(8);

        let content = column![provider_row, model_row, grid_row].spacing(8);

        container(content)
            .padding(12)
            .width(Length::Fill)
            .into()
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new("Ollama".to_string(), "gemma:2b".to_string())
    }
}
