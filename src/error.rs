use std::fmt::Formatter;

use thiserror::Error;
use vfs::VfsError;

#[derive(Error, Debug)]
pub enum FileError {
    /// Generic error variant
    Generic {
        /// The generic error message
        message: String,
    },
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::Generic { message } => {
                write!(f, "{message}")
            }
        }
    }
}

impl From<VfsError> for FileError {
    fn from(err: VfsError) -> Self {
        FileError::Generic {
            message: err.to_string(),
        }
    }
}
