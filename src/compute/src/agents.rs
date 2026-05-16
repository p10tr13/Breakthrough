mod mcts;
mod metrics;
mod minimax;
mod stats;

pub use mcts::{MctsAgent, MctsStats, MctsStatsAccumulator};
pub use metrics::*;
pub use minimax::{HeuristicEvaluator, MinimaxAgent, MinimaxStats, MinimaxStatsAccumulator};
pub use stats::{AgentRuntimeStats, AgentStatsAccumulator};

use serde::{Deserialize, Serialize};

use crate::core::{Board, BoardConfig, Ply};

fn default_max_depth() -> u8 {
    4
}

fn default_material() -> i32 {
    100
}
fn default_advancement() -> i32 {
    10
}
fn default_defended() -> i32 {
    5
}
fn default_edge_penalty() -> i32 {
    -2
}

fn default_max_iterations() -> u32 {
    75000
}
fn default_max_time_ms() -> Option<u64> {
    None
}
fn default_exploration_constant() -> f64 {
    1.41
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AgentConfig {
    Minimax {
        #[serde(default = "default_max_depth")]
        max_depth: u8,

        #[serde(default = "default_material")]
        material_weight: i32,

        #[serde(default = "default_advancement")]
        advancement_weight: i32,

        #[serde(default = "default_defended")]
        defended_weight: i32,

        #[serde(default = "default_edge_penalty")]
        edge_penalty_weight: i32,
    },
    Mcts {
        #[serde(default = "default_max_iterations")]
        max_iterations: u32,
        #[serde(default = "default_max_time_ms")]
        max_time_ms: Option<u64>,
        #[serde(default = "default_exploration_constant")]
        exploration_constant: f64,
    },
    Human,
}

impl From<&AgentConfig> for AgentType {
    fn from(config: &AgentConfig) -> Self {
        match config {
            AgentConfig::Minimax { .. } => Self::Minimax,
            AgentConfig::Mcts { .. } => Self::Mcts,
            AgentConfig::Human => Self::Human,
        }
    }
}

impl AgentConfig {
    pub fn create_agent(&self, seed: u64) -> BreakthroughAgent {
        match self {
            Self::Minimax {
                max_depth,
                material_weight,
                advancement_weight,
                defended_weight,
                edge_penalty_weight,
            } => {
                let heuristic = HeuristicEvaluator::new(
                    *material_weight,
                    *advancement_weight,
                    *defended_weight,
                    *edge_penalty_weight,
                );
                BreakthroughAgent::Minimax(MinimaxAgent::new(*max_depth, seed, Box::new(heuristic)))
            }
            Self::Mcts {
                max_iterations,
                max_time_ms,
                exploration_constant,
            } => BreakthroughAgent::Mcts(MctsAgent::new(
                *max_iterations,
                *max_time_ms,
                *exploration_constant,
                seed,
            )),
            Self::Human => BreakthroughAgent::Human,
        }
    }

    pub fn is_human(&self) -> bool {
        matches!(self, Self::Human)
    }
}

#[derive(Debug, Default, Clone)]
pub enum AgentStats {
    #[default]
    None,
    Minimax(MinimaxStats),
    Mcts(MctsStats),
}

pub trait Agent: Send {
    fn select_ply(&mut self, board: &Board, config: &BoardConfig) -> Option<Ply>;

    fn take_stats(&mut self) -> AgentStats;
}

#[derive(Debug)]
pub enum BreakthroughAgent {
    Minimax(MinimaxAgent),
    Mcts(MctsAgent),
    Human,
}

impl BreakthroughAgent {
    pub fn select_ply(&mut self, board: &Board, config: &BoardConfig) -> Option<Ply> {
        match self {
            Self::Minimax(agent) => agent.select_ply(board, config),
            Self::Mcts(agent) => agent.select_ply(board, config),
            Self::Human => None,
        }
    }

    pub fn take_stats(&mut self) -> AgentStats {
        match self {
            Self::Minimax(agent) => agent.take_stats(),
            Self::Mcts(agent) => agent.take_stats(),
            Self::Human => AgentStats::None,
        }
    }

    pub fn is_human(&self) -> bool {
        matches!(self, Self::Human)
    }
}
