use crate::config::DCACHE_SIZE;
use crate::error::CNFSError::{InvalidPath, PathNotFound};
use crate::error::CNFSResult;
use crate::sync::UPCell;
use crate::vfs::mnt::MNTPOINT_TABLE;
use crate::vfs::path::Path;
use crate::vfs::vinode::{VInode, VInodeRef, VInodeType};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::{Ref, RefMut};
use lazy_static::lazy_static;

pub struct Dentry
{
    pub path: Path,
    pub inode: VInodeRef,
    pub exist: UPCell<bool>,
}

impl Dentry
{
    pub fn new(path: Path, inode: VInodeRef) -> Self
    {
        Dentry { path, inode, exist: unsafe { UPCell::new(true) } }
    }

    pub fn inode(&self) -> Ref<'_, VInode>
    {
        self.inode.0.shared_access()
    }

    pub fn inode_mut(&self) -> RefMut<'_, VInode>
    {
        self.inode.0.exclusive_access()
    }
}

lazy_static! {
    pub static ref DCACHE: UPCell<BTreeMap<String, Vec<Arc<Dentry>>>> = unsafe{UPCell::new(BTreeMap::new())};
}

pub(crate) fn insert_dcache(dentry: Arc<Dentry>)
{
    let mut dcache = DCACHE.exclusive_access();
    while dcache.len() >= DCACHE_SIZE
    {
        dcache.pop_first();
    }
    let vec = dcache
        .entry(dentry.path[dentry.path.len() - 1].clone()).or_insert(Vec::new());
    if vec.iter().find(|x| { x.path == dentry.path }).is_none()
    {
        vec.push(dentry);
    }
}

pub(crate) fn remove_dcache(path: &Path)
{
    let mut dcache = DCACHE.exclusive_access();
    let cached = dcache.get_mut(path[path.len() - 1].as_str());
    if let Some(vec) = cached
    {
        vec.retain(|d| { d.path != *path });
    }
}

/// Look up a dentry from the given path
pub(crate) fn lookup_dentry(path: &Path) -> CNFSResult<Arc<Dentry>>
{
    if path.len() == 0 { return Err(InvalidPath.into()); }
    // first we look up the cache
    let dcache = DCACHE.shared_access();

    let mut curr = path.clone();
    let mut cached = dcache.get(curr[curr.len() - 1].as_str());
    let mut cached_dentry: Option<Arc<Dentry>> = None;
    'outer: loop
    {
        if let Some(mnt) = MNTPOINT_TABLE.shared_access().get(&curr)
        {
            cached_dentry = Some(Arc::new(Dentry::new(curr,
                                                      VInodeRef::new(mnt.fs.root_inode()))));
            break 'outer;
        } else if let Some(cached_vec) = cached
        {
            for c in cached_vec.iter()
            {
                if c.path == curr
                {
                    cached_dentry = Some(c.clone());
                    break 'outer;
                }
            }
            cached = None;
        } else if let Some(p) = curr.parent() {
            curr = p;
            cached = dcache.get(curr[curr.len() - 1].as_str());
        } else { break; }
    }
    drop(dcache);
    let mut search_parent: Option<Arc<Dentry>> = None;
    if cached_dentry.is_some() { search_parent = cached_dentry; } else {
        for mnt in MNTPOINT_TABLE.shared_access().iter()
        {
            if path.starts_with(mnt.0)
            {
                search_parent = Some(Arc::new(Dentry::new(mnt.0.clone(),
                                                          VInodeRef::new(mnt.1.fs.root_inode()))));
                insert_dcache(search_parent.clone().unwrap());
                break;
            }
        }
    }

    if search_parent.is_none() { return Err(PathNotFound); }
    let mut curr = search_parent.unwrap();
    loop {
        if *path == curr.path
        {
            return if *curr.exist.shared_access() { Ok(curr) } else { Err(PathNotFound) };
        }
        let name = path[curr.path.len()].to_string();
        match curr.clone().inode().lookup(name.as_str())
        {
            Ok(inode) => {
                curr = Arc::new(Dentry::new(path[..curr.path.len() + 1].into(),
                                            VInodeRef::new(inode)));
                insert_dcache(curr.clone());
            }
            Err(_) => { return Err(PathNotFound); }
        }
    }
}

pub type DentryType = VInodeType;

/// Create a dentry
pub(crate) fn create_dentry(path: &Path, inode_type: DentryType) -> CNFSResult<Arc<Dentry>>
{
    if path.len() == 0 { return Err(InvalidPath); }
    let i = lookup_dentry(&path.parent().unwrap())?.inode()
        .create(path[path.len() - 1].as_str(), inode_type)?;
    let dentry = Arc::new(Dentry::new(path.clone(),
                                      VInodeRef::new(i)));
    insert_dcache(dentry.clone());
    Ok(dentry)
}

/// Remove a dentry
pub(crate) fn remove_dentry(path: &Path) -> CNFSResult
{
    let dentry = lookup_dentry(&path)?;
    *dentry.exist.exclusive_access() = false;
    let parent_dentry = lookup_dentry(&path.parent().unwrap())?;
    remove_dcache(&path);
    parent_dentry.clone().inode().remove(path[path.len() - 1].as_str())
}