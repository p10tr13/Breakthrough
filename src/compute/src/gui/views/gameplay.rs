mod gameplay_action;
mod gameplay_controller;

use std::path::PathBuf;

use crate::{
    BreakthroughConfig,
    agents::{
        self, AgentConfig, AgentStatsAccumulator, BreakthroughAgent, CommonMetrics, HumanMetrics,
        MctsAgent, MctsMetrics, MinimaxAgent, MinimaxMetrics, PlayoutStrategy, SelectionStrategy,
        append_record_to_jsonl,
    },
    core::{Board, BoardConfig, Player, Status},
    gui::themes::BoardTheme,
};

use egui_macroquad::egui;
use macroquad::prelude::*;

pub use gameplay_action::GameplayAction;
use gameplay_controller::GameplayController;

struct BoardMetrics {
    pub offset_x: f32,
    pub offset_y: f32,
    pub width_px: f32,
    pub height_px: f32,
    pub square_size: f32,
}

pub struct GameplayView {
    config: BreakthroughConfig,
    board_config: BoardConfig,
    board: Board,
    theme: BoardTheme,
    flip_board: bool,
    white_output: PathBuf,
    black_output: PathBuf,
    pan: egui::Vec2,
    zoom: f32,
    selected_square: Option<u8>,
    pub metrics_saved: bool,
    pub gameplay_controller: GameplayController,
}

impl GameplayView {
    pub fn new(config: BreakthroughConfig, white_output: PathBuf, black_output: PathBuf) -> Self {
        let board_config = BoardConfig::new(config.board_width, config.board_height);
        let board = Board::new(config.board_width, config.board_height);
        let flip_board = !config.white_player.is_human() && config.black_player.is_human();

        let gameplay_controller = GameplayController::new(
            &config.white_player,
            &config.black_player,
            config.seed.unwrap_or(42),
        );

        Self {
            config,
            board_config,
            board,
            flip_board,
            white_output,
            black_output,
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
            selected_square: None,
            theme: BoardTheme::default(),
            metrics_saved: false,
            gameplay_controller,
        }
    }

    pub fn get_config(&self) -> &BreakthroughConfig {
        &self.config
    }

