#[derive(Debug, Default, Clone)]
pub struct MinimaxStats {
    pub nodes_evaluated: u64,
    pub cutoffs: u64,
}

impl MinimaxStats {
    pub fn new(nodes_evaluated: u64, cutoffs: u64) -> Self {
        Self {
            nodes_evaluated,
            cutoffs,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MinimaxStatsAccumulator {
    pub total_nodes: u64,
    pub total_cutoffs: u64,
}

impl MinimaxStatsAccumulator {
    pub fn accumulate(&mut self, stats: &MinimaxStats) {
        self.total_nodes += stats.nodes_evaluated;
        self.total_cutoffs += stats.cutoffs;
    }
}
