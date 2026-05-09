use std::{
    sync::mpsc::{self, Receiver},
    thread,
    time::Instant,
};

use crate::{
    agents::{AgentConfig, AgentRuntimeStats, AgentStats, BreakthroughAgent},
    core::{Board, BoardConfig, Player, Ply},
};

use macroquad::prelude::*;

pub struct GameplayController {
    white_agent: BreakthroughAgent,
    black_agent: BreakthroughAgent,
    turn_timer: Instant,
    ply_receiver: Option<Receiver<(BreakthroughAgent, Option<Ply>, AgentStats)>>,

    pub white_stats: AgentRuntimeStats,
    pub black_stats: AgentRuntimeStats,
}

impl GameplayController {
    pub fn new(white_config: &AgentConfig, black_config: &AgentConfig, seed: u64) -> Self {
        Self {
            white_agent: white_config.create_agent(seed),
            black_agent: black_config.create_agent(seed),
            white_stats: AgentRuntimeStats::new(&white_config.into()),
            black_stats: AgentRuntimeStats::new(&black_config.into()),
            turn_timer: Instant::now(),
            ply_receiver: None,
        }
    }

    pub fn is_selecting(&self) -> bool {
        self.ply_receiver.is_some()
    }

    pub fn try_receive_ply(&mut self, current_turn: Player, board_width: u8) -> Option<Ply> {
        if let Some(rx) = &self.ply_receiver
            && let Ok((agent, ply, stats)) = rx.try_recv()
        {
            let time_taken = self.turn_timer.elapsed().as_millis();
            let ply_str = ply
                .map(|p| p.encode(board_width))
                .unwrap_or_else(|| "none".to_string());

            match current_turn {
                Player::White => {
                    self.white_agent = agent;
                    self.white_stats.record_move(ply_str, time_taken, stats);
                }
                Player::Black => {
                    self.black_agent = agent;
                    self.black_stats.record_move(ply_str, time_taken, stats);
                }
            }

            self.ply_receiver = None;
            self.turn_timer = Instant::now();

            return ply;
        }
        None
    }

    pub fn get_opponent_agent_mut(&mut self, current_turn: Player) -> &mut BreakthroughAgent {
        match current_turn {
            Player::White => &mut self.black_agent,
            Player::Black => &mut self.white_agent,
        }
    }

    pub fn trigger_ai_ply(&mut self, board: &Board, config: &BoardConfig) {
        if self.ply_receiver.is_some() {
            return;
        }

        let is_white_turn = board.turn.is_white();

        let is_human = if is_white_turn {
            self.white_agent.is_human()
        } else {
            self.black_agent.is_human()
        };

        if is_human {
            return;
        }

        let mut agent = if is_white_turn {
            std::mem::replace(&mut self.white_agent, BreakthroughAgent::Human)
        } else {
            std::mem::replace(&mut self.black_agent, BreakthroughAgent::Human)
        };

        self.turn_timer = Instant::now();

        let (tx, rx) = mpsc::channel();
        self.ply_receiver = Some(rx);

        let board_copy = *board;
        let config_copy = config.clone();

        thread::spawn(move || {
            let ply = agent.select_ply(&board_copy, &config_copy);
            let stats = agent.take_stats();
            let _ = tx.send((agent, ply, stats));
        });
    }

    pub fn record_human_ply(&mut self, current_turn: Player, ply: Ply, board_width: u8) {
        let time_taken = self.turn_timer.elapsed().as_millis();
        let ply_str = ply.encode(board_width);

        match current_turn {
            Player::White => self
                .white_stats
                .record_move(ply_str, time_taken, AgentStats::None),
            Player::Black => self
                .black_stats
                .record_move(ply_str, time_taken, AgentStats::None),
        }

        self.turn_timer = Instant::now();
    }
}
