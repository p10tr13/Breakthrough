use crate::core::{Player, Ply};

#[derive(Debug, Default, Clone, Copy)]
pub struct AmafStats {
    pub visits: u32,
    pub wins: f64,
}

#[derive(Debug, Default)]
pub struct MctsNode {
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub ply_to_reach: Option<Ply>,
    pub unexpanded_plies: Vec<Ply>,

    pub visits: u32,
    pub wins: f64,

    pub turn: Player,

    pub amaf: AmafStats,
}

impl MctsNode {
    pub fn new(
        parent: Option<usize>,
        ply_to_reach: Option<Ply>,
        unexpanded_plies: Vec<Ply>,
        turn: Player,
    ) -> Self {
        Self {
            parent,
            ply_to_reach,
            unexpanded_plies,
            turn,
            ..Default::default()
        }
    }
}
