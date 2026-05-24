use rand::RngExt;
use rand::rngs::SmallRng;

use super::super::minimax::PositionEvaluator;
use crate::core::{Board, BoardConfig, Ply};

/// Strategy for selecting moves during the playout phase
#[derive(Debug)]
pub enum PlayoutStrategy {
    /// Uniformly random move selection
    Random,
    /// Heuristic-guided move selection with epsilon-greedy exploration
    Heavy {
        evaluator: Box<dyn PositionEvaluator>,
        epsilon: f64,
    },
}

impl PlayoutStrategy {
    pub fn random() -> Self {
        Self::Random
    }

    pub fn heavy(evaluator: Box<dyn PositionEvaluator>, epsilon: f64) -> Self {
        Self::Heavy { evaluator, epsilon }
    }

    /// Select a move according to this strategy
    pub fn select_move(
        &self,
        plies: &[Ply],
        board: &Board,
        config: &BoardConfig,
        rng: &mut SmallRng,
    ) -> Ply {
        match self {
            PlayoutStrategy::Random => plies[rng.random_range(0..plies.len())],
            PlayoutStrategy::Heavy { evaluator, epsilon } => {
                // Epsilon-greedy: random with probability epsilon, greedy otherwise
                if rng.random_range(0.0..1.0) < *epsilon {
                    return plies[rng.random_range(0..plies.len())];
                }

                // Greedy: select best move according to heuristic
                let mut best_ply = plies[0];
                let mut best_score = if board.turn.is_white() {
                    i32::MIN
                } else {
                    i32::MAX
                };

                for &ply in plies {
                    let mut next_board = *board;
                    next_board.apply_ply(ply);
                    let score = evaluator.evaluate(&next_board, config);

                    if (board.turn.is_white() && score > best_score)
                        || (!board.turn.is_white() && score < best_score)
                    {
                        best_score = score;
                        best_ply = ply;
                    }
                }

                best_ply
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::HeuristicEvaluator;
    use crate::core::Player;
    use rand::SeedableRng;

    fn rng() -> SmallRng {
        SmallRng::seed_from_u64(0)
    }

    #[test]
    fn random_playout_returns_the_only_available_move() {
        let config = BoardConfig::new(3, 3);
        let board = Board {
            white_board: 1 << 3,
            black_board: 1 << 8,
            turn: Player::White,
        };
        let plies = [Ply { from: 3, to: 6 }];

        let selected = PlayoutStrategy::Random.select_move(&plies, &board, &config, &mut rng());

        assert_eq!(selected, plies[0]);
    }

    #[test]
    fn heavy_playout_greedily_selects_white_terminal_win() {
        let config = BoardConfig::new(3, 3);
        let board = Board {
            white_board: (1 << 0) | (1 << 3),
            black_board: 1 << 8,
            turn: Player::White,
        };
        let winning_ply = Ply { from: 3, to: 6 };
        let plies = [Ply { from: 0, to: 4 }, winning_ply];
        let strategy = PlayoutStrategy::Heavy {
            evaluator: Box::new(HeuristicEvaluator::default()),
            epsilon: 0.0,
        };

        let selected = strategy.select_move(&plies, &board, &config, &mut rng());

        assert_eq!(selected, winning_ply);
    }

    #[test]
    fn heavy_playout_greedily_selects_black_terminal_win() {
        let config = BoardConfig::new(3, 3);
        let board = Board {
            white_board: 1 << 3,
            black_board: (1 << 5) | (1 << 8),
            turn: Player::Black,
        };
        let winning_ply = Ply { from: 5, to: 2 };
        let plies = [Ply { from: 8, to: 4 }, winning_ply];
        let strategy = PlayoutStrategy::Heavy {
            evaluator: Box::new(HeuristicEvaluator::default()),
            epsilon: 0.0,
        };

        let selected = strategy.select_move(&plies, &board, &config, &mut rng());

        assert_eq!(selected, winning_ply);
    }

    #[test]
    fn heavy_playout_with_full_epsilon_uses_random_branch() {
        let config = BoardConfig::new(3, 3);
        let board = Board {
            white_board: 1 << 3,
            black_board: 1 << 8,
            turn: Player::White,
        };
        let plies = [Ply { from: 3, to: 6 }];
        let strategy = PlayoutStrategy::Heavy {
            evaluator: Box::new(HeuristicEvaluator::default()),
            epsilon: 1.0,
        };

        let selected = strategy.select_move(&plies, &board, &config, &mut rng());

        assert_eq!(selected, plies[0]);
    }
}
