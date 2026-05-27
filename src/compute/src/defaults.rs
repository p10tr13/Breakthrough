pub const DEFAULT_MINIMAX_MAX_DEPTH: u8 = 4;
pub const DEFAULT_HEURISTIC_MATERIAL_WEIGHT: i32 = 200;
pub const DEFAULT_HEURISTIC_ADVANCEMENT_WEIGHT: i32 = 5;
pub const DEFAULT_HEURISTIC_DEFENDED_WEIGHT: i32 = 5;
pub const DEFAULT_HEURISTIC_EDGE_PENALTY_WEIGHT: i32 = -2;
pub const DEFAULT_MCTS_MAX_ITERATIONS: u32 = 75000;
pub const DEFAULT_MCTS_MAX_TIME_MS: Option<u64> = None;
pub const DEFAULT_MCTS_EXPLORATION_CONSTANT: f64 = 1.41;
pub const DEFAULT_MCTS_USE_HEAVY_PLAYOUTS: bool = false;
pub const DEFAULT_MCTS_HEAVY_PLAYOUTS_EPSILON: f64 = 0.1;
pub const DEFAULT_MCTS_USE_RAVE: bool = false;
pub const DEFAULT_MCTS_RAVE_K: f64 = 1000.0;

pub fn default_max_depth() -> u8 {
    DEFAULT_MINIMAX_MAX_DEPTH
}

pub fn default_material() -> i32 {
    DEFAULT_HEURISTIC_MATERIAL_WEIGHT
}

pub fn default_advancement() -> i32 {
    DEFAULT_HEURISTIC_ADVANCEMENT_WEIGHT
}

pub fn default_defended() -> i32 {
    DEFAULT_HEURISTIC_DEFENDED_WEIGHT
}

pub fn default_edge_penalty() -> i32 {
    DEFAULT_HEURISTIC_EDGE_PENALTY_WEIGHT
}

pub fn default_max_iterations() -> u32 {
    DEFAULT_MCTS_MAX_ITERATIONS
}

pub fn default_max_time_ms() -> Option<u64> {
    DEFAULT_MCTS_MAX_TIME_MS
}

pub fn default_exploration_constant() -> f64 {
    DEFAULT_MCTS_EXPLORATION_CONSTANT
}

pub fn default_use_heavy_playouts() -> bool {
    DEFAULT_MCTS_USE_HEAVY_PLAYOUTS
}

pub fn default_heavy_playouts_epsilon() -> f64 {
    DEFAULT_MCTS_HEAVY_PLAYOUTS_EPSILON
}

pub fn default_use_rave() -> bool {
    DEFAULT_MCTS_USE_RAVE
}

pub fn default_rave_k() -> f64 {
    DEFAULT_MCTS_RAVE_K
}
