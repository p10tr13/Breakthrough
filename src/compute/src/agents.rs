mod minimax;

pub use minimax::MinimaxAgent;

use serde::{Deserialize, Serialize};

use crate::core::{Board, BoardConfig, Ply};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AgentConfig {
    Minimax { max_depth: u8 },
    Human,
}

impl AgentConfig {
    pub fn create_agent(&self, seed: u64) -> BreakthroughAgent {
        match self {
            Self::Minimax { max_depth } => {
                BreakthroughAgent::Minimax(MinimaxAgent::new(*max_depth, seed))
            }
            Self::Human => BreakthroughAgent::Human,
        }
    }

    pub fn is_human(&self) -> bool {
        matches!(self, Self::Human)
    }
}

pub trait Agent: Send {
    fn select_ply(&mut self, board: &Board, config: &BoardConfig) -> Option<Ply>;
}

#[derive(Debug)]
pub enum BreakthroughAgent {
    Minimax(MinimaxAgent),
    Human,
}

impl BreakthroughAgent {
    pub fn select_ply(&mut self, board: &Board, config: &BoardConfig) -> Option<Ply> {
        match self {
            Self::Minimax(agent) => agent.select_ply(board, config),
            Self::Human => None,
        }
    }

    pub fn is_human(&self) -> bool {
        matches!(self, Self::Human)
    }
}
