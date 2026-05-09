use serde::{Deserialize, Serialize};

use crate::agents::metrics::CommonMetrics;

#[derive(Debug, Serialize, Deserialize)]
pub struct MinimaxMetrics {
    #[serde(flatten)]
    pub common: CommonMetrics,
    pub max_depth: u8,
    pub total_nodes_evaluated: u64,
    pub total_cutoffs: u64,
    pub material_weight: i32,
    pub advancement_weight: i32,
    pub defended_weight: i32,
    pub edge_penalty_weight: i32,
}
