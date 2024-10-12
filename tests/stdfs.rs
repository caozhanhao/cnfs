use cnfs::{create_directory, exists, mount, open, remove, umount, CNFSResult, FileMode, Path};
use std::env::current_dir;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

mod adapter;
use adapter::stdfs::*;

#[test]
fn stdfs_test() -> CNFSResult
{
    let fs = Arc::new(FSWrapper::new(current_dir().unwrap()));

    let real_curr = current_dir().unwrap();
    let real_curr_str = real_curr.to_str().unwrap().to_owned();

    // mount
    mount(fs, Path::new("/"))?;
    let test_dir = Path::new("/test_directory");
    let test_file = Path::new("/test_file");
    let real_dir_str: String = real_curr_str.clone() + &test_dir.to_string();
    let real_file_str: String = real_curr_str.clone() + &test_file.to_string();
    let std_dir_path = std::path::Path::new(&real_dir_str);
    let std_file_path = std::path::Path::new(&real_file_str);

    // Directory Test
    if exists(&test_dir)?
    {
        remove(&test_dir)?;
    }
    assert!(!exists(&test_dir)? && !std_dir_path.exists());
    create_directory(&test_dir)?;
    assert!(exists(&test_dir)? && std_dir_path.exists());
    remove(&test_dir)?;
    assert!(!exists(&test_dir)? && !std_dir_path.exists());

    // File Test
    if exists(&test_file)?
    {
        remove(&test_file)?;
    }
    assert!(!exists(&test_file)? && !std_file_path.exists());
    let mut file = open(&test_file, FileMode::write)?;

    let data = "cnss{th1s_i5_my_vfs_t3st}";
    let mut dest = vec![0_u8; data.len()];
    file.write_all(data.as_bytes())?;
    file.sync()?;

    file.seek(0)?;
    assert_eq!(file.read(dest.as_mut_slice())?, dest.len());
    assert_eq!(dest, data.as_bytes());

    dest.fill(0);
    let ret = File::open(std_file_path).unwrap().read(dest.as_mut_slice());
    assert_eq!(ret.unwrap(), dest.len());
    assert_eq!(dest, data.as_bytes());

    file.seek(0)?;
    for _ in 0..10000
    {
        file.write_all(data.as_bytes())?;
    }

    file.sync()?;
    file.seek(0)?;
    let mut stdfile = File::open(std_file_path).unwrap();
    for _ in 0..10000
    {
        dest.fill(0);
        assert_eq!(file.read(dest.as_mut_slice())?, dest.len());
        assert_eq!(dest, data.as_bytes());
        dest.fill(0);
        assert_eq!(stdfile.read(dest.as_mut_slice()).unwrap(), dest.len());
        assert_eq!(dest, data.as_bytes());
    }

    assert!(exists(&test_file)? && std_file_path.exists());
    remove(&test_file)?;
    assert!(!exists(&test_file)? && !std_file_path.exists());

    umount(Path::new("/"))?;
    Ok(())
}