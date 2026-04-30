use egui_macroquad::egui;

#[derive(Clone, Copy)]
pub struct BoardTheme {
    pub light_square: egui::Color32,
    pub dark_square: egui::Color32,
    pub white_piece: egui::Color32,
    pub black_piece: egui::Color32,
    pub highlight_selected: egui::Color32,
    pub highlight_dot: egui::Color32,
    pub white_piece_stroke: egui::Color32,
    pub black_piece_stroke: egui::Color32,
}

impl Default for BoardTheme {
    fn default() -> Self {
        Self {
            light_square: egui::Color32::from_rgb(238, 238, 210),
            dark_square: egui::Color32::from_rgb(118, 150, 86),
            white_piece: egui::Color32::from_rgb(250, 250, 250),
            black_piece: egui::Color32::from_rgb(30, 30, 30),
            highlight_selected: egui::Color32::from_rgba_premultiplied(255, 255, 0, 70),
            highlight_dot: egui::Color32::from_black_alpha(100),
            white_piece_stroke: egui::Color32::BLACK,
            black_piece_stroke: egui::Color32::WHITE,
        }
    }
}
