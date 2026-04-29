mod minimax;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AgentConfig {
    Minimax { max_depth: u8 },
    Human,
}
