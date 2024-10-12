use cnfs::{create_directory, exists, mount, open, remove, umount,
           CNFSResult, FileMode, Path};
use lazy_static::lazy_static;
use std::sync::Arc;

mod adapter;
use adapter::fatfs::*;

lazy_static! {
    pub static ref fs: Arc<FSWrapper> = Arc::new(FSWrapper::new("tests/resources/fat_1.img".into()));
}

#[test]
fn fatfs_test() -> CNFSResult
{
    fs.init();
    // mount
    mount(fs.clone(), Path::new("/"))?;
    let test_dir = Path::new("/test_directory");
    let test_file = Path::new("/test_file");

    // Directory Test
    if exists(&test_dir)?
    {
        remove(&test_dir)?;
    }
    assert!(!exists(&test_dir)?);
    create_directory(&test_dir)?;
    assert!(exists(&test_dir)?);
    remove(&test_dir)?;
    assert!(!exists(&test_dir)?);

    // File Test
    if exists(&test_file)?
    {
        remove(&test_file)?;
    }
    assert!(!exists(&test_file)?);
    let mut file = open(&test_file, FileMode::write)?;

    let data = "cnss{th1s_i5_my_vfs_t3st}";
    let mut dest = vec![0_u8; data.len()];
    file.write_all(data.as_bytes())?;
    file.sync()?;

    file.seek(0)?;
    assert_eq!(file.read(dest.as_mut_slice())?, dest.len());
    assert_eq!(dest, data.as_bytes());

    file.seek(0)?;
    for _ in 0..10000
    {
        file.write_all(data.as_bytes())?;
    }

    file.sync()?;
    file.seek(0)?;
    for _ in 0..10000
    {
        dest.fill(0);
        assert_eq!(file.read(dest.as_mut_slice())?, dest.len());
        assert_eq!(dest, data.as_bytes());
    }

    assert!(exists(&test_file)?);
    remove(&test_file)?;
    assert!(!exists(&test_file)?);

    umount(Path::new("/"))?;
    Ok(())
}