use crate::core::{Player, Ply, Status};

use super::node::MctsNode;

const MAX_BOARD_SQUARES: usize = 128;

#[derive(Debug)]
pub enum RaveTracker {
    Disabled,
    Enabled(Box<MoveTracking>),
}

impl RaveTracker {
    pub fn new(enabled: bool) -> Self {
        if enabled {
            Self::Enabled(Box::new(MoveTracking::new()))
        } else {
            Self::Disabled
        }
    }

    pub fn record_playout_move(&mut self, player: Player, ply: Ply) {
        if let Self::Enabled(tracking) = self {
            tracking.record_move(player, ply);
        }
    }

    pub fn record_path_move(&mut self, player: Player, ply: Ply) {
        if let Self::Enabled(tracking) = self {
            tracking.record_move(player, ply);
        }
    }

    pub fn update_amaf_children(&self, tree: &mut [MctsNode], node_idx: usize, status: Status) {
        let Self::Enabled(tracking) = self else {
            return;
        };

        let node_turn = tree[node_idx].turn;
        let node_turn_won = status.is_won_by(node_turn);
        let children_len = tree[node_idx].children.len();

        for child_position in 0..children_len {
            let child_idx = tree[node_idx].children[child_position];
            let child = &mut tree[child_idx];
            if let Some(ply) = child.ply_to_reach
                && tracking.contains_move(node_turn, ply)
            {
                child.amaf.visits += 1;
                if node_turn_won {
                    child.amaf.wins += 1.0;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct MoveTracking {
    // The board is represented as u128 bitboards, so legal square ids are always in 0..128.
    white_moves: [[bool; MAX_BOARD_SQUARES]; MAX_BOARD_SQUARES],
    black_moves: [[bool; MAX_BOARD_SQUARES]; MAX_BOARD_SQUARES],
}

impl MoveTracking {
    fn new() -> Self {
        Self {
            white_moves: [[false; MAX_BOARD_SQUARES]; MAX_BOARD_SQUARES],
            black_moves: [[false; MAX_BOARD_SQUARES]; MAX_BOARD_SQUARES],
        }
    }

    fn record_move(&mut self, player: Player, ply: Ply) {
        if player.is_white() {
            self.white_moves[ply.from as usize][ply.to as usize] = true;
        } else {
            self.black_moves[ply.from as usize][ply.to as usize] = true;
        }
    }

    fn contains_move(&self, player: Player, ply: Ply) -> bool {
        if player.is_white() {
            self.white_moves[ply.from as usize][ply.to as usize]
        } else {
            self.black_moves[ply.from as usize][ply.to as usize]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::BoardConfig;

    fn root_with_children() -> Vec<MctsNode> {
        let mut root = MctsNode::new(None, None, Vec::new(), Player::White);
        root.children = vec![1, 2];

        let matching_child = MctsNode::new(
            Some(0),
            Some(Ply { from: 1, to: 4 }),
            Vec::new(),
            Player::Black,
        );
        let other_child = MctsNode::new(
            Some(0),
            Some(Ply { from: 2, to: 5 }),
            Vec::new(),
            Player::Black,
        );

        vec![root, matching_child, other_child]
    }

    #[test]
    fn disabled_tracker_does_not_update_amaf_stats() {
        let mut tree = root_with_children();
        let tracker = RaveTracker::Disabled;

        tracker.update_amaf_children(&mut tree, 0, Status::WhiteWon);

        assert_eq!(tree[1].amaf.visits, 0);
        assert_eq!(tree[2].amaf.visits, 0);
    }

    #[test]
    fn playout_move_updates_matching_child_for_same_player() {
        let mut tree = root_with_children();
        let mut tracker = RaveTracker::new(true);

        tracker.record_playout_move(Player::White, Ply { from: 1, to: 4 });
        tracker.update_amaf_children(&mut tree, 0, Status::WhiteWon);

        assert_eq!(tree[1].amaf.visits, 1);
        assert_eq!(tree[1].amaf.wins, 1.0);
        assert_eq!(tree[2].amaf.visits, 0);
    }

    #[test]
    fn move_from_opponent_does_not_update_node_turn_children() {
        let mut tree = root_with_children();
        let mut tracker = RaveTracker::new(true);

        tracker.record_playout_move(Player::Black, Ply { from: 1, to: 4 });
        tracker.update_amaf_children(&mut tree, 0, Status::WhiteWon);

        assert_eq!(tree[1].amaf.visits, 0);
        assert_eq!(tree[1].amaf.wins, 0.0);
    }

    #[test]
    fn amaf_visit_is_recorded_even_when_node_turn_loses() {
        let mut tree = root_with_children();
        let mut tracker = RaveTracker::new(true);

        tracker.record_playout_move(Player::White, Ply { from: 1, to: 4 });
        tracker.update_amaf_children(&mut tree, 0, Status::BlackWon);

        assert_eq!(tree[1].amaf.visits, 1);
        assert_eq!(tree[1].amaf.wins, 0.0);
    }

    #[test]
    fn path_move_can_be_used_by_ancestor_updates() {
        let mut tree = root_with_children();
        let mut tracker = RaveTracker::new(true);

        tracker.record_path_move(Player::White, Ply { from: 1, to: 4 });
        tracker.update_amaf_children(&mut tree, 0, Status::WhiteWon);

        assert_eq!(tree[1].amaf.visits, 1);
        assert_eq!(tree[1].amaf.wins, 1.0);
    }

    #[test]
    fn move_tracking_supports_highest_board_square() {
        let config = BoardConfig::new(16, 8);
        assert_eq!(config.width * config.height, 128);

        let mut tracker = MoveTracking::new();
        let ply = Ply { from: 126, to: 127 };
        tracker.record_move(Player::White, ply);

        assert!(tracker.contains_move(Player::White, ply));
        assert!(!tracker.contains_move(Player::Black, ply));
    }
}
