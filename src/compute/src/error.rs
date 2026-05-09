use thiserror::Error;

pub type ComputeResult<T> = std::result::Result<T, ComputeError>;

#[derive(Error, Debug)]
pub enum ComputeError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV Export failed: {0}")]
    Csv(#[from] csv::Error),

    #[error("JSON Serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}
