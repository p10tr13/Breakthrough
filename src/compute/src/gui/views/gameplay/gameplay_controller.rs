use std::{
    sync::mpsc::{self, Receiver},
    thread,
};

use crate::{
    agents::{AgentConfig, BreakthroughAgent},
    core::{Board, BoardConfig, Player, Ply},
};

use macroquad::prelude::*;

pub struct GameplayController {
    white_agent: BreakthroughAgent,
    black_agent: BreakthroughAgent,
    ply_receiver: Option<Receiver<(BreakthroughAgent, Option<Ply>)>>,
}

impl GameplayController {
    pub fn new(white_config: &AgentConfig, black_config: &AgentConfig, seed: u64) -> Self {
        Self {
            white_agent: white_config.create_agent(seed),
            black_agent: black_config.create_agent(seed),
            ply_receiver: None,
        }
    }

    pub fn is_selecting(&self) -> bool {
        self.ply_receiver.is_some()
    }

    pub fn try_receive_ply(&mut self, current_turn: Player) -> Option<Ply> {
        if let Some(rx) = &self.ply_receiver
            && let Ok((agent, ply)) = rx.try_recv()
        {
            match current_turn {
                Player::White => self.white_agent = agent,
                Player::Black => self.black_agent = agent,
            }
            self.ply_receiver = None;
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

        let (tx, rx) = mpsc::channel();
        self.ply_receiver = Some(rx);

        let board_copy = *board;
        let config_copy = config.clone();

        thread::spawn(move || {
            let ply = agent.select_ply(&board_copy, &config_copy);
            let _ = tx.send((agent, ply));
        });
    }
}
