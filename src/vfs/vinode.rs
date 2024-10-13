use crate::config::{OSINODE_PAGE_ENTRY_SIZE, OSINODE_PAGE_SIZE};
use crate::sync::UPCell;
use crate::vfs::fs::InodeRef;
use crate::{CNFSResult, InodeType};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::min;
use core::option::Option;

struct Page
{
    dirty: bool,
    data: Vec<u8>,
}

impl Page {
    fn new() -> Self
    {
        Self {
            dirty: false,
            data: Vec::with_capacity(OSINODE_PAGE_SIZE),
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
struct Offset(u64);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
struct PageNumber(u64);

impl Offset
{
    fn floor(&self) -> PageNumber
    {
        PageNumber(self.0 / OSINODE_PAGE_SIZE as u64)
    }

    fn page_offset(&self) -> usize
    {
        self.0 as usize % OSINODE_PAGE_SIZE
    }
}

impl PageNumber
{
    fn next(&mut self)
    {
        self.0 += 1;
    }

    fn offset(&self) -> u64
    {
        self.0 * OSINODE_PAGE_SIZE as u64
    }
}

pub(crate) struct VInode
{
    fs_inode: InodeRef,
    cache: BTreeMap<PageNumber, Page>,
}

pub type VInodeType = InodeType;
pub(crate) struct VInodeRef(pub(crate) Arc<UPCell<VInode>>);
impl VInodeRef {
    pub(crate) fn new(fs_inode: InodeRef) -> Self
    {
        Self(Arc::new(unsafe { UPCell::new(VInode::new(fs_inode)) }))
    }
}

#[allow(unused)]
impl VInode
{
    pub fn new(fs_inode: InodeRef) -> Self
    {
        Self {
            fs_inode,
            cache: BTreeMap::new(),
        }
    }

    pub fn read(&mut self, offset: u64, buffer: &mut [u8]) -> CNFSResult<usize>
    {
        let off = Offset(offset);
        let mut curr_page = off.floor();
        let mut curr_page_offset = off.page_offset();
        let mut nread: usize = 0;

        loop
        {
            let page = match self.load_page(curr_page)
            {
                Ok(p) =>
                    {
                        if p.data.len() == 0
                        {
                            self.cache.remove(&curr_page);
                            return Ok(nread);
                        } else {
                            p
                        }
                    }
                Err(err) => {
                    self.cache.remove(&curr_page);
                    return if nread != 0 { Ok(nread) } else { Err(err) };
                }
            };

            if buffer.len() - nread > page.data.len() - curr_page_offset
            {
                let end = page.data.len() - curr_page_offset + nread;
                buffer[nread..end].copy_from_slice(&page.data[curr_page_offset..]);
                nread += page.data.len() - curr_page_offset;
            } else {
                let end = buffer.len() - nread + curr_page_offset;
                buffer[nread..].copy_from_slice(&page.data[curr_page_offset..end]);
                nread += buffer.len() - nread;
            }
            if nread < buffer.len() {
                curr_page.next();
                curr_page_offset = 0;
            } else { break; }
        }

        Ok(buffer.iter().len())
    }

    pub fn write(&mut self, offset: u64, buffer: &[u8]) -> CNFSResult<usize>
    {
        let off = Offset(offset);
        let mut curr_page = off.floor();
        let mut curr_page_offset = off.page_offset();
        let mut nwritten: usize = 0;
        loop
        {
            let page = match self.load_page(curr_page)
            {
                Ok(p) =>
                    {
                        let min_page_size = min(OSINODE_PAGE_SIZE, buffer.len() - nwritten);
                        if p.data.len() < min_page_size
                        {
                            p.data.resize(min_page_size, 0);
                        }
                        p.dirty = true;
                        p
                    }
                Err(err) => {
                    self.cache.remove(&curr_page);
                    return if nwritten != 0 { Ok(nwritten) } else { Err(err) };
                }
            };

            if buffer.len() - nwritten > page.data.len() - curr_page_offset
            {
                let end = page.data.len() - curr_page_offset + nwritten;
                page.data[curr_page_offset..].copy_from_slice(&buffer[nwritten..end]);
                nwritten += page.data.len() - curr_page_offset;
            } else {
                let end = buffer.len() - nwritten + curr_page_offset;
                page.data[curr_page_offset..end].copy_from_slice(&buffer[nwritten..]);
                nwritten += buffer.len() - nwritten;
            }
            if nwritten < buffer.len() {
                curr_page.next();
                curr_page_offset = 0;
            } else { break; }
        }

        Ok(buffer.iter().len())
    }

    pub fn sync(&mut self) -> CNFSResult
    {
        for page in self.cache.iter_mut()
        {
            if page.1.dirty
            {
                self.fs_inode.write(page.0.offset(), page.1.data.as_slice())?;
                page.1.dirty = false;
            }
        }
        self.fs_inode.sync()
    }

    pub fn lookup(&self, name: &str) -> CNFSResult<InodeRef>
    {
        self.fs_inode.lookup(name)
    }

    pub fn create(&self, name: &str, inode_type: VInodeType) -> CNFSResult<InodeRef>
    {
        self.fs_inode.create(name, inode_type)
    }

    pub fn remove(&self, name: &str) -> CNFSResult
    {
        self.fs_inode.remove(name)
    }

    fn load_page(&mut self, page_number: PageNumber) -> CNFSResult<&mut Page> {
        if self.cache.len() >= OSINODE_PAGE_ENTRY_SIZE
        {
            let mut target: Option<PageNumber> = None;
            for i in self.cache.iter()
            {
                if *i.0 != page_number
                {
                    if target.is_none()
                    {
                        target = Some(*i.0);
                    }

                    if !i.1.dirty
                    {
                        target = Some(*i.0);
                        break;
                    }
                }
            }
            let removed = self.cache.remove(&target.unwrap()).unwrap();
            if removed.dirty
            {
                self.fs_inode.write(target.unwrap().offset(), removed.data.as_slice())
                    .expect("Failed to write to file.");
            }
        }

        let page = self.cache.entry(page_number).or_insert(Page::new());
        if page.data.is_empty()
        {
            page.data.resize(OSINODE_PAGE_SIZE, 0);
            let bytes = self.fs_inode.read(page_number.offset(), page.data.as_mut_slice())?;
            page.data.truncate(bytes);
        }
        Ok(page)
    }
}

impl Drop for VInode
{
    fn drop(&mut self) {
        let _ = self.sync();
    }
}