use serde::{Deserialize, Serialize};

use crate::agents::metrics::CommonMetrics;

#[derive(Debug, Serialize, Deserialize)]
pub struct MctsMetrics {
    #[serde(flatten)]
    pub common: CommonMetrics,
    pub max_iterations: u32,
    pub max_time_ms: Option<u64>,
    pub exploration_constant: f64,
    pub total_iterations: u64,
    pub total_nodes_created: u64,
}
