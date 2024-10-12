use crate::error::CNFSResult;
use crate::CNFSError::NotImplemented;
use alloc::sync::Arc;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// The Inode type
pub enum InodeType {
    /// Directory
    Dir,
    /// Regular file
    File,
}

/// Trait for inode
pub trait Inode: Send + Sync {
    /// Read data from file to buffer at a given offset
    fn read(&self, _offset: u64, _buffer: &mut [u8]) -> CNFSResult<usize>
    {
        Err(NotImplemented)
    }

    /// Write data from buffer to file at a given offset
    fn write(&self, _offset: u64, _buffer: &[u8]) -> CNFSResult<usize>
    {
        Err(NotImplemented)
    }

    /// Synchronize the data to filesystem.
    fn sync(&self) -> CNFSResult
    {
        Err(NotImplemented)
    }

    /// Lookup a node with a given name
    fn lookup(&self, _name: &str) -> CNFSResult<InodeRef>
    {
        Err(NotImplemented)
    }

    /// Create a new node with a given name
    fn create(&self, _name: &str, _node_type: InodeType) -> CNFSResult<InodeRef>
    {
        Err(NotImplemented)
    }

    /// Remove a node with a given name
    fn remove(&self, _name: &str) -> CNFSResult
    {
        Err(NotImplemented)
    }
}

/// Inode reference (Arc<dyn Inode>)
pub type InodeRef = Arc<dyn Inode>;

/// Trait for filesystem
pub trait FileSystem: Send + Sync {
    /// Returns the root directory of the filesystem
    fn root_inode(&self) -> InodeRef;
}