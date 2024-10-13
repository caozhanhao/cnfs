use cnfs::{CNFSError::*, CNFSResult, FileSystem, Inode, InodeRef, InodeType};
use std::fs::{create_dir, read_dir, remove_dir, remove_file, File, OpenOptions};
use std::io::SeekFrom::Start;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::sync::Arc;

macro_rules! ecast {
    ($x: expr) => {$x.map_err(|e| FSInternal(e.to_string()))};
}

pub struct FSWrapper(PathBuf);

pub struct FileWrapper(PathBuf);

pub struct DirWrapper(PathBuf);

impl FSWrapper
{
    #[allow(dead_code)]
    pub fn new(dir: PathBuf) -> Self
    {
        Self(dir)
    }
}

impl FileWrapper
{
    pub fn new(path: PathBuf) -> Self
    {
        Self(path)
    }
}

impl DirWrapper
{
    pub fn new(path: PathBuf) -> Self
    {
        Self(path)
    }
}

impl Inode for FileWrapper
{
    fn read(&self, offset: u64, buffer: &mut [u8]) -> CNFSResult<usize> {
        let mut file = ecast!(OpenOptions::new().read(true).open(&self.0))?;
        ecast!(file.seek(Start(offset)))?;
        ecast!(file.read(buffer))
    }

    fn write(&self, offset: u64, buffer: &[u8]) -> CNFSResult<usize> {
        let mut file = ecast!(OpenOptions::new().write(true).open(&self.0))?;
        ecast!(file.seek(Start(offset)))?;
        ecast!(file.write(buffer))
    }

    fn sync(&self) -> CNFSResult {
        Ok(())
    }
}

impl Inode for DirWrapper
{
    fn lookup(&self, name: &str) -> CNFSResult<InodeRef> {
        for entry in ecast!(read_dir(&self.0))? {
            let path = ecast!(entry)?.path();
            if path.file_name().unwrap() == name
            {
                if path.is_file() {
                    return Ok(Arc::new(FileWrapper::new(path)));
                } else {
                    return Ok(Arc::new(DirWrapper::new(path)));
                }
            }
        }
        Err(PathNotFound.into())
    }

    fn create(&self, name: &str, node_type: InodeType) -> CNFSResult<InodeRef> {
        let mut target = self.0.clone().into_os_string();
        target.push("/");
        target.push(name);
        match node_type
        {
            InodeType::Dir => {
                ecast!(create_dir(target))?;
            }
            InodeType::File => {
                ecast!(File::create(target))?;
            }
        }
        self.lookup(name)
    }

    fn remove(&self, name: &str) -> CNFSResult {
        for e in ecast!(read_dir(&self.0))? {
            let entry = ecast!(e)?;
            let path = entry.path();
            if path.file_name().unwrap() == name
            {
                if path.is_file() {
                    return ecast!(remove_file(path));
                } else {
                    return ecast!(remove_dir(path));
                }
            }
        }
        Err(PathNotFound.into())
    }
}

impl FileSystem for FSWrapper
{
    fn root_inode(&self) -> InodeRef {
        Arc::new(DirWrapper(self.0.clone()))
    }
}

unsafe impl Sync for FileWrapper {}
unsafe impl Send for FileWrapper {}

unsafe impl Sync for DirWrapper {}
unsafe impl Send for DirWrapper {}

unsafe impl Sync for FSWrapper {}
unsafe impl Send for FSWrapper {}
