use crate::config::DEFAULT_FILE_BUFFER_SIZE;
use crate::error::CNFSResult;
use crate::vfs::Dentry;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;


bitflags! {
    /// Open File Mode
    pub struct FileMode: u32 {
        /// read
        const read = 0b00000001;
        /// write
        const write = 0b00000010;
    }
}

/// File Interface
pub struct File {
    pub(crate) dentry: Arc<Dentry>,
    pub(crate) mode: FileMode,
    pub(crate) buffer: Vec<u8>,
    pub(crate) max_buffer_size: usize,
    pub(crate) offset: u64,
}

impl File
{
    pub(super) fn new(dentry: Arc<Dentry>, mode: FileMode) -> Self
    {
        Self {
            dentry,
            mode,
            buffer: Vec::new(),
            max_buffer_size: DEFAULT_FILE_BUFFER_SIZE,
            offset: 0,
        }
    }

    /// Attempts to write an entire buffer into a file.
    pub fn write_all(&mut self, data: &[u8]) -> CNFSResult
    {
        let mut written: usize = 0;
        while written < data.len()
        {
            written += self.write(&data[written..])?;
        }
        Ok(())
    }

    /// Write a buffer into a file, returning how many bytes were written.
    pub fn write(&mut self, src: &[u8]) -> CNFSResult<usize>
    {
        if self.buffer.len() + src.len() > self.max_buffer_size
        {
            self.sync()?;
        }

        let mut nwritten: usize = 0;
        while src.len() - nwritten > self.max_buffer_size
        {
            assert!(self.buffer.is_empty());
            match self.dentry.inode_mut().write(self.offset, &src[nwritten..])
            {
                Ok(bytes) => {
                    nwritten += bytes;
                    self.offset += bytes as u64;
                }
                Err(err) => {
                    if nwritten != 0
                    {
                        return Ok(nwritten);
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        self.buffer.extend_from_slice(&src[nwritten..]);
        Ok(src.len())
    }

    /// Pull some bytes from this file into the specified buffer, returning how many bytes were read.
    pub fn read(&mut self, dest: &mut [u8]) -> CNFSResult<usize>
    {
        if dest.len() <= self.buffer.len()
        {
            dest.copy_from_slice(&self.buffer[..dest.len()]);
            Ok(dest.len())
        } else {
            dest[..self.buffer.len()].copy_from_slice(self.buffer.as_slice());

            let mut nread: usize = self.buffer.len();

            while nread < dest.len()
            {
                match self.dentry.inode_mut().read(self.offset, &mut dest[nread..])
                {
                    Ok(bytes) => {
                        if bytes > 0
                        {
                            nread += bytes;
                            self.offset += bytes as u64;
                        } else if bytes == 0
                        {
                            return Ok(nread);
                        }
                    }
                    Err(err) => {
                        if nread != 0
                        {
                            return Ok(nread);
                        } else {
                            return Err(err);
                        }
                    }
                }
            }
            Ok(dest.len())
        }
    }

    /// Read all bytes until EOF in this source, placing them into `dest`.
    pub fn read_to_end(&mut self, dest: &mut Vec<u8>) -> CNFSResult
    {
        dest.resize(256, 0);
        let mut nread = 0;
        loop {
            match self.read(&mut dest[nread..])
            {
                Ok(bytes) => {
                    nread += bytes;
                    if nread <= dest.len() {
                        dest.truncate(nread);
                        return Ok(());
                    } else {
                        dest.resize(nread + 512, 0);
                    }
                }
                Err(err) => { return Err(err); }
            }
        }
    }

    /// Seek to an offset
    pub fn seek(&mut self, new_offset: u64) -> CNFSResult
    {
        self.sync()?;
        self.offset = new_offset;
        Ok(())
    }

    /// Synchronize the data to filesystem.
    pub fn sync(&mut self) -> CNFSResult
    {
        if self.buffer.is_empty() { return Ok(()); }
        if self.mode.contains(FileMode::write)
        {
            let mut written: usize = 0;
            while written < self.buffer.len()
            {
                written += self.dentry.inode_mut().write(self.offset,
                                                         &self.buffer[written..])?;
                self.offset += written as u64;
            }
        }
        self.buffer.clear();
        Ok(())
    }
}

impl Drop for File
{
    fn drop(&mut self) {
        self.sync().expect("Failed to write to file.");
        if *self.dentry.exist.shared_access()
        {
            self.dentry.inode_mut().sync().expect("Failed to write to file.");
        }
    }
}