    pub fn trigger_ai_ply(&mut self) {
        self.gameplay_controller
            .trigger_ai_ply(&self.board, &self.board_config);
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<GameplayAction> {
        if let Some(ply) = self
            .gameplay_controller
            .try_receive_ply(self.board.turn, self.board_config.width)
        {
            self.board.apply_ply(ply);
            self.gameplay_controller
                .trigger_ai_ply(&self.board, &self.board_config);
        }

        self.check_and_save_metrics();

        let mut action = self.show_game_over_popup(ctx);

        egui::SidePanel::right("gameplay_side_panel")
            .exact_width(250.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("Player Parameters").strong());
                ui.separator();

                ui.add_space(10.0);

                if !self.gameplay_controller.is_selecting() {
                    let is_white_turn = self.board.turn.is_white();
                    let current_turn = self.board.turn;
                    let opponent_config = if is_white_turn {
                        &mut self.config.black_player
                    } else {
                        &mut self.config.white_player
                    };
                    let opponent_agent = self
                        .gameplay_controller
                        .get_opponent_agent_mut(current_turn);
                    let (current_player_label, opponent_player_label) = if is_white_turn {
                        (Player::White.to_string(), Player::Black.to_string())
                    } else {
                        (Player::Black.to_string(), Player::White.to_string())
                    };

                    ui.label(
                        egui::RichText::new(format!("{current_player_label}'s Turn (Human)"))
                            .strong()
                            .color(egui::Color32::LIGHT_GREEN),
                    );
                    ui.label("Awaiting your move...");
                    ui.add_space(15.0);

                    match opponent_agent {
                        BreakthroughAgent::Minimax(minimax) => {
                            Self::show_minimax_settings(ui, minimax, &opponent_player_label);
                        }
                        BreakthroughAgent::Mcts(mcts) => {
                            Self::show_mcts_settings(
                                ui,
                                mcts,
                                opponent_config,
                                &opponent_player_label,
                            );
                        }
                        BreakthroughAgent::Human => {
                            ui.label(format!("Opponent ({opponent_player_label}) is also Human.",));
                        }
                    }
                } else {
                    Self::show_agent_thinking(ui);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    if ui
                        .add_sized(
                            [200.0, 40.0],
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} Back to Menu",
                                    egui_phosphor::fill::HOUSE
                                ))
                                .strong(),
                            ),
                        )
                        .clicked()
                    {
                        action = Some(GameplayAction::BackToMenu);
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(30, 30, 30)))
            .show(ctx, |ui| {
                self.show_board(ui);
            });

        action
    }

    fn show_minimax_settings(ui: &mut egui::Ui, agent: &mut MinimaxAgent, player_label: &str) {
        ui.label(egui::RichText::new(format!("{player_label} Player (Minimax):")).strong());
        ui.add_space(5.0);
        ui.add(egui::Slider::new(&mut agent.max_depth, 1..=8).text("Search Depth"));
    }

    fn show_mcts_settings(
        ui: &mut egui::Ui,
        agent: &mut MctsAgent,
        config: &mut AgentConfig,
        player_label: &str,
    ) {
        ui.label(egui::RichText::new(format!("{player_label} Player (MCTS):")).strong());
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            if ui
                .radio(agent.max_time_ms.is_none(), "Iterations")
                .clicked()
            {
                agent.max_time_ms = None;
                if let AgentConfig::Mcts { max_time_ms, .. } = config {
                    *max_time_ms = None;
                }
            }
            if ui
                .radio(agent.max_time_ms.is_some(), "Time Limit")
                .clicked()
            {
                agent.max_time_ms = Some(1000);
                if let AgentConfig::Mcts { max_time_ms, .. } = config {
                    *max_time_ms = agent.max_time_ms;
                }
            }
        });

        if let Some(ref mut max_time) = agent.max_time_ms {
            if ui
                .add(
                    egui::Slider::new(max_time, 100..=10000)
                        .text("ms")
                        .logarithmic(true),
                )
                .changed()
                && let AgentConfig::Mcts { max_time_ms, .. } = config
            {
                *max_time_ms = Some(*max_time);
            }
        } else {
            if ui
                .add(
                    egui::Slider::new(&mut agent.max_iterations, 1000..=100000)
                        .text("iters")
                        .logarithmic(true),
                )
                .changed()
                && let AgentConfig::Mcts { max_iterations, .. } = config
            {
                *max_iterations = agent.max_iterations;
            }
        }

        if ui
            .add(egui::Slider::new(&mut agent.exploration_constant, 0.0..=5.0).text("exploration"))
            .changed()
            && let AgentConfig::Mcts {
                exploration_constant,
                ..
            } = config
        {
            *exploration_constant = agent.exploration_constant;
        }

        ui.separator();

        let mut use_rave = matches!(agent.selection_strategy, SelectionStrategy::Rave { .. });
        if ui.checkbox(&mut use_rave, "RAVE").changed() {
            let rave_k = match agent.selection_strategy {
                SelectionStrategy::Rave { k } => k,
                SelectionStrategy::Ucb1 => {
                    if let AgentConfig::Mcts { rave_k, .. } = config {
                        *rave_k
                    } else {
                        agents::DEFAULT_MCTS_RAVE_K
                    }
                }
            };
            agent.selection_strategy = agents::build_mcts_selection_strategy(use_rave, rave_k);
            if let AgentConfig::Mcts {
                use_rave: config_use_rave,
                rave_k: config_rave_k,
                ..
            } = config
            {
                *config_use_rave = use_rave;
                *config_rave_k = rave_k;
            }
        }

        if let SelectionStrategy::Rave { k } = &mut agent.selection_strategy
            && ui
                .add(
                    egui::Slider::new(k, 10.0..=10000.0)
                        .text("RAVE k")
                        .logarithmic(true),
                )
                .changed()
            && let AgentConfig::Mcts { rave_k, .. } = config
        {
            *rave_k = *k;
        }

        let mut use_heavy_playouts =
            matches!(agent.playout_strategy, PlayoutStrategy::Heavy { .. });
        if ui
            .checkbox(&mut use_heavy_playouts, "Heavy playouts")
            .changed()
        {
            let (epsilon, material, advancement, defended, edge_penalty) =
                if let AgentConfig::Mcts {
                    heavy_playouts_epsilon,
                    material_weight,
                    advancement_weight,
                    defended_weight,
                    edge_penalty_weight,
                    ..
                } = config
                {
                    (
                        *heavy_playouts_epsilon,
                        *material_weight,
                        *advancement_weight,
                        *defended_weight,
                        *edge_penalty_weight,
                    )
                } else {
                    (
                        agents::DEFAULT_MCTS_HEAVY_PLAYOUTS_EPSILON,
                        agents::DEFAULT_HEURISTIC_MATERIAL_WEIGHT,
                        agents::DEFAULT_HEURISTIC_ADVANCEMENT_WEIGHT,
                        agents::DEFAULT_HEURISTIC_DEFENDED_WEIGHT,
                        agents::DEFAULT_HEURISTIC_EDGE_PENALTY_WEIGHT,
                    )
                };

            agent.playout_strategy = agents::build_mcts_playout_strategy(
                use_heavy_playouts,
                epsilon,
                material,
                advancement,
                defended,
                edge_penalty,
            );

            if let AgentConfig::Mcts {
                use_heavy_playouts: config_use_heavy_playouts,
                ..
            } = config
            {
                *config_use_heavy_playouts = use_heavy_playouts;
            }
        }

        if let PlayoutStrategy::Heavy { epsilon, .. } = &mut agent.playout_strategy
            && ui
                .add(egui::Slider::new(epsilon, 0.0..=1.0).text("epsilon"))
                .changed()
            && let AgentConfig::Mcts {
                heavy_playouts_epsilon,
                ..
            } = config
        {
            *heavy_playouts_epsilon = *epsilon;
        }
    }

    fn show_agent_thinking(ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(
                egui::RichText::new("Agent is thinking...")
                    .strong()
                    .color(egui::Color32::GOLD),
            );
        });
    }

    pub fn show_board(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        self.update_camera(ui, rect, &response);

        let cols = self.board_config.width as usize;
        let rows = self.board_config.height as usize;

        let base_square_size = (rect.width() / cols as f32).min(rect.height() / rows as f32);
        let square_size = base_square_size * self.zoom;

        let board_width_px = square_size * cols as f32;
        let board_height_px = square_size * rows as f32;

        let center_x = rect.center().x + self.pan.x;
        let center_y = rect.center().y + self.pan.y;

        let offset_x = center_x - board_width_px / 2.0;
        let offset_y = center_y - board_height_px / 2.0;

        let metrics = BoardMetrics {
            offset_x,
            offset_y,
            width_px: board_width_px,
            height_px: board_height_px,
            square_size,
        };

        self.handle_board_interaction(ui, &response, metrics);

        self.render_board(&painter, rect, offset_x, offset_y, square_size);
    }

    fn is_current_player_human(&self) -> bool {
        match self.board.turn {
            Player::White => self.config.white_player.is_human(),
            Player::Black => self.config.black_player.is_human(),
        }
    }

    fn show_game_over_popup(&mut self, ctx: &egui::Context) -> Option<GameplayAction> {
        let current_status = self.board.get_status(&self.board_config);
        let is_game_over = !current_status.is_ongoing();
        if !is_game_over {
            return None;
        }

        let (window_title, winner_text) = match current_status {
            Status::WhiteWon => (
                format!("{} White Victory!", egui_phosphor::fill::TROPHY),
                "White Player has won the game!",
            ),
            Status::BlackWon => (
                format!("{} Black Victory!", egui_phosphor::fill::TROPHY),
                "Black Player has won the game!",
            ),
            _ => ("Game Over".to_string(), ""),
        };

        let mut action = None;

        egui::Window::new(window_title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.heading(
                        egui::RichText::new(winner_text)
                            .strong()
                            .color(egui::Color32::GOLD),
                    );
                    ui.add_space(30.0);

                    if ui
                        .add_sized(
                            [200.0, 40.0],
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} Back to Menu",
                                    egui_phosphor::fill::HOUSE
                                ))
                                .size(18.0)
                                .strong(),
                            ),
                        )
                        .clicked()
                    {
                        action = Some(GameplayAction::BackToMenu);
                    }
                    ui.add_space(10.0);
                });
            });

        action
    }

    fn render_board(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        offset_x: f32,
        offset_y: f32,
        square_size: f32,
    ) {
        let cols = self.board_config.width as usize;
        let rows = self.board_config.height as usize;

        let mut highlighted_squares_mask = 0_u128;

        if let Some(selected_index) = self.selected_square {
            let all_legal_plies = self.board.get_legal_plies(&self.board_config);
            for ply in all_legal_plies {
                if ply.from == selected_index {
                    highlighted_squares_mask |= 1_u128 << ply.to;
                }
            }
        }

        for visual_row in 0..rows {
            for visual_col in 0..cols {
                let min_p = egui::pos2(
                    offset_x + visual_col as f32 * square_size,
                    offset_y + visual_row as f32 * square_size,
                );
                let max_p = egui::pos2(min_p.x + square_size, min_p.y + square_size);
                let square_rect = egui::Rect::from_min_max(min_p, max_p);

                if !rect.intersects(square_rect) {
                    continue;
                }

                let is_dark_square = (visual_row + visual_col) % 2 == 1;
                let square_color = if is_dark_square {
                    self.theme.dark_square
                } else {
                    self.theme.light_square
                };

                painter.rect_filled(square_rect, 0.0, square_color);

                let (logical_row, logical_col) = if self.flip_board {
                    (visual_row, cols - 1 - visual_col)
                } else {
                    (rows - 1 - visual_row, visual_col)
                };

                let bit_index = (logical_row * cols + logical_col) as u8;
                let mask = 1_u128 << bit_index;
                let center = square_rect.center();

                if (highlighted_squares_mask & mask) != 0 {
                    let dot_radius = square_size * 0.2;
                    painter.circle_filled(center, dot_radius, self.theme.highlight_dot);
                }

                if Some(bit_index) == self.selected_square {
                    painter.rect_filled(square_rect, 0.0, self.theme.highlight_selected);
                }

                let radius = square_size * 0.4;

                if (self.board.white_board & mask) != 0 {
                    painter.circle_filled(center, radius, self.theme.white_piece);
                    painter.circle_stroke(
                        center,
                        radius,
                        egui::Stroke::new(2.0, self.theme.white_piece_stroke),
                    );
                } else if (self.board.black_board & mask) != 0 {
                    painter.circle_filled(center, radius, self.theme.black_piece);
                    painter.circle_stroke(
                        center,
                        radius,
                        egui::Stroke::new(2.0, self.theme.black_piece_stroke),
                    );
                }
            }
        }
    }

    fn handle_board_interaction(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        metrics: BoardMetrics,
    ) {
        if !self.is_current_player_human() {
            self.selected_square = None;
            return;
        }

        if !response.clicked_by(egui::PointerButton::Primary) {
            return;
        }

        let input_state = ui.input(|i| i.pointer.interact_pos());

        if input_state.is_none() {
            self.selected_square = None;
            return;
        }

        let mouse_pos = input_state.unwrap();

        let dx = mouse_pos.x - metrics.offset_x;
        let dy = mouse_pos.y - metrics.offset_y;

        if dx >= 0.0 && dx <= metrics.width_px && dy >= 0.0 && dy <= metrics.height_px {
            let visual_col = (dx / metrics.square_size).floor() as usize;
            let visual_row = (dy / metrics.square_size).floor() as usize;
            let cols = self.board_config.width as usize;
            let rows = self.board_config.height as usize;

            let (logical_row, logical_col) = if self.flip_board {
                (visual_row, cols - 1 - visual_col)
            } else {
                (rows - 1 - visual_row, visual_col)
            };

            let clicked_bit = (logical_row * cols + logical_col) as u8;

            if let Some(from_bit) = self.selected_square {
                let legal_plies = self.board.get_legal_plies(&self.board_config);

                let valid_move = legal_plies
                    .into_iter()
                    .find(|ply| ply.from == from_bit && ply.to == clicked_bit);

                if let Some(ply) = valid_move {
                    self.gameplay_controller.record_human_ply(
                        self.board.turn,
                        ply,
                        self.board_config.width,
                    );
                    self.board.apply_ply(ply);
                    self.selected_square = None;
                    self.gameplay_controller
                        .trigger_ai_ply(&self.board, &self.board_config);
                    return;
                }
            }

            let mask = 1_u128 << clicked_bit;
            let is_own_piece = if self.board.turn == crate::core::Player::White {
                (self.board.white_board & mask) != 0
            } else {
                (self.board.black_board & mask) != 0
            };

            if is_own_piece {
                self.selected_square = Some(clicked_bit);
            } else {
                self.selected_square = None;
            }
        }
    }

    fn update_camera(&mut self, ui: &egui::Ui, rect: egui::Rect, response: &egui::Response) {
        if response.dragged_by(egui::PointerButton::Secondary) {
            self.pan += response.drag_delta();
        }

        if response.clicked_by(egui::PointerButton::Middle) {
            self.pan = egui::Vec2::ZERO;
            self.zoom = 1.0;
        }

        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);

            if scroll_delta != 0.0 {
                self.update_zoom(scroll_delta, rect, response);
            }
        }
    }

    fn update_zoom(&mut self, scroll_delta: f32, rect: egui::Rect, response: &egui::Response) {
        let zoom_factor = (scroll_delta * 0.002).exp();
        let new_zoom = (self.zoom * zoom_factor).clamp(0.2, 5.0);
        let actual_zoom_factor = new_zoom / self.zoom;

        if actual_zoom_factor != 1.0 {
            if let Some(mouse_pos) = response.hover_pos() {
                let center = rect.center().to_vec2();
                let mouse_from_center = mouse_pos.to_vec2() - center;
                self.pan = mouse_from_center - (mouse_from_center - self.pan) * actual_zoom_factor;
            }
            self.zoom = new_zoom;
        }
    }

    fn check_and_save_metrics(&mut self) {
        let status = self.board.get_status(&self.board_config);

        if status.is_ongoing() || self.metrics_saved {
            return;
        }

        let is_white_won = status.is_white_won();
        let white_pieces = self.board.get_piece_count(Player::White) as u8;
        let black_pieces = self.board.get_piece_count(Player::Black) as u8;

        let white_common = CommonMetrics {
            board_width: self.board_config.width,
            board_height: self.board_config.height,
            seed: self.config.seed,
            agent_type: (&self.config.white_player).into(),
            agent_color: Player::White,
            agent_won: is_white_won,
            pieces_remaining: white_pieces,
            move_times_ms: self.gameplay_controller.white_stats.move_times_ms.clone(),
            total_moves: self.gameplay_controller.white_stats.total_moves,
            moves: self.gameplay_controller.white_stats.move_history.clone(),
            opponent_type: (&self.config.black_player).into(),
            opponent_pieces_remaining: black_pieces,
        };

        match &self.gameplay_controller.white_stats.stats_accumulator {
            AgentStatsAccumulator::Minimax(acc) => {
                let AgentConfig::Minimax {
                    max_depth,
                    material_weight,
                    advancement_weight,
                    defended_weight,
                    edge_penalty_weight,
                } = self.config.white_player
                else {
                    unreachable!();
                };
                let metrics = MinimaxMetrics {
                    common: white_common,
                    max_depth,
                    total_nodes_evaluated: acc.total_nodes,
                    total_cutoffs: acc.total_cutoffs,
                    material_weight,
                    advancement_weight,
                    defended_weight,
                    edge_penalty_weight,
                };
                let _ = append_record_to_jsonl(&metrics, self.white_output.clone());
            }
            AgentStatsAccumulator::Mcts(acc) => {
                let AgentConfig::Mcts {
                    max_iterations,
                    max_time_ms,
                    exploration_constant,
                    use_rave,
                    rave_k,
                    use_heavy_playouts,
                    heavy_playouts_epsilon,
                    material_weight,
                    advancement_weight,
                    defended_weight,
                    edge_penalty_weight,
                } = self.config.white_player
                else {
                    unreachable!();
                };
                let heavy_options = agents::heavy_playout_metrics_options(
                    use_heavy_playouts,
                    heavy_playouts_epsilon,
                    material_weight,
                    advancement_weight,
                    defended_weight,
                    edge_penalty_weight,
                );
                let metrics = MctsMetrics {
                    common: white_common,
                    max_iterations,
                    max_time_ms,
                    exploration_constant,
                    use_rave,
                    use_heavy_playouts,

                    rave_k: if use_rave { Some(rave_k) } else { None },

                    heavy_playouts_epsilon: heavy_options.epsilon,
                    material_weight: heavy_options.material_weight,
                    advancement_weight: heavy_options.advancement_weight,
                    defended_weight: heavy_options.defended_weight,
                    edge_penalty_weight: heavy_options.edge_penalty_weight,
                    total_iterations: acc.total_iterations,
                    total_nodes_created: acc.total_nodes_created,
                };
                let _ = append_record_to_jsonl(&metrics, self.white_output.clone());
            }
            AgentStatsAccumulator::None => {
                let metrics = HumanMetrics {
                    common: white_common,
                };
                let _ = append_record_to_jsonl(&metrics, self.white_output.clone());
            }
        }

        let black_common = CommonMetrics {
            board_width: self.board_config.width,
            board_height: self.board_config.height,
            seed: self.config.seed,
            agent_type: (&self.config.black_player).into(),
            agent_color: Player::Black,
            agent_won: !is_white_won,
            pieces_remaining: black_pieces,
            move_times_ms: self.gameplay_controller.black_stats.move_times_ms.clone(),
            total_moves: self.gameplay_controller.black_stats.total_moves,
            moves: self.gameplay_controller.black_stats.move_history.clone(),
            opponent_type: (&self.config.white_player).into(),
            opponent_pieces_remaining: white_pieces,
        };

        match &self.gameplay_controller.black_stats.stats_accumulator {
            AgentStatsAccumulator::Minimax(acc) => {
                let AgentConfig::Minimax {
                    max_depth,
                    material_weight,
                    advancement_weight,
                    defended_weight,
                    edge_penalty_weight,
                } = self.config.black_player
                else {
                    unreachable!();
                };
                let metrics = MinimaxMetrics {
                    common: black_common,
                    max_depth,
                    total_nodes_evaluated: acc.total_nodes,
                    total_cutoffs: acc.total_cutoffs,
                    material_weight,
                    advancement_weight,
                    defended_weight,
                    edge_penalty_weight,
                };
                let _ = append_record_to_jsonl(&metrics, self.black_output.clone());
            }
            AgentStatsAccumulator::Mcts(acc) => {
                let AgentConfig::Mcts {
                    max_iterations,
                    max_time_ms,
                    exploration_constant,
                    use_rave,
                    rave_k,
                    use_heavy_playouts,
                    heavy_playouts_epsilon,
                    material_weight,
                    advancement_weight,
                    defended_weight,
                    edge_penalty_weight,
                } = self.config.black_player
                else {
                    unreachable!();
                };
                let heavy_options = agents::heavy_playout_metrics_options(
                    use_heavy_playouts,
                    heavy_playouts_epsilon,
                    material_weight,
                    advancement_weight,
                    defended_weight,
                    edge_penalty_weight,
                );
                let metrics = MctsMetrics {
                    common: black_common,
                    max_iterations,
                    max_time_ms,
                    exploration_constant,
                    use_rave,
                    use_heavy_playouts,

                    rave_k: if use_rave { Some(rave_k) } else { None },

                    heavy_playouts_epsilon: heavy_options.epsilon,
                    material_weight: heavy_options.material_weight,
                    advancement_weight: heavy_options.advancement_weight,
                    defended_weight: heavy_options.defended_weight,
                    edge_penalty_weight: heavy_options.edge_penalty_weight,
                    total_iterations: acc.total_iterations,
                    total_nodes_created: acc.total_nodes_created,
                };
                let _ = append_record_to_jsonl(&metrics, self.black_output.clone());
            }
            AgentStatsAccumulator::None => {
                let metrics = HumanMetrics {
                    common: black_common,
                };
                let _ = append_record_to_jsonl(&metrics, self.black_output.clone());
            }
        }

        self.metrics_saved = true;
    }
}
