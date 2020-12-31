pub use anyhow::{Result, anyhow};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitTargetError {
    #[error("Watcher must be initialized with an absolute path!")]
    NonAbsPath,
    #[error("Watcher must be initialized with an existed path!")]
    NonExistent,
    #[error("Watcher must be initialized with a directory!")]
    NotADirectory,
}

#[derive(Error, Debug)]
pub enum WatchingError {
    #[error("No running loop in the watcher")]
    NoRunningLoop,
    #[error("Making duplicate loops in a single watcher!")]
    DuplicateLoop,
}
