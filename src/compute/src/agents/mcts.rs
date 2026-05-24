#![allow(dead_code)]

mod metrics;
pub mod node;
mod playout_strategy;
mod rave;
mod selection_strategy;
mod stats;

use node::MctsNode;
pub use playout_strategy::PlayoutStrategy;
use rave::RaveTracker;
pub use selection_strategy::SelectionStrategy;
pub use stats::{MctsStats, MctsStatsAccumulator};

use rand::rngs::SmallRng;
use rand::{SeedableRng, seq::SliceRandom};
use std::time::Instant;

use super::{Agent, AgentStats};
use crate::core::{Board, BoardConfig, Ply, Status};

#[derive(Debug)]
pub struct MctsAgent {
    pub max_iterations: u32,
    pub max_time_ms: Option<u64>,
    pub exploration_constant: f64,

    pub selection_strategy: SelectionStrategy,
    pub playout_strategy: PlayoutStrategy,

    pub rng: SmallRng,
    pub current_stats: MctsStats,

    // Configuration constants
    check_interval: u32,
    tree_capacity_time_mode: usize,
}

impl MctsAgent {
    pub fn new(
        max_iterations: u32,
        max_time_ms: Option<u64>,
        exploration_constant: f64,
        selection_strategy: SelectionStrategy,
        playout_strategy: PlayoutStrategy,
        seed: u64,
    ) -> Self {
        Self {
            max_iterations,
            max_time_ms,
            exploration_constant,
            selection_strategy,
            playout_strategy,
            rng: SmallRng::seed_from_u64(seed),
            current_stats: MctsStats::default(),
            check_interval: 511,
            tree_capacity_time_mode: 100_000,
        }
    }

