use std::io;

use thiserror::Error;

use crate::util::error::ErrorReport;

pub type DocResult<T> = Result<T, DocError>;

#[derive(Error, Debug)]
pub enum DocError {
    #[error("Failed to read SPWN file")]
    FailedToReadSpwnFile,

    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),

    #[error("{0}")]
    Error(#[from] ErrorReport),
}
