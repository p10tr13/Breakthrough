#![allow(dead_code)]

use egui_macroquad::egui;

pub struct MenuButton {
    text: String,
    icon: String,
    size: egui::Vec2,
    text_color: egui::Color32,
}

impl MenuButton {
    pub fn new(text: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: icon.into(),
            size: egui::vec2(250.0, 60.0),
            text_color: egui::Color32::from_rgb(255, 255, 255),
        }
    }

    pub fn size(mut self, size: egui::Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn text_color(mut self, color: egui::Color32) -> Self {
        self.text_color = color;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let button_text = format!("{} {}", self.icon, self.text);

        ui.add_sized(
            self.size,
            egui::Button::new(
                egui::RichText::new(button_text)
                    .size(18.0)
                    .color(self.text_color),
            ),
        )
    }
}
