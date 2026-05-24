/// Selection strategy for the tree search phase
#[derive(Debug, Clone)]
pub enum SelectionStrategy {
    /// UCB1 (Upper Confidence Bound) - vanilla MCTS selection
    Ucb1,
    /// RAVE (Rapid Action Value Estimation) - uses AMAF (all-moves-as-first heuristic)
    Rave { k: f64 },
}

impl SelectionStrategy {
    pub fn ucb1() -> Self {
        Self::Ucb1
    }

    pub fn rave(k: f64) -> Self {
        Self::Rave { k }
    }

    /// Computes the UCT score for a child node
    pub fn compute_uct_score(
        &self,
        child_visits: u32,
        child_wins: f64,
        child_amaf_visits: u32,
        child_amaf_wins: f64,
        parent_visits: u32,
        exploration_constant: f64,
    ) -> f64 {
        if child_visits == 0 {
            return f64::INFINITY; // Unvisited nodes have highest priority
        }

        let exploitation = child_wins / (child_visits as f64);
        let exploration =
            exploration_constant * ((parent_visits as f64).ln() / (child_visits as f64)).sqrt();

        match self {
            SelectionStrategy::Ucb1 => exploitation + exploration,
            SelectionStrategy::Rave { k } => {
                if child_amaf_visits > 0 {
                    let amaf_exploitation = child_amaf_wins / (child_amaf_visits as f64);
                    let beta = k / (k + child_visits as f64);
                    (1.0 - beta) * exploitation + beta * amaf_exploitation + exploration
                } else {
                    exploitation + exploration
                }
            }
        }
    }

    /// Returns true if this strategy uses AMAF tracking
    pub fn uses_amaf(&self) -> bool {
        matches!(self, SelectionStrategy::Rave { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unvisited_child_has_infinite_priority() {
        let score = SelectionStrategy::Ucb1.compute_uct_score(0, 0.0, 0, 0.0, 10, 1.41);

        assert!(score.is_infinite());
        assert!(score.is_sign_positive());
    }

    #[test]
    fn ucb1_combines_exploitation_and_exploration() {
        let score = SelectionStrategy::Ucb1.compute_uct_score(4, 3.0, 0, 0.0, 16, 2.0);
        let expected = 0.75_f64 + 2.0_f64 * (16.0_f64.ln() / 4.0_f64).sqrt();

        assert!((score - expected).abs() < 1e-12);
    }

    #[test]
    fn rave_falls_back_to_ucb_when_amaf_has_no_visits() {
        let ucb = SelectionStrategy::Ucb1.compute_uct_score(4, 3.0, 0, 0.0, 16, 1.41);
        let rave =
            SelectionStrategy::Rave { k: 1000.0 }.compute_uct_score(4, 3.0, 0, 0.0, 16, 1.41);

        assert!((rave - ucb).abs() < 1e-12);
    }

    #[test]
    fn rave_blends_amaf_and_direct_value() {
        let score = SelectionStrategy::Rave { k: 4.0 }.compute_uct_score(4, 1.0, 8, 6.0, 16, 0.0);

        let beta = 4.0 / (4.0 + 4.0);
        let expected = (1.0 - beta) * 0.25 + beta * 0.75;

        assert!((score - expected).abs() < 1e-12);
    }

    #[test]
    fn uses_amaf_only_for_rave() {
        assert!(!SelectionStrategy::Ucb1.uses_amaf());
        assert!(SelectionStrategy::Rave { k: 1000.0 }.uses_amaf());
    }
}
