#![allow(dead_code)]

mod heuristic;

use heuristic::PositionEvaluator;

use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

use crate::core::{Board, BoardConfig, Player, Ply, Status};

pub struct MinimaxAgent {
    pub max_depth: u8,
    pub heuristic: Box<dyn PositionEvaluator>,
    pub rng: SmallRng,
}

impl MinimaxAgent {
    pub fn new(max_depth: u8, heuristic: Box<dyn PositionEvaluator>, seed: u64) -> Self {
        Self {
            max_depth,
            heuristic,
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn get_ply(&mut self, board: &Board, config: &BoardConfig) -> Option<Ply> {
        let alpha = self.heuristic.score_loss() * 2;
        let beta = self.heuristic.score_win() * 2;

        let (_, best_move) = self.alphabeta(*board, self.max_depth, alpha, beta, config);
        best_move
    }

    fn alphabeta(
        &mut self,
        board: Board,
        depth: u8,
        mut alpha: i32,
        mut beta: i32,
        config: &BoardConfig,
    ) -> (i32, Option<Ply>) {
        let higher_score = self.heuristic.score_win() + depth as i32;
        let lower_score = self.heuristic.score_loss() - depth as i32;
        let status = board.get_status(config);

        if status == Status::WhiteWon {
            return (higher_score, None);
        } else if status == Status::BlackWon {
            return (lower_score, None);
        }

        if depth == 0 {
            return (self.heuristic.evaluate(&board, config), None);
        }

        let mut legal_plies = board.get_legal_plies(config);

        if legal_plies.is_empty() {
            if board.turn == Player::White {
                return (lower_score, None);
            } else {
                return (higher_score, None);
            }
        }

        let is_maximizing = board.turn == crate::core::Player::White;
        let cols = config.width as usize;

        legal_plies.sort_unstable_by_key(|ply| {
            let target_mask = 1_u128 << ply.to;

            let is_capture = if is_maximizing {
                (board.black_board & target_mask) != 0
            } else {
                (board.white_board & target_mask) != 0
            };

            let to_row = (ply.to as usize) / cols;
            let progression = if is_maximizing {
                to_row
            } else {
                (config.height as usize - 1) - to_row
            };

            std::cmp::Reverse((is_capture, progression))
        });

        let mut best_move = None;

        if is_maximizing {
            let mut max_eval = self.heuristic.score_loss() * 2;
            let mut ties_count = 0;

            for ply in legal_plies {
                let mut next_board = board;
                next_board.apply_ply(ply);

                let (eval, _) = self.alphabeta(next_board, depth - 1, alpha, beta, config);

                if eval > max_eval {
                    max_eval = eval;
                    best_move = Some(ply);
                    ties_count = 1;
                } else if eval == max_eval && depth == self.max_depth {
                    ties_count += 1;

                    if self.rng.random_range(0..ties_count) == 0 {
                        best_move = Some(ply);
                    }
                }

                alpha = alpha.max(eval);
                if beta <= alpha {
                    break;
                }
            }
            (max_eval, best_move)
        } else {
            let mut min_eval = self.heuristic.score_win() * 2;
            let mut ties_count = 0;

            for ply in legal_plies {
                let mut next_board = board;
                next_board.apply_ply(ply);

                let (eval, _) = self.alphabeta(next_board, depth - 1, alpha, beta, config);

                if eval < min_eval {
                    min_eval = eval;
                    best_move = Some(ply);
                    ties_count = 1;
                } else if eval == min_eval && depth == self.max_depth {
                    ties_count += 1;
                    if self.rng.random_range(0..ties_count) == 0 {
                        best_move = Some(ply);
                    }
                }

                beta = beta.min(eval);
                if beta <= alpha {
                    break;
                }
            }
            (min_eval, best_move)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agents::minimax::heuristic::HeuristicEvaluator,
        core::{Board, BoardConfig, Player, Ply},
    };

    fn create_agent() -> MinimaxAgent {
        MinimaxAgent::new(3, Box::new(HeuristicEvaluator::default()), 42)
    }

    #[test]
    fn test_minimax_finds_mate_in_one_for_white() {
        let config = BoardConfig::new(4, 4);
        let mut agent = create_agent();

        let board = Board {
            white_board: 1 << 9,
            black_board: 1 << 15,
            turn: Player::White,
        };

        let best_move = agent
            .get_ply(&board, &config)
            .expect("Agent should return a ply");
        let winning_targets = vec![12, 13, 14];
        assert!(winning_targets.contains(&best_move.to));
    }

    #[test]
    fn test_minimax_forces_defense() {
        let config = BoardConfig::new(4, 4);
        let mut agent = create_agent();

        let board = Board {
            white_board: 1 << 0,
            black_board: 1 << 5,
            turn: Player::White,
        };

        let best_move = agent
            .get_ply(&board, &config)
            .expect("Agent should return a ply");

        assert_eq!(best_move, Ply { from: 0, to: 5 });
    }

    #[test]
    fn test_minimax_finds_mate_in_one_for_black() {
        let config = BoardConfig::new(4, 4);
        let mut agent = create_agent();

        let board = Board {
            white_board: 1 << 9,
            black_board: 1 << 6,
            turn: Player::Black,
        };

        let best_move = agent
            .get_ply(&board, &config)
            .expect("Agent should return a ply");

        let winning_targets = vec![1, 2, 3];
        assert!(winning_targets.contains(&best_move.to));
    }
}
