use crate::error::CNFSResult;
use crate::sync::UPCell;
use crate::vfs::dentry::remove_cache;
use crate::vfs::fs::FileSystem;
use crate::vfs::lookup_dentry;
use crate::vfs::path::Path;
use crate::CNFSError::{AlreadyMountedPath, NoMountedFilesystem};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use lazy_static::lazy_static;

pub struct Mount
{
    pub(crate) fs: Arc<dyn FileSystem>
}

lazy_static! {
    pub static ref MNTPOINT_TABLE: UPCell<BTreeMap<Path, Mount>> = unsafe{UPCell::new(BTreeMap::<Path, Mount>::new())};
}

/// Mount a filesystem at the given path.
pub fn mount(fs: Arc<dyn FileSystem>, mnt_point: Path) -> CNFSResult
{
    if mnt_point.to_string() != "/" {
        let dentry = lookup_dentry(&mnt_point)?;
        remove_cache(&dentry.path);
    }
    let table = MNTPOINT_TABLE.shared_access();
    let already_mounted = table.get(&mnt_point);
    if already_mounted.is_some() { return Err(AlreadyMountedPath); }
    drop(table);
    MNTPOINT_TABLE.exclusive_access().insert(mnt_point.clone(), Mount { fs });
    Ok(())
}
/// Mount the filesystem at the given path.
pub fn umount(mnt_point: Path) -> CNFSResult
{
    let mut table = MNTPOINT_TABLE.exclusive_access();
    let already_mounted = table.get(&mnt_point);
    if already_mounted.is_none() { return Err(NoMountedFilesystem); }
    table.remove(&mnt_point);
    Ok(())
}