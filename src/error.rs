use thiserror::Error;
#[derive(Error, Debug)]
pub enum DiffError {
    #[error("io error")]
    IO(#[from] std::io::Error),
    #[error("serialization error")]
    SE(#[from] serde_json::Error),
}
