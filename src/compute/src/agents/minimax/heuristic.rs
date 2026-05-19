#![allow(dead_code)]

use crate::core::{Board, BoardConfig, Status};
use std::fmt::Debug;

const SCORE_WIN: i32 = 1e6 as i32;
const SCORE_LOSS: i32 = -1e6 as i32;

pub trait PositionEvaluator: Send + Debug {
    fn evaluate(&self, board: &Board, config: &BoardConfig) -> i32;
    fn score_loss(&self) -> i32;
    fn score_win(&self) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeuristicEvaluator {
    pub material_weight: i32,
    pub advancement_weight: i32,
    pub defended_weight: i32,
    pub edge_penalty_weight: i32,
}

impl Default for HeuristicEvaluator {
    fn default() -> Self {
        Self {
            material_weight: 10,
            advancement_weight: 10,
            defended_weight: 5,
            edge_penalty_weight: -2,
        }
    }
}

impl HeuristicEvaluator {
    pub fn new(
        material_weight: i32,
        advancement_weight: i32,
        defended_weight: i32,
        edge_penalty_weight: i32,
    ) -> Self {
        Self {
            material_weight,
            advancement_weight,
            defended_weight,
            edge_penalty_weight,
        }
    }

    fn evaluate_material_advantage(&self, board: &Board) -> i32 {
        let white_count = board.white_board.count_ones() as i32;
        let black_count = board.black_board.count_ones() as i32;
        white_count - black_count
    }

    fn advancement_score(&self, board: u128, is_white: bool, config: &BoardConfig) -> i32 {
        let mut score = 0;
        let mut pieces = board;
        let height = config.height as i32;
        let width = config.width as i32;

        while pieces != 0 {
            let square = pieces.trailing_zeros() as i32;
            let row = square / width;
            let advancement = if is_white { row } else { (height - 1) - row };
            score += 1 << advancement;
            pieces &= pieces - 1;
        }
        score
    }

    fn evaluate_piece_advancement(&self, board: &Board, config: &BoardConfig) -> i32 {
        let white_advancement = self.advancement_score(board.white_board, true, config);
        let black_advancement = self.advancement_score(board.black_board, false, config);

        white_advancement - black_advancement
    }

    fn evaluate_defended_pieces(&self, board: &Board, config: &BoardConfig) -> i32 {
        let width = config.width;

        let white_defends_left = (board.white_board & config.not_left_edge) << (width - 1);
        let white_defends_right = (board.white_board & config.not_right_edge) << (width + 1);
        let white_defense_mask = white_defends_left | white_defends_right;

        let white_defended_count = (board.white_board & white_defense_mask).count_ones() as i32;

        let black_defends_left = (board.black_board & config.not_left_edge) >> (width + 1);
        let black_defends_right = (board.black_board & config.not_right_edge) >> (width - 1);
        let black_defense_mask = black_defends_left | black_defends_right;

        let black_defended_count = (board.black_board & black_defense_mask).count_ones() as i32;

        white_defended_count - black_defended_count
    }

    fn evaluate_edge_penalty(&self, board: &Board, config: &BoardConfig) -> i32 {
        let edge_mask = !config.not_left_edge | !config.not_right_edge;

        let valid_edge_mask = edge_mask & config.valid_board_mask;

        let white_on_edges = (board.white_board & valid_edge_mask).count_ones() as i32;
        let black_on_edges = (board.black_board & valid_edge_mask).count_ones() as i32;

        white_on_edges - black_on_edges
    }
}

impl PositionEvaluator for HeuristicEvaluator {
    fn evaluate(&self, board: &Board, config: &BoardConfig) -> i32 {
        match board.get_status(config) {
            Status::WhiteWon => return self.score_win(),
            Status::BlackWon => return self.score_loss(),
            Status::Ongoing => {}
        }

        let material_score = self.evaluate_material_advantage(board);
        let advancement_score = self.evaluate_piece_advancement(board, config);
        let defended_score = self.evaluate_defended_pieces(board, config);
        let edge_score = self.evaluate_edge_penalty(board, config);

        (self.material_weight * material_score)
            + (self.advancement_weight * advancement_score)
            + (self.defended_weight * defended_score)
            + (self.edge_penalty_weight * edge_score)
    }

