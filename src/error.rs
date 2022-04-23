use std::fmt::Formatter;

use thiserror::Error;
use vfs::VfsError;

#[derive(Error, Debug)]
pub enum FileError {
    /// Generic error variant
    Other {
        /// The generic error message
        message: String,
    },
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::Other { message } => {
                write!(f, "FileSystem error: {}", message)
            }
        }
    }
}

impl From<VfsError> for FileError {
    fn from(err: VfsError) -> Self {
        FileError::Other {
            message: err.to_string(),
        }
    }
}
