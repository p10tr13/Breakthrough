use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Debug, Default, Display, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Player {
    #[default]
    White,
    Black,
}

impl Player {
    pub fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    pub fn is_white(self) -> bool {
        matches!(self, Self::White)
    }

    pub fn is_black(self) -> bool {
        matches!(self, Self::Black)
    }
}
