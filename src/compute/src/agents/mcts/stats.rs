#[derive(Debug, Default, Clone)]
pub struct MctsStats {
    pub iterations: u32,
    pub nodes_created: u32,
    pub playout_steps: u64,
}

impl MctsStats {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct MctsStatsAccumulator {
    pub total_iterations: u64,
    pub total_nodes_created: u64,
    pub total_playout_steps: u64,
}

impl MctsStatsAccumulator {
    pub fn accumulate(&mut self, stats: &MctsStats) {
        self.total_iterations += stats.iterations as u64;
        self.total_nodes_created += stats.nodes_created as u64;
        self.total_playout_steps += stats.playout_steps;
    }
}
