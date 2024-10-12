use alloc::string::{String, ToString};
use core::fmt;

#[derive(PartialEq, Eq, Debug)]
/// Error definitions
pub enum CNFSError
{
    /// The path has already mounted a filesystem
    AlreadyMountedPath,
    /// Invalid path
    InvalidPath,
    /// Path not found
    PathNotFound,
    /// Already existed
    AlreadyExisted,
    /// There is no filesystem mounted on the path
    NoMountedFilesystem,
    /// The method has not been implemented
    NotImplemented,
    /// Internal filesystem error
    FSInternal(String),
    /// Unexpected error
    Unexpected,
}

impl CNFSError {
    /// Returns the error description.
    pub fn to_string(&self) -> String {
        use CNFSError::*;
        match self {
            AlreadyMountedPath => "The path has already mounted a filesystem".into(),
            InvalidPath => "Invalid path".into(),
            PathNotFound => "Path not found".into(),
            AlreadyExisted => "Already existed".into(),
            NoMountedFilesystem => "There is no filesystem mounted on the path".into(),
            NotImplemented => "Not implemented".into(),
            FSInternal(description) => "Internal Filesystem Error: ".to_string() + description,
            Unexpected => "Unexpected Error".into(),
        }
    }
}
impl fmt::Display for CNFSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CNFSError: {}.", self.to_string())
    }
}

/// Result definition
pub type CNFSResult<T = ()> = Result<T, CNFSError>;