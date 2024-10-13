use lazy_static::lazy_static;
use std::env::current_dir;
use std::sync::Arc;

use cnfs::{create_directory, exists, mount, open, remove, CNFSResult, FileMode, Path};

mod adapter;
use adapter::*;

fn test_dir(dir: &Path) -> CNFSResult
{
    if exists(&dir)?
    {
        remove(&dir)?;
    }
    assert!(!exists(&dir)?);
    create_directory(&dir)?;
    assert!(exists(&dir)?);
    remove(&dir)?;
    assert!(!exists(&dir)?);
    Ok(())
}

fn test_file(path: &Path) -> CNFSResult
{
    if exists(&path)?
    {
        remove(&path)?;
    }
    assert!(!exists(&path)?);
    let mut file = open(&path, FileMode::write)?;

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

    assert!(exists(&path)?);
    remove(&path)?;
    assert!(!exists(&path)?);
    Ok(())
}

lazy_static! {
    pub static ref fat_fs: Arc<fatfs::FSWrapper>
    = Arc::new(fatfs::FSWrapper::new("tests/resources/fat_2.img".into()));
}

#[test]
fn mixed_test() -> CNFSResult
{
    fat_fs.init();
    let std_fs = Arc::new(stdfs::FSWrapper::new(current_dir().unwrap()));

    mount(std_fs.clone(), Path::new("/"))?;
    let fat_mnt = Path::new("/mnt");
    if !exists(&fat_mnt)?
    {
        create_directory(&fat_mnt)?;
    }
    mount(fat_fs.clone(), Path::new("/mnt"))?;

    let dir = Path::new("/test_directory");
    let file = Path::new("/test_file");

    let dir1 = Path::new("/mnt/test_directory");
    let file1 = Path::new("/mnt/test_file");

    // Directory Test
    test_dir(&dir)?;
    test_dir(&dir1)?;

    // File Test
    test_file(&file)?;
    test_file(&file1)?;

    remove(&fat_mnt)?;

    Ok(())
}
