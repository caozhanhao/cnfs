use std::cell::{RefCell, UnsafeCell};
use std::io::{Read, Seek, Write};
use std::io::SeekFrom::Start;
use std::path::PathBuf;
use std::sync::Arc;
use cnfs::{FileSystem, Inode, InodeType, InodeRef, CNFSResult, CNFSError::*};

macro_rules! ecast {
    ($x: expr) => {$x.map_err(|e| FSInternal(format!("{e}")))};
}

#[allow(dead_code)]
pub struct FSWrapper
{
    fs: fatfs::FileSystem<std::fs::File>,
    root: UnsafeCell<Option<InodeRef>>
}

pub struct FileWrapper<'a>(RefCell<fatfs::File<'a, std::fs::File>>);

pub struct DirWrapper<'a>(fatfs::Dir<'a,std::fs::File>);

impl FSWrapper
{
    #[allow(dead_code)]
    pub fn new(img_path: PathBuf) -> Self
    {
        let img_file = std::fs::OpenOptions::new().read(true).write(true)
            .open(img_path).unwrap();
        let fatfs = fatfs::FileSystem::new(img_file, fatfs::FsOptions::new()).unwrap();
        Self {
            fs: fatfs,
            root: UnsafeCell::new(None)
        }
    }

    #[allow(dead_code)]
    pub fn init(&'static self)
    {
        unsafe {*self.root.get() = Some(Arc::new(DirWrapper(self.fs.root_dir())))}
    }
}

impl Inode for FileWrapper<'static>
{
    fn read(&self, offset: u64, buffer: &mut [u8]) -> CNFSResult<usize> {
        let mut file = self.0.borrow_mut();
        ecast!(file.seek(Start(offset)))?;
        ecast!(file.read(buffer))
    }

    fn write(&self, offset: u64, buffer: &[u8]) -> CNFSResult<usize> {
        let mut file = self.0.borrow_mut();
        ecast!(file.seek(Start(offset)))?;
        ecast!(file.write(buffer))
    }

    fn sync(&self) -> CNFSResult {
        ecast!(self.0.borrow_mut().flush())
    }
}

impl Inode for DirWrapper<'static>
{
    fn lookup(&self, name: &str) -> CNFSResult<InodeRef> {
        if let Ok(file) = self.0.open_file(name) {
            Ok(Arc::new(FileWrapper(RefCell::new(file))))
        } else if let Ok(dir) = self.0.open_dir(name){
            Ok(Arc::new(DirWrapper(dir)))
        } else {
            Err(PathNotFound.into())
        }
    }

    fn create(&self, name: &str, node_type: InodeType) -> CNFSResult<InodeRef> {
        match node_type
        {
            InodeType::Dir => {
                Ok(Arc::new(DirWrapper(ecast!(self.0.create_dir(name))?)))
            }
            InodeType::File => {
                Ok(Arc::new(FileWrapper(RefCell::new(ecast!(self.0.create_file(name))?))))
            }
        }
    }

    fn remove(&self, name: &str) -> CNFSResult {
        ecast!(self.0.remove(name))
    }
}

impl FileSystem for FSWrapper
{
    fn root_inode(&self) -> InodeRef {
        unsafe { (*self.root.get()).as_ref().unwrap().clone() }
    }
}

unsafe impl Sync for FSWrapper {}
unsafe impl Send for FSWrapper {}
unsafe impl<'a> Send for FileWrapper<'a> {}
unsafe impl<'a> Sync for FileWrapper<'a> {}
unsafe impl<'a> Send for DirWrapper<'a> {}
unsafe impl<'a> Sync for DirWrapper<'a> {}