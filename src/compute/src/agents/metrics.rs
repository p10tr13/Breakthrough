use std::{fs::OpenOptions, path::Path};

use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::io::Write;
use strum_macros::Display;

use crate::{ComputeResult, core::Player};

pub fn append_record_to_csv<T: Serialize, P: AsRef<Path>>(
    record: &T,
    path: P,
) -> ComputeResult<()> {
    let file_exists = path.as_ref().exists();

    let file = OpenOptions::new().create(true).append(true).open(path)?;

    let mut wtr = WriterBuilder::new()
        .has_headers(!file_exists)
        .from_writer(file);

    wtr.serialize(record)?;
    wtr.flush()?;

    Ok(())
}

pub fn append_record_to_jsonl<T: Serialize, P: AsRef<Path>>(
    record: &T,
    path: P,
) -> ComputeResult<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    let json_string = serde_json::to_string(record)?;

    writeln!(file, "{json_string}")?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Default, Display)]
pub enum AgentType {
    #[default]
    Human,
    Minimax,
    Mcts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommonMetrics {
    pub board_width: u8,
    pub board_height: u8,
    pub seed: Option<u64>,
    pub agent_type: AgentType,
    pub agent_color: Player,
    pub agent_won: bool,
    pub pieces_remaining: u8,
    pub total_time_ms: u128,
    pub total_moves: usize,
    pub moves: String,
    pub opponent_type: AgentType,
    pub opponent_pieces_remaining: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HumanMetrics {
    #[serde(flatten)]
    pub common: CommonMetrics,
}

impl HumanMetrics {
    pub fn new(common: CommonMetrics) -> Self {
        Self { common }
    }
}

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
