use alloc::vec::Vec;
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

/// Write a slice as the entire contents of a file.
///
/// This is a convenience function for using [open] and [File::write_all] with fewer imports.
pub fn write_all(path: &Path, data: &[u8]) -> CNFSResult
{
    open(path, FileMode::write)?.write_all(data)
}

/// Read a file into a slice.
///
/// This is a convenience function for using [open] and [File::read] with fewer imports.
pub fn read(path: &Path, data: &mut [u8]) -> CNFSResult<usize>
{
    open(path, FileMode::read)?.read(data)
}

/// Read the entire contents of a file into a slice.
///
/// This is a convenience function for using [open] and [File::read_to_end] with fewer imports.
pub fn read_to_end(path: &Path) -> CNFSResult<Vec<u8>>
{
    let mut buffer = Vec::new();
    open(path, FileMode::read)?.read_to_end(&mut buffer)?;
    Ok(buffer)
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