//! Save dialog UI component.

#![allow(dead_code)]

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum SaveDialogMessage {
    NameChanged(String),
    Save,
    Cancel,
}

pub struct SaveDialog {
    pub name: String,
    pub is_visible: bool,
    pub error: Option<String>,
}

impl SaveDialog {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            is_visible: false,
            error: None,
        }
    }

    pub fn show(&mut self) {
        self.is_visible = true;
        self.error = None;
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        self.name.clear();
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn validate_name(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if self.name.len() > 64 {
            return Err("Name is too long (max 64 characters)".to_string());
        }

        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        for ch in invalid_chars {
            if self.name.contains(ch) {
                return Err(format!("Name cannot contain '{}'", ch));
            }
        }

        Ok(())
    }

    pub fn view<'a, Message>(&'a self) -> Element<'a, Message>
    where
        Message: 'a + Clone + From<SaveDialogMessage>,
    {
        if !self.is_visible {
            return container(text("")).into();
        }

        let title = text("Save Session").size(18);

        let name_input = text_input("Enter session name...", &self.name)
            .on_input(|s| SaveDialogMessage::NameChanged(s).into())
            .padding(8)
            .width(Length::Fill);

        let error_text = if let Some(ref error) = self.error {
            text(error).size(12).color(iced::Color::from_rgb(0.9, 0.2, 0.2))
        } else {
            text("").size(12)
        };

        let buttons = row![
            button(text("Cancel")).on_press(SaveDialogMessage::Cancel.into()),
            button(text("Save")).on_press(SaveDialogMessage::Save.into()),
        ]
        .spacing(8);

        let content = column![title, name_input, error_text, buttons]
            .spacing(12)
            .padding(16)
            .width(Length::Fixed(300.0));

        container(content)
            .style(container::bordered_box)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

impl Default for SaveDialog {
    fn default() -> Self {
        Self::new()
    }
}