    fn check_limits(&self, iterations: u32, start_time: Instant) -> bool {
        match self.max_time_ms {
            Some(max_time) => {
                if (iterations & self.check_interval) == 0 {
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
            self.tree_capacity_time_mode
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

        // Main MCTS loop
        while self.check_limits(self.current_stats.iterations, start_time) {
            let mut current_idx = 0;
            let mut sim_board = *board;

            // Phase 1: Selection
            current_idx = self.select_node(&tree, &mut sim_board, current_idx);

            // Phase 2: Expansion
            let (leaf_idx, status) =
                self.expand_node(&mut tree, current_idx, &mut sim_board, config);

            if status != Status::Ongoing {
                let mut rave_tracker = self.new_rave_tracker();
                self.backpropagate(&mut tree, leaf_idx, status, &mut rave_tracker);
                self.current_stats.iterations += 1;
                continue;
            }

            // Phase 3 & 4: Playout and Backpropagation
            let (_final_status, playout_steps) =
                self.do_playout_and_backprop(&mut tree, leaf_idx, &mut sim_board, config);

            self.current_stats.playout_steps += playout_steps as u64;
            self.current_stats.iterations += 1;
        }

        // Select best move from root
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

impl MctsAgent {
    /// Phase 1: Selection
    fn select_node(
        &self,
        tree: &[MctsNode],
        sim_board: &mut Board,
        mut current_idx: usize,
    ) -> usize {
        while tree[current_idx].unexpanded_plies.is_empty()
            && !tree[current_idx].children.is_empty()
        {
            let mut best_child_idx = 0;
            let mut best_score = f64::NEG_INFINITY;

            for &child_idx in &tree[current_idx].children {
                let child = &tree[child_idx];

                if child.visits == 0 {
                    best_child_idx = child_idx;
                    break;
                }

                let score = self.selection_strategy.compute_uct_score(
                    child.visits,
                    child.wins,
                    child.amaf.visits,
                    child.amaf.wins,
                    tree[current_idx].visits,
                    self.exploration_constant,
                );

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

        current_idx
    }

    /// Phase 2: Expansion
    fn expand_node(
        &mut self,
        tree: &mut Vec<MctsNode>,
        current_idx: usize,
        sim_board: &mut Board,
        config: &BoardConfig,
    ) -> (usize, Status) {
        let mut status = sim_board.get_status(config);
        let mut leaf_idx = current_idx;

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

            self.current_stats.nodes_created += 1;
            leaf_idx = new_child_idx;
            status = sim_board.get_status(config);
        }

        (leaf_idx, status)
    }

    /// Combined phase 3 (Playout) and 4 (Backpropagation)
    fn do_playout_and_backprop(
        &mut self,
        tree: &mut [MctsNode],
        current_idx: usize,
        sim_board: &mut Board,
        config: &BoardConfig,
    ) -> (Status, u32) {
        // Phase 3: Playout - Play out a random/heuristic game
        let mut playout_steps = 0;
        let mut status = sim_board.get_status(config);

        let mut rave_tracker = self.new_rave_tracker();

        while status == Status::Ongoing {
            let plies = sim_board.get_legal_plies(config);
            if plies.is_empty() {
                status = if sim_board.turn.is_white() {
                    Status::BlackWon
                } else {
                    Status::WhiteWon
                };
                break;
            }

            let chosen_ply =
                self.playout_strategy
                    .select_move(&plies, sim_board, config, &mut self.rng);

            rave_tracker.record_playout_move(sim_board.turn, chosen_ply);

            sim_board.apply_ply(chosen_ply);
            status = sim_board.get_status(config);
            playout_steps += 1;
        }

        // Phase 4: Backpropagation - Update tree statistics
        self.backpropagate(tree, current_idx, status, &mut rave_tracker);

        (status, playout_steps)
    }

    /// Phase 4: Backpropagation
    fn backpropagate(
        &mut self,
        tree: &mut [MctsNode],
        mut leaf_idx: usize,
        status: Status,
        rave_tracker: &mut RaveTracker,
    ) {
        while leaf_idx != usize::MAX {
            let node_turn = tree[leaf_idx].turn;
            let parent_idx = tree[leaf_idx].parent;
            let ply_to_reach = tree[leaf_idx].ply_to_reach;

            tree[leaf_idx].visits += 1;

            let player_who_just_moved = node_turn.opponent();
            if status.is_won_by(player_who_just_moved) {
                tree[leaf_idx].wins += 1.0;
            }

            rave_tracker.update_amaf_children(tree, leaf_idx, status);

            if let Some(ply) = ply_to_reach {
                rave_tracker.record_path_move(node_turn.opponent(), ply);
            }

            leaf_idx = parent_idx.unwrap_or(usize::MAX);
        }
    }

    fn new_rave_tracker(&self) -> RaveTracker {
        RaveTracker::new(self.selection_strategy.uses_amaf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Player;

    fn test_agent(selection_strategy: SelectionStrategy) -> MctsAgent {
        MctsAgent::new(
            1,
            None,
            1.41,
            selection_strategy,
            PlayoutStrategy::Random,
            0,
        )
    }

    #[test]
    fn status_winner_check_matches_terminal_status() {
        assert!(Status::WhiteWon.is_won_by(Player::White));
        assert!(!Status::WhiteWon.is_won_by(Player::Black));
        assert!(Status::BlackWon.is_won_by(Player::Black));
        assert!(!Status::BlackWon.is_won_by(Player::White));
        assert!(!Status::Ongoing.is_won_by(Player::White));
        assert!(!Status::Ongoing.is_won_by(Player::Black));
    }

    #[test]
    fn select_node_stops_at_node_with_unexpanded_plies() {
        let mut board = Board {
            white_board: 1 << 3,
            black_board: 1 << 8,
            turn: Player::White,
        };
        let original_board = board;
        let mut root = MctsNode::new(None, None, vec![Ply { from: 3, to: 6 }], board.turn);
        root.visits = 10;
        root.children = vec![1];
        let child = MctsNode::new(
            Some(0),
            Some(Ply { from: 3, to: 7 }),
            Vec::new(),
            Player::Black,
        );
        let tree = vec![root, child];
        let agent = test_agent(SelectionStrategy::Ucb1);

        let selected_idx = agent.select_node(&tree, &mut board, 0);

        assert_eq!(selected_idx, 0);
        assert_eq!(board, original_board);
    }

    #[test]
    fn select_node_descends_to_highest_scoring_child_and_applies_ply() {
        let mut board = Board {
            white_board: 1 << 3,
            black_board: 1 << 8,
            turn: Player::White,
        };
        let mut root = MctsNode::new(None, None, Vec::new(), board.turn);
        root.visits = 10;
        root.children = vec![1, 2];

        let mut good_child = MctsNode::new(
            Some(0),
            Some(Ply { from: 3, to: 6 }),
            Vec::new(),
            Player::Black,
        );
        good_child.visits = 5;
        good_child.wins = 5.0;

        let mut bad_child = MctsNode::new(
            Some(0),
            Some(Ply { from: 3, to: 7 }),
            Vec::new(),
            Player::Black,
        );
        bad_child.visits = 5;
        bad_child.wins = 0.0;

        let tree = vec![root, good_child, bad_child];
        let agent = test_agent(SelectionStrategy::Ucb1);

        let selected_idx = agent.select_node(&tree, &mut board, 0);

        assert_eq!(selected_idx, 1);
        assert_eq!(board.turn, Player::Black);
        assert_eq!(board.white_board, 1 << 6);
    }

    #[test]
    fn expansion_creates_child_for_nonterminal_move() {
        let config = BoardConfig::new(3, 3);
        let mut board = Board {
            white_board: 1 << 1,
            black_board: 1 << 7,
            turn: Player::White,
        };
        let ply = Ply { from: 1, to: 4 };
        let mut agent = test_agent(SelectionStrategy::Ucb1);
        let mut tree = vec![MctsNode::new(None, None, vec![ply], board.turn)];

        let (leaf_idx, status) = agent.expand_node(&mut tree, 0, &mut board, &config);

        assert_eq!(leaf_idx, 1);
        assert_eq!(status, Status::Ongoing);
        assert_eq!(tree[0].children, vec![1]);
        assert_eq!(tree[1].parent, Some(0));
        assert_eq!(tree[1].ply_to_reach, Some(ply));
        assert_eq!(tree[1].turn, Player::Black);
        assert_eq!(agent.current_stats.nodes_created, 1);
    }

    #[test]
    fn expansion_does_not_create_child_for_already_terminal_node() {
        let config = BoardConfig::new(3, 3);
        let mut board = Board {
            white_board: 1 << 6,
            black_board: 1 << 8,
            turn: Player::Black,
        };
        let mut agent = test_agent(SelectionStrategy::Ucb1);
        let mut tree = vec![MctsNode::new(
            None,
            None,
            vec![Ply { from: 8, to: 5 }],
            board.turn,
        )];

        let (leaf_idx, status) = agent.expand_node(&mut tree, 0, &mut board, &config);

        assert_eq!(leaf_idx, 0);
        assert_eq!(status, Status::WhiteWon);
        assert_eq!(tree.len(), 1);
        assert!(tree[0].children.is_empty());
        assert_eq!(agent.current_stats.nodes_created, 0);
    }

    #[test]
    fn expansion_backpropagates_from_new_child() {
        let config = BoardConfig::new(3, 3);
        let mut board = Board {
            white_board: 1 << 3,
            black_board: 1 << 8,
            turn: Player::White,
        };
        let winning_ply = Ply { from: 3, to: 6 };

        let mut agent = test_agent(SelectionStrategy::Ucb1);
        let mut tree = vec![MctsNode::new(None, None, vec![winning_ply], board.turn)];

        let (leaf_idx, status) = agent.expand_node(&mut tree, 0, &mut board, &config);
        assert_eq!(leaf_idx, 1);
        assert_eq!(status, Status::WhiteWon);

        let mut rave_tracker = RaveTracker::Disabled;
        agent.backpropagate(&mut tree, leaf_idx, status, &mut rave_tracker);

        assert_eq!(tree[0].visits, 1);
        assert_eq!(tree[0].wins, 0.0);
        assert_eq!(tree[1].visits, 1);
        assert_eq!(tree[1].wins, 1.0);
    }

    #[test]
    fn backpropagation_credits_black_child_on_black_win() {
        let mut root = MctsNode::new(None, None, Vec::new(), Player::Black);
        root.children = vec![1];
        let child = MctsNode::new(
            Some(0),
            Some(Ply { from: 5, to: 2 }),
            Vec::new(),
            Player::White,
        );
        let mut tree = vec![root, child];
        let mut agent = test_agent(SelectionStrategy::Ucb1);
        let mut rave_tracker = RaveTracker::Disabled;

        agent.backpropagate(&mut tree, 1, Status::BlackWon, &mut rave_tracker);

        assert_eq!(tree[0].visits, 1);
        assert_eq!(tree[0].wins, 0.0);
        assert_eq!(tree[1].visits, 1);
        assert_eq!(tree[1].wins, 1.0);
    }

    #[test]
    fn do_playout_and_backprop_updates_leaf_and_counts_steps() {
        let config = BoardConfig::new(4, 4);
        let mut board = Board {
            white_board: 1 << 4,
            black_board: 1 << 15,
            turn: Player::White,
        };
        board.apply_ply(Ply { from: 4, to: 8 });
        let mut root = MctsNode::new(None, None, Vec::new(), Player::White);
        root.children = vec![1];
        let child = MctsNode::new(
            Some(0),
            Some(Ply { from: 4, to: 8 }),
            Vec::new(),
            Player::Black,
        );
        let mut tree = vec![root, child];
        let mut agent = test_agent(SelectionStrategy::Ucb1);

        let (status, steps) = agent.do_playout_and_backprop(&mut tree, 1, &mut board, &config);

        assert_ne!(status, Status::Ongoing);
        assert!(steps > 0);
        assert_eq!(tree[1].visits, 1);
        assert_eq!(tree[0].visits, 1);
    }

    #[test]
    fn select_ply_returns_none_when_side_to_move_has_no_pieces() {
        let config = BoardConfig::new(3, 3);
        let board = Board {
            white_board: 0,
            black_board: 1 << 8,
            turn: Player::White,
        };
        let mut agent = test_agent(SelectionStrategy::Ucb1);

        let selected = agent.select_ply(&board, &config);

        assert_eq!(selected, None);
        assert_eq!(agent.current_stats.iterations, 0);
    }

    #[test]
    fn select_ply_with_one_iteration_returns_a_legal_root_move() {
        let config = BoardConfig::new(3, 3);
        let board = Board {
            white_board: 1 << 3,
            black_board: 1 << 8,
            turn: Player::White,
        };
        let legal_plies = board.get_legal_plies(&config);
        let mut agent = test_agent(SelectionStrategy::Ucb1);

        let selected = agent.select_ply(&board, &config);

        assert!(selected.is_some());
        assert!(legal_plies.contains(&selected.unwrap()));
        assert_eq!(agent.current_stats.iterations, 1);
        assert_eq!(agent.current_stats.nodes_created, 2);
    }

    #[test]
    fn terminal_expansion_records_path_move_for_rave() {
        let config = BoardConfig::new(3, 3);
        let mut board = Board {
            white_board: 1 << 3,
            black_board: 1 << 8,
            turn: Player::White,
        };
        let winning_ply = Ply { from: 3, to: 6 };

        let mut agent = test_agent(SelectionStrategy::Rave { k: 1000.0 });
        let mut tree = vec![MctsNode::new(None, None, vec![winning_ply], board.turn)];

        let (leaf_idx, status) = agent.expand_node(&mut tree, 0, &mut board, &config);
        let mut rave_tracker = agent.new_rave_tracker();

        agent.backpropagate(&mut tree, leaf_idx, status, &mut rave_tracker);

        assert_eq!(tree[1].amaf.visits, 1);
        assert_eq!(tree[1].amaf.wins, 1.0);
    }
}
