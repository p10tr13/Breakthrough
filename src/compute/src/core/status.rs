use strum_macros::Display;

use super::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Status {
    Ongoing,
    WhiteWon,
    BlackWon,
}

impl Status {
    pub fn is_ongoing(&self) -> bool {
        matches!(self, Status::Ongoing)
    }

    pub fn is_white_won(&self) -> bool {
        matches!(self, Status::WhiteWon)
    }

    pub fn is_black_won(&self) -> bool {
        matches!(self, Status::BlackWon)
    }

    pub fn is_won_by(&self, player: Player) -> bool {
        matches!(
            (self, player),
            (Status::WhiteWon, Player::White) | (Status::BlackWon, Player::Black)
        )
    }
}
