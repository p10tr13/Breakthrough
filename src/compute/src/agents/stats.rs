use crate::agents::AgentStats;

use super::{AgentType, MinimaxStatsAccumulator};

#[derive(Debug, Clone)]
pub enum AgentStatsAccumulator {
    None,
    Minimax(MinimaxStatsAccumulator),
}

#[derive(Debug, Clone)]
pub struct AgentRuntimeStats {
    pub total_time_ms: u128,
    pub total_moves: usize,
    pub move_history: Vec<String>,
    pub stats_accumulator: AgentStatsAccumulator,
}

impl AgentRuntimeStats {
    pub fn new(agent_type: &AgentType) -> Self {
        Self {
            total_time_ms: 0,
            total_moves: 0,
            move_history: Vec::new(),
            stats_accumulator: match agent_type {
                AgentType::Minimax => {
                    AgentStatsAccumulator::Minimax(MinimaxStatsAccumulator::default())
                }
                AgentType::Human => AgentStatsAccumulator::None,
            },
        }
    }

    pub fn record_move(&mut self, ply_str: String, time_ms: u128, move_stats: AgentStats) {
        self.total_moves += 1;
        self.move_history.push(ply_str);
        self.total_time_ms += time_ms;

        match (&mut self.stats_accumulator, move_stats) {
            (AgentStatsAccumulator::Minimax(acc), AgentStats::Minimax(stats)) => {
                acc.accumulate(&stats);
            }
            (AgentStatsAccumulator::None, AgentStats::None) => {
                // Human agent doesn't provide stats
            }
            _ => unreachable!("Mismatch between agent type and stats type"),
        }
    }
}
