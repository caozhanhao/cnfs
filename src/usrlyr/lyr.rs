use crate::error::CNFSError::PathNotFound;
use crate::error::CNFSResult;
use crate::usrlyr::{File, FileMode};
use crate::vfs::*;

/// Opens a file at path with the given mode.
pub fn open(path: &Path, mode: FileMode) -> CNFSResult<File>
{
    let dentry = lookup_dentry(path);
    match dentry
    {
        Ok(d) => Ok(File::new(d, mode)),
        Err(e) => {
            if e == PathNotFound && mode.contains(FileMode::write)
            {
                Ok(File::new(create_dentry(path, DentryType::File)?, mode))
            } else {
                Err(e)
            }
        }
    }
}

/// Close a file.
pub fn close(file: File)
{
    drop(file)
}

/// Create a directory at the given path.
pub fn create_directory(path: &Path) -> CNFSResult
{
    create_dentry(path, DentryType::Dir).map(|_| ())
}

/// Remove a file or directory at the given path.
pub fn remove(path: &Path) -> CNFSResult
{
    remove_dentry(path)
}

/// Check if the path points at an existing file or directory.
pub fn exists(path: &Path) -> CNFSResult<bool>
{
    let dentry = lookup_dentry(path);
    match dentry
    {
        Ok(_) => Ok(true),
        Err(e) => {
            if e == PathNotFound
            {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}