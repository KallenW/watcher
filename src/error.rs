pub use anyhow::{Result, anyhow};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitDirWatcherError {
    #[error("Watcher must be initialized with an absolute path!")]
    NonAbsPath,
    #[error("Watcher must be initialized with an existed path!")]
    InExistence,
    #[error("Watcher must be initialized with a directory!")]
    NotADirectory,
}
