#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}
