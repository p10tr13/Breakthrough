use std::path::PathBuf;

use egui_macroquad::egui;
use macroquad::prelude::*;
use miette::{IntoDiagnostic, Result};

use compute::{
    BreakthroughConfig,
    cli::build_command,
    gui::{GameplayAction, GameplayView, MenuAction, MenuView},
};

enum AppState {
    Menu(BreakthroughConfig),
    Gameplay(Box<GameplayView>),
}

#[macroquad::main("Breakthrough")]
async fn main() -> Result<()> {
    miette::set_panic_hook();

    let command = build_command();
    let matches = command.get_matches();

    let config_path = matches.get_one::<PathBuf>("config").unwrap();
    let config_content = std::fs::read_to_string(config_path).into_diagnostic()?;
    let mut config: BreakthroughConfig = toml::from_str(&config_content).into_diagnostic()?;

    if let Some(board_width) = matches.get_one::<u8>("board-width") {
        config.board_width = *board_width;
    }

    if let Some(board_height) = matches.get_one::<u8>("board-height") {
        config.board_height = *board_height;
    }
    if let Some(seed) = matches.get_one::<u64>("seed") {
        config.seed = Some(*seed);
    }

    dbg!(&config);

    let mut state = AppState::Menu(config);
    let mut image_loaders_installed = false;

    loop {
        clear_background(Color::new(0.05, 0.05, 0.05, 1.0));

        egui_macroquad::ui(|egui_ctx| {
            if !image_loaders_installed {
                setup_egui(egui_ctx);
                image_loaders_installed = true;
            }

            egui_ctx.set_visuals(egui::Visuals::dark());

            handle_state(egui_ctx, &mut state);
        });

        egui_macroquad::draw();
        next_frame().await;
    }
}

fn setup_egui(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Fill);
    ctx.set_fonts(fonts);
}

fn handle_state(egui_ctx: &egui::Context, state: &mut AppState) {
    let mut next_state = None;

    match state {
        AppState::Menu(config) => {
            let frame = egui::Frame::NONE
                .fill(egui::Color32::from_rgb(15, 15, 15))
                .inner_margin(egui::Margin::same(20));

            egui::CentralPanel::default()
                .frame(frame)
                .show(egui_ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        let menu = MenuView::new(config);
                        if let Some(action) = menu.show(ui) {
                            match action {
                                MenuAction::GoToGameplay => {
                                    let mut gameplay_view = GameplayView::new(config.clone());
                                    gameplay_view.trigger_ai_ply();
                                    next_state = Some(AppState::Gameplay(Box::new(gameplay_view)));
                                }
                                MenuAction::Exit => {
                                    std::process::exit(0);
                                }
                            }
                        }
                    });
                });
        }
        AppState::Gameplay(gameplay_view) => {
            if let Some(action) = gameplay_view.show(egui_ctx) {
                match action {
                    GameplayAction::BackToMenu => {
                        let config = gameplay_view.get_config().clone();
                        next_state = Some(AppState::Menu(config));
                    }
                }
            }
        }
    }

    if let Some(new_state) = next_state {
        *state = new_state;
    }
}
