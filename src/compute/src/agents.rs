mod mcts;
mod metrics;
mod minimax;
mod stats;

use crate::core::{Board, BoardConfig, Ply};
pub use mcts::{MctsAgent, MctsStats, MctsStatsAccumulator, PlayoutStrategy, SelectionStrategy};
pub use metrics::*;
pub use minimax::{
    HeuristicEvaluator, MinimaxAgent, MinimaxStats, MinimaxStatsAccumulator, PositionEvaluator,
};
pub use stats::{AgentRuntimeStats, AgentStatsAccumulator};

use super::defaults::*;

use serde::{Deserialize, Serialize};

pub fn build_mcts_selection_strategy(use_rave: bool, rave_k: f64) -> SelectionStrategy {
    if use_rave {
        SelectionStrategy::rave(rave_k)
    } else {
        SelectionStrategy::ucb1()
    }
}

pub fn build_mcts_playout_strategy(
    use_heavy_playouts: bool,
    heavy_playouts_epsilon: f64,
    material_weight: i32,
    advancement_weight: i32,
    defended_weight: i32,
    edge_penalty_weight: i32,
) -> PlayoutStrategy {
    if use_heavy_playouts {
        let evaluator = Box::new(HeuristicEvaluator::new(
            material_weight,
            advancement_weight,
            defended_weight,
            edge_penalty_weight,
        ));
        PlayoutStrategy::heavy(evaluator, heavy_playouts_epsilon)
    } else {
        PlayoutStrategy::random()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HeavyPlayoutMetricsOptions {
    pub epsilon: Option<f64>,
    pub material_weight: Option<i32>,
    pub advancement_weight: Option<i32>,
    pub defended_weight: Option<i32>,
    pub edge_penalty_weight: Option<i32>,
}

pub fn heavy_playout_metrics_options(
    use_heavy_playouts: bool,
    heavy_playouts_epsilon: f64,
    material_weight: i32,
    advancement_weight: i32,
    defended_weight: i32,
    edge_penalty_weight: i32,
) -> HeavyPlayoutMetricsOptions {
    if use_heavy_playouts {
        HeavyPlayoutMetricsOptions {
            epsilon: Some(heavy_playouts_epsilon),
            material_weight: Some(material_weight),
            advancement_weight: Some(advancement_weight),
            defended_weight: Some(defended_weight),
            edge_penalty_weight: Some(edge_penalty_weight),
        }
    } else {
        HeavyPlayoutMetricsOptions {
            epsilon: None,
            material_weight: None,
            advancement_weight: None,
            defended_weight: None,
            edge_penalty_weight: None,
        }
    }
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

        #[serde(default = "default_use_rave")]
        use_rave: bool,

        #[serde(default = "default_rave_k")]
        rave_k: f64,

        #[serde(default = "default_use_heavy_playouts")]
        use_heavy_playouts: bool,

        #[serde(default = "default_heavy_playouts_epsilon")]
        heavy_playouts_epsilon: f64,

        #[serde(default = "default_material")]
        material_weight: i32,

        #[serde(default = "default_advancement")]
        advancement_weight: i32,

        #[serde(default = "default_defended")]
        defended_weight: i32,

        #[serde(default = "default_edge_penalty")]
        edge_penalty_weight: i32,
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
                use_rave,
                rave_k,
                use_heavy_playouts,
                heavy_playouts_epsilon,
                material_weight,
                advancement_weight,
                defended_weight,
                edge_penalty_weight,
            } => {
                let selection_strategy = build_mcts_selection_strategy(*use_rave, *rave_k);
                let playout_strategy = build_mcts_playout_strategy(
                    *use_heavy_playouts,
                    *heavy_playouts_epsilon,
                    *material_weight,
                    *advancement_weight,
                    *defended_weight,
                    *edge_penalty_weight,
                );

                BreakthroughAgent::Mcts(MctsAgent::new(
                    *max_iterations,
                    *max_time_ms,
                    *exploration_constant,
                    selection_strategy,
                    playout_strategy,
                    seed,
                ))
            }
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
