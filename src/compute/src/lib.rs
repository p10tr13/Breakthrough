mod error;

pub mod agents;
pub mod cli;
pub mod core;
pub mod defaults;
pub mod gui;

pub use error::{ComputeError, ComputeResult};

use serde::{Deserialize, Serialize};

use crate::agents::AgentConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BreakthroughConfig {
    pub board_width: u8,
    pub board_height: u8,
    pub white_player: AgentConfig,
    pub black_player: AgentConfig,
    pub seed: Option<u64>,
    #[serde(default, alias = "blind_mode")]
    pub study_mode: bool,
}
