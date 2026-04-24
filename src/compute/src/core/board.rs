use super::{BoardConfig, Player, Ply, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board {
    pub white_board: u128,

    pub black_board: u128,

    pub turn: Player,
}

impl Board {
    pub fn new(width: u8, height: u8) -> Self {
        let total_squares = width * height;
        let mut white_board = 0_u128;
        let mut black_board = 0_u128;

        for i in 0..(2 * width) {
            white_board |= 1_u128 << i;
        }

        for i in (total_squares - 2 * width)..total_squares {
            black_board |= 1_u128 << i;
        }

        Self {
            white_board,
            black_board,
            turn: Player::White,
        }
    }

    pub fn apply_ply(&mut self, ply: Ply) {
        let from_mask = 1_u128 << ply.from;
        let to_mask = 1_u128 << ply.to;

        if self.turn == Player::White {
            self.white_board ^= from_mask; // remove pawn (XOR)
            self.white_board |= to_mask; // place pawn (OR)
            self.black_board &= !to_mask; // capture opponent's pawn, if any (AND NOT)
        } else {
            self.black_board ^= from_mask;
            self.black_board |= to_mask;
            self.white_board &= !to_mask;
        }

        self.turn = self.turn.opponent();
    }

    pub fn get_legal_plies_for_piece(&self, square: u8, config: &BoardConfig) -> Vec<Ply> {
        let piece_mask = 1_u128 << square;

        let occupied = self.white_board | self.black_board;
        let empty_squares = !occupied & config.valid_board_mask;
        let width = config.width;

        let mut plies = Vec::with_capacity(3);

        if self.turn == Player::White {
            if (self.white_board & piece_mask) == 0 {
                return plies;
            }

            let forward = (piece_mask << width) & empty_squares;
            let left = ((piece_mask & config.not_left_edge) << (width - 1))
                & !self.white_board
                & config.valid_board_mask;
            let right = ((piece_mask & config.not_right_edge) << (width + 1))
                & !self.white_board
                & config.valid_board_mask;

            if forward != 0 {
                plies.push(Ply {
                    from: square,
                    to: forward.trailing_zeros() as u8,
                });
            }
            if left != 0 {
                plies.push(Ply {
                    from: square,
                    to: left.trailing_zeros() as u8,
                });
            }
            if right != 0 {
                plies.push(Ply {
                    from: square,
                    to: right.trailing_zeros() as u8,
                });
            }
        } else {
            if (self.black_board & piece_mask) == 0 {
                return plies;
            }

            let forward = (piece_mask >> width) & empty_squares;
            let left = ((piece_mask & config.not_left_edge) >> (width + 1))
                & !self.black_board
                & config.valid_board_mask;
            let right = ((piece_mask & config.not_right_edge) >> (width - 1))
                & !self.black_board
                & config.valid_board_mask;

            if forward != 0 {
                plies.push(Ply {
                    from: square,
                    to: forward.trailing_zeros() as u8,
                });
            }
            if left != 0 {
                plies.push(Ply {
                    from: square,
                    to: left.trailing_zeros() as u8,
                });
            }
            if right != 0 {
                plies.push(Ply {
                    from: square,
                    to: right.trailing_zeros() as u8,
                });
            }
        }

        plies
    }

    pub fn get_legal_plies(&self, config: &BoardConfig) -> Vec<Ply> {
        match self.turn {
            Player::White => self.generate_white_plies(config),
            Player::Black => self.generate_black_plies(config),
        }
    }

    pub fn get_status(&self, config: &BoardConfig) -> Status {
        // breakthrough
        if (self.white_board & config.top_row_mask) != 0 {
            return Status::WhiteWon;
        }
        if (self.black_board & config.bottom_row_mask) != 0 {
            return Status::BlackWon;
        }

        // no opponent's pieces left
        if self.white_board == 0 {
            return Status::BlackWon;
        }
        if self.black_board == 0 {
            return Status::WhiteWon;
        }

        Status::Ongoing
    }

    fn generate_white_plies(&self, config: &BoardConfig) -> Vec<Ply> {
        let occupied = self.white_board | self.black_board;
        let empty_squares = !occupied & config.valid_board_mask;
        let width = config.width;

        let mut valid_forward = (self.white_board << width) & empty_squares;
        let mut valid_left = ((self.white_board & config.not_left_edge) << (width - 1))
            & !self.white_board
            & config.valid_board_mask;
        let mut valid_right = ((self.white_board & config.not_right_edge) << (width + 1))
            & !self.white_board
            & config.valid_board_mask;

        let mut plies = Vec::with_capacity(40);

        while valid_forward != 0 {
            let to = valid_forward.trailing_zeros() as u8;
            plies.push(Ply {
                from: to - width,
                to,
            });
            valid_forward &= valid_forward - 1;
        }

        while valid_left != 0 {
            let to = valid_left.trailing_zeros() as u8;
            plies.push(Ply {
                from: to - width + 1,
                to,
            });
            valid_left &= valid_left - 1;
        }

        while valid_right != 0 {
            let to = valid_right.trailing_zeros() as u8;
            plies.push(Ply {
                from: to - width - 1,
                to,
            });
            valid_right &= valid_right - 1;
        }

        plies
    }

    fn generate_black_plies(&self, config: &BoardConfig) -> Vec<Ply> {
        let occupied = self.white_board | self.black_board;
        let empty_squares = !occupied & config.valid_board_mask;
        let width = config.width;

        let mut valid_forward = (self.black_board >> width) & empty_squares;

        let mut valid_left = ((self.black_board & config.not_left_edge) >> (width + 1))
            & !self.black_board
            & config.valid_board_mask;

        let mut valid_right = ((self.black_board & config.not_right_edge) >> (width - 1))
            & !self.black_board
            & config.valid_board_mask;

        let mut plies = Vec::with_capacity(40);

        while valid_forward != 0 {
            let to = valid_forward.trailing_zeros() as u8;
            plies.push(Ply {
                from: to + width,
                to,
            });
            valid_forward &= valid_forward - 1;
        }

        while valid_left != 0 {
            let to = valid_left.trailing_zeros() as u8;
            plies.push(Ply {
                from: to + width + 1,
                to,
            });
            valid_left &= valid_left - 1;
        }

        while valid_right != 0 {
            let to = valid_right.trailing_zeros() as u8;
            plies.push(Ply {
                from: to + width - 1,
                to,
            });
            valid_right &= valid_right - 1;
        }

        plies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_initialization_4x4() {
        let board = Board::new(4, 4);

        assert_eq!(board.white_board, 0b0000_0000_1111_1111);
        assert_eq!(board.black_board, 0b1111_1111_0000_0000);
        assert_eq!(board.turn, Player::White);
    }

    #[test]
    fn test_apply_ply_move_and_capture() {
        let mut board = Board {
            white_board: 0b0001, // White piece at index 0
            black_board: 0b1010, // Black pieces at indices 1 and 3
            turn: Player::White,
        };

        // White moves forward (from 0 to 2) — no capture
        board.apply_ply(Ply { from: 0, to: 2 });
        assert_eq!(board.white_board, 0b0100);
        assert_eq!(board.black_board, 0b1010); // Black unchanged
        assert_eq!(board.turn, Player::Black);

        // Black captures White (from 3 to 2)
        board.apply_ply(Ply { from: 3, to: 2 });
        assert_eq!(board.black_board, 0b0110); // Black piece moved to 2
        assert_eq!(board.white_board, 0b0000); // White piece was captured
        assert_eq!(board.turn, Player::White);
    }

    #[test]
    fn test_get_status() {
        let config = BoardConfig::new(4, 4);

        // Game is ongoing
        let mut board = Board::new(4, 4);
        assert_eq!(board.get_status(&config), Status::Ongoing);

        // White wins — breakthrough (reaches top row)
        board.white_board = 1 << 14;
        assert_eq!(board.get_status(&config), Status::WhiteWon);

        // Black wins — breakthrough (reaches bottom row)
        board.white_board = 1 << 5; // Reset White (outside scoring zone)
        board.black_board = 1 << 2; // Black piece in the first row (indices 0–3)
        assert_eq!(board.get_status(&config), Status::BlackWon);

        // Black wins — wipeout (all White pieces captured)
        board.black_board = 1 << 8;
        board.white_board = 0;
        assert_eq!(board.get_status(&config), Status::BlackWon);

        // White wins — wipeout (all Black pieces captured)
        board.black_board = 0;
        board.white_board = 1 << 8;
        assert_eq!(board.get_status(&config), Status::WhiteWon);
    }

    #[test]
    fn test_get_legal_plies_for_single_piece() {
        let config = BoardConfig::new(4, 4);

        let board = Board {
            white_board: (1 << 1) | (1 << 2), // White pieces at indices 1 and 2
            black_board: 1 << 5, // Black piece blocks index 5 (directly in front of White at 1)
            turn: Player::White,
        };

        let plies = board.get_legal_plies_for_piece(1, &config);
        assert_eq!(plies.len(), 2);
        assert!(plies.contains(&Ply { from: 1, to: 4 }));
        assert!(plies.contains(&Ply { from: 1, to: 6 }));

        let empty_plies = board.get_legal_plies_for_piece(0, &config);
        assert!(empty_plies.is_empty());

        let opponent_plies = board.get_legal_plies_for_piece(5, &config);
        assert!(opponent_plies.is_empty());
    }

    #[test]
    fn test_generate_white_plies_blocking_and_capturing() {
        let config = BoardConfig::new(3, 3);

        // 3x3 board:
        // 6 7 8 (empty)
        // 3 4 5 (Black pieces)
        // 0 1 2 (one White piece at index 1)
        let board = Board {
            white_board: 1 << 1,
            black_board: (1 << 3) | (1 << 4) | (1 << 5),
            turn: Player::White,
        };

        let plies = board.get_legal_plies(&config);

        // Forward move (to 4) is blocked,
        // diagonal captures (to 3 and 5) should be legal
        assert_eq!(plies.len(), 2);

        // Verify expected moves are present (order may vary)
        assert!(plies.contains(&Ply { from: 1, to: 3 }));
        assert!(plies.contains(&Ply { from: 1, to: 5 }));
    }

    #[test]
    fn test_generate_black_plies_blocking_and_capturing() {
        let config = BoardConfig::new(3, 3);

        // 3x3 board:
        // 6 7 8 (one Black piece at index 7)
        // 3 4 5 (White pieces)
        // 0 1 2 (empty)
        let board = Board {
            white_board: (1 << 3) | (1 << 4) | (1 << 5),
            black_board: 1 << 7,
            turn: Player::Black,
        };

        let plies = board.get_legal_plies(&config);

        // Black moves "down". Forward move (to 4) is blocked by White.
        // Diagonal captures (to 3 and 5) should be allowed.
        assert_eq!(plies.len(), 2);
        assert!(plies.contains(&Ply { from: 7, to: 3 }));
        assert!(plies.contains(&Ply { from: 7, to: 5 }));
    }

    #[test]
    fn test_edge_wrapping_prevention() {
        let config = BoardConfig::new(3, 3);

        // White piece on the left edge (index 3)
        let board = Board {
            white_board: 1 << 3,
            black_board: 0,
            turn: Player::White,
        };

        let plies = board.get_legal_plies(&config);

        // From the left edge (3), White should only be able to move:
        // forward (to 6) or diagonally right (to 7).
        // Diagonal left would wrap around to index 5 — this must be prevented!
        assert_eq!(plies.len(), 2);
        assert!(plies.contains(&Ply { from: 3, to: 6 }));
        assert!(plies.contains(&Ply { from: 3, to: 7 }));
        assert!(!plies.contains(&Ply { from: 3, to: 5 })); // Ensure no wrap-around occurred
    }
}
