use serde::{Deserialize, Serialize};

use crate::agents::metrics::CommonMetrics;

#[derive(Debug, Serialize, Deserialize)]
pub struct MctsMetrics {
    #[serde(flatten)]
    pub common: CommonMetrics,
    pub max_iterations: u32,
    pub max_time_ms: Option<u64>,
    pub exploration_constant: f64,

    pub use_rave: bool,
    pub use_heavy_playouts: bool,

    pub rave_k: Option<f64>,
    pub heavy_playouts_epsilon: Option<f64>,
    pub material_weight: Option<i32>,
    pub advancement_weight: Option<i32>,
    pub defended_weight: Option<i32>,
    pub edge_penalty_weight: Option<i32>,

    pub total_iterations: u64,
    pub total_nodes_created: u64,
}
