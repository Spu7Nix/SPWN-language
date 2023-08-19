use std::io;

use thiserror::Error;

use crate::doc::error::DocError;
use crate::gd::error::LevelError;
#[cfg(target_os = "windows")]
use crate::liveeditor::WebSocketError;
use crate::util::error::ErrorReport;

#[derive(Error, Debug)]
pub enum SpwnError {
    #[error("Failed to read SPWN file")]
    FailedToReadSpwnFile,

    #[error("Failed to read save file ({0})")]
    FailedToReadSaveFile(#[from] io::Error),

    #[error("Unsupported Operating System")]
    UsuppportedOS,

    #[error("`--lib`/`-l` argument expected a library name, got SPWN file instead")]
    DocExpectedLibrary,
    #[error("{0}")]
    DocError(#[from] DocError),

    #[cfg(target_os = "windows")]
    #[error("{0}")]
    WebSocketError(#[from] WebSocketError),

    #[error("{0}")]
    LevelError(#[from] LevelError),

    // any parser, compiler, vm, etc error
    #[error("{0}")]
    Error(#[from] ErrorReport),
}
