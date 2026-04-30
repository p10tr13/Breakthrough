mod menu_action;
mod menu_button;

use menu_button::MenuButton;

pub use menu_action::MenuAction;

use egui_macroquad::egui::{self, Vec2};

use crate::BreakthroughConfig;

pub struct MenuView<'a> {
    config: &'a BreakthroughConfig,
}

impl<'a> MenuView<'a> {
    pub fn new(config: &'a BreakthroughConfig) -> Self {
        Self { config }
    }

    pub fn show(&self, ui: &mut egui::Ui) -> Option<MenuAction> {
        ui.heading(egui::RichText::new("Breakthrough").size(60.0).strong());
        ui.add_space(40.0);

        ui.vertical_centered(|ui| {
            ui.set_max_width(500.0);
            self.show_launch_settings(ui);
        });

        ui.add_space(50.0);
        self.show_menu_buttons(ui)
    }

    fn show_launch_settings(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} Launch Configuration",
                        egui_phosphor::fill::GEAR
                    ))
                    .size(20.0)
                    .strong(),
                );
                ui.separator();
                ui.add_space(10.0);

                let label_value = |ui: &mut egui::Ui, label: &str, value: String, icon: &str| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("{icon} {label}")));
                        ui.label(egui::RichText::new(value).strong());
                    });
                };

                label_value(
                    ui,
                    "Board Size:",
                    format!("{} x {}", self.config.board_width, self.config.board_height),
                    egui_phosphor::fill::GRID_FOUR,
                );

                let seed_str = self
                    .config
                    .seed
                    .map_or("None (Random)".to_string(), |s| s.to_string());
                label_value(ui, "Random Seed:", seed_str, egui_phosphor::fill::DICE_FIVE);

                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new(format!("{} Players:", egui_phosphor::fill::USERS))
                        .strong(),
                );

                ui.collapsing(
                    format!("{} White Player Details", egui_phosphor::fill::USER),
                    |ui| {
                        ui.label(egui::RichText::new(format!(
                            "{:#?}",
                            self.config.white_player
                        )));
                    },
                );

                ui.collapsing(
                    format!("{} Black Player Details", egui_phosphor::fill::USER),
                    |ui| {
                        ui.label(egui::RichText::new(format!(
                            "{:#?}",
                            self.config.black_player
                        )));
                    },
                );

                ui.add_space(5.0);
            });
        });
    }

    fn show_menu_buttons(&self, ui: &mut egui::Ui) -> Option<MenuAction> {
        if MenuButton::new("Start Game", egui_phosphor::fill::PLAY)
            .size(Vec2::new(200.0, 40.0))
            .show(ui)
            .clicked()
        {
            return Some(MenuAction::GoToGameplay);
        }

        ui.add_space(15.0);

        if MenuButton::new("Exit", egui_phosphor::fill::SIGN_OUT)
            .size(Vec2::new(200.0, 40.0))
            .show(ui)
            .clicked()
        {
            return Some(MenuAction::Exit);
        }

        None
    }
}