    fn score_loss(&self) -> i32 {
        SCORE_LOSS
    }

    fn score_win(&self) -> i32 {
        SCORE_WIN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Player;

    #[test]
    fn test_evaluate_material_advantage() {
        let evaluator = HeuristicEvaluator::default();

        let board = Board {
            white_board: 0b1011, // 3 pawns
            black_board: 0b0100, // 1 pawn
            turn: Player::White,
        };

        // 3 - 1 = 2
        assert_eq!(evaluator.evaluate_material_advantage(&board), 2);
    }

    #[test]
    fn test_evaluate_piece_advancement() {
        let config = BoardConfig::new(4, 4);
        let evaluator = HeuristicEvaluator::default();

        // White advancement: 1 << (9 / 4) = 1 << 2 = 4
        // Black advancement: 1 << ((4 - 1) - (14 / 4)) = 1 << (3 - 3) = 1 << 0 = 1
        let board = Board {
            white_board: 1 << 9,
            black_board: 1 << 14,
            turn: Player::White,
        };

        // 4 - 1 = 3
        assert_eq!(evaluator.evaluate_piece_advancement(&board, &config), 3);
    }

    #[test]
    fn test_evaluate_defended_pieces() {
        let config = BoardConfig::new(3, 3);
        let evaluator = HeuristicEvaluator::default();

        // 2 defended white pawns
        let white_board = (1 << 1) | (1 << 3) | (1 << 5);

        // 1 defended black pawn
        let black_board = (1 << 7) | (1 << 3);

        let board = Board {
            white_board,
            black_board,
            turn: Player::White,
        };

        // 2 - 1 = 1
        assert_eq!(evaluator.evaluate_defended_pieces(&board, &config), 1);
    }

    #[test]
    fn test_evaluate_edge_penalty() {
        let config = BoardConfig::new(3, 3);
        let evaluator = HeuristicEvaluator::default();

        // edges on a 3x3 board are columns 0 (0,3,6) and 2 (2,5,8).
        // White: 2 pawns on edge, one at center
        let white_board = (1 << 0) | (1 << 4) | (1 << 5);

        // Black: 1 pawn on edge
        let black_board = 1 << 8;

        let board = Board {
            white_board,
            black_board,
            turn: Player::White,
        };

        // 2 - 1 = 1
        assert_eq!(evaluator.evaluate_edge_penalty(&board, &config), 1);
    }

    #[test]
    fn test_terminal_evaluation() {
        let config = BoardConfig::new(4, 4);
        let evaluator = HeuristicEvaluator::default();

        let mut board = Board::new(4, 4);

        // breakthrough condition
        board.white_board = 1 << 13;
        board.black_board = 1 << 0;
        assert_eq!(evaluator.evaluate(&board, &config), evaluator.score_win());

        // wipeout condition - no white pieces left
        board.white_board = 0;
        board.black_board = 1 << 10;
        assert_eq!(evaluator.evaluate(&board, &config), evaluator.score_loss());
    }

    #[test]
    fn test_overall_weighted_evaluation() {
        let config = BoardConfig::new(3, 3);
        // material=10, advancement=40, defense=5, edges=-2
        let evaluator = HeuristicEvaluator::new(10, 40, 5, -2);

        // White: 1 (bottom center), 3 (left edge), 5 (right edge)
        // Black: 7 (top center)
        let board = Board {
            white_board: (1 << 1) | (1 << 3) | (1 << 5),
            black_board: 1 << 7,
            turn: Player::White,
        };

        // expected result calculation:
        // Material: (3 - 1) * 10 = 20
        // Advancement:
        // White: (1<<0) + (1<<1) + (1<<1) = 1 + 2 + 2 = 5
        // Black: 1<<(2-2) = 1<<0 = 1
        // (5 - 1) * 40 = 160
        // Defense: White(2) - Black(0) = 2. 2 * 5 = 10
        // Edges: White(2) - Black(0) = 2. 2 * -2 = -4
        // Total: 20 + 160 + 10 - 4 = 186

        assert_eq!(evaluator.evaluate(&board, &config), 186);
    }
}
