#![allow(dead_code)]

mod metrics;
pub mod node;
mod stats;

use node::MctsNode;
pub use stats::{MctsStats, MctsStatsAccumulator};

use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng, seq::SliceRandom};
use std::time::Instant;

use super::{Agent, AgentStats};
use crate::core::{Board, BoardConfig, Ply, Status};

#[derive(Debug)]
pub struct MctsAgent {
    pub max_iterations: u32,
    pub max_time_ms: Option<u64>,
    pub exploration_constant: f64,
    pub rng: SmallRng,
    pub current_stats: MctsStats,
}

impl MctsAgent {
    pub fn new(
        max_iterations: u32,
        max_time_ms: Option<u64>,
        exploration_constant: f64,
        seed: u64,
    ) -> Self {
        Self {
            max_iterations,
            max_time_ms,
            exploration_constant,
            rng: SmallRng::seed_from_u64(seed),
            current_stats: MctsStats::default(),
        }
    }

    fn check_limits(&self, iterations: u32, start_time: Instant) -> bool {
        match self.max_time_ms {
            Some(max_time) => {
                if (iterations & 511) == 0 {
                    (start_time.elapsed().as_millis() as u64) < max_time
                } else {
                    true
                }
            }
            None => iterations < self.max_iterations,
        }
    }
}

impl Agent for MctsAgent {
    fn select_ply(&mut self, board: &Board, config: &BoardConfig) -> Option<Ply> {
        self.current_stats = MctsStats::default();
        let start_time = Instant::now();

        let use_time = self.max_time_ms.is_some();
        let capacity = if use_time {
            100_000
        } else {
            self.max_iterations as usize + 1
        };

        let mut tree: Vec<MctsNode> = Vec::with_capacity(capacity);

        let mut root_plies = board.get_legal_plies(config);
        if root_plies.is_empty() {
            return None;
        }

        root_plies.shuffle(&mut self.rng);

        tree.push(MctsNode::new(None, None, root_plies, board.turn));
        self.current_stats.nodes_created += 1;

        while self.check_limits(self.current_stats.iterations, start_time) {
            let mut current_idx = 0;
            let mut sim_board = *board;

            // Phase 1: Selection
            while tree[current_idx].unexpanded_plies.is_empty()
                && !tree[current_idx].children.is_empty()
            {
                let parent_visits = tree[current_idx].visits;
                let c = self.exploration_constant;
                let mut best_child_idx = 0;
                let mut best_score = f64::NEG_INFINITY;

                for &child_idx in &tree[current_idx].children {
                    let child = &tree[child_idx];
                    let score = if child.visits == 0 {
                        f64::INFINITY
                    } else {
                        let exploitation = child.wins / (child.visits as f64);
                        let exploration =
                            c * ((parent_visits as f64).ln() / (child.visits as f64)).sqrt();
                        exploitation + exploration
                    };

                    if score == f64::INFINITY {
                        best_child_idx = child_idx;
                        break;
                    }

                    if score > best_score {
                        best_score = score;
                        best_child_idx = child_idx;
                    }
                }

                current_idx = best_child_idx;
                if let Some(ply) = tree[current_idx].ply_to_reach {
                    sim_board.apply_ply(ply);
                }
            }

            // Phase 2: Expansion
            let mut status = sim_board.get_status(config);

            if status == Status::Ongoing && !tree[current_idx].unexpanded_plies.is_empty() {
                let ply_to_expand = tree[current_idx].unexpanded_plies.pop().unwrap();
                sim_board.apply_ply(ply_to_expand);

                let mut next_plies = sim_board.get_legal_plies(config);
                next_plies.shuffle(&mut self.rng);
                let new_node = MctsNode::new(
                    Some(current_idx),
                    Some(ply_to_expand),
                    next_plies,
                    sim_board.turn,
                );

                let new_child_idx = tree.len();
                tree.push(new_node);
                tree[current_idx].children.push(new_child_idx);

                current_idx = new_child_idx;
                self.current_stats.nodes_created += 1;
                status = sim_board.get_status(config);
            }

            // Phase 3: Playout
            let mut playout_steps = 0;
            while status == Status::Ongoing {
                let plies = sim_board.get_legal_plies(config);
                if plies.is_empty() {
                    // Should not happen, but just in case
                    status = if sim_board.turn.is_white() {
                        Status::BlackWon
                    } else {
                        Status::WhiteWon
                    };
                    break;
                }
                let random_idx = self.rng.random_range(0..plies.len());
                sim_board.apply_ply(plies[random_idx]);
                status = sim_board.get_status(config);
                playout_steps += 1;
            }
            self.current_stats.playout_steps += playout_steps;

            // Phase 4: Backpropagation
            let mut node_ptr = Some(current_idx);

            while let Some(idx) = node_ptr {
                let node = &mut tree[idx];
                node.visits += 1;

                let player_who_just_moved = node.turn.opponent();

                let is_win = match status {
                    Status::WhiteWon => player_who_just_moved.is_white(),
                    Status::BlackWon => !player_who_just_moved.is_white(),
                    Status::Ongoing => false,
                };

                if is_win {
                    node.wins += 1.0;
                }

                node_ptr = node.parent;
            }

            self.current_stats.iterations += 1;
        }

        let root = &tree[0];
        let mut best_ply = None;
        let mut max_visits = 0;

        for &child_idx in &root.children {
            let child = &tree[child_idx];
            if child.visits > max_visits {
                max_visits = child.visits;
                best_ply = child.ply_to_reach;
            }
        }

        best_ply
    }

    fn take_stats(&mut self) -> AgentStats {
        let stats = std::mem::take(&mut self.current_stats);
        AgentStats::Mcts(stats)
    }
}
