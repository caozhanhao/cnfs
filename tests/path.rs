use cnfs::*;
#[test]
fn path_test()
{
    let p1 = Path::new("/home//caozhanhao/cnss");
    let p2 = Path::new("/home/caozhanhao/cnss/dev/../");
    let p3 = Path::new("/home");
    assert_eq!(p1, p2);
    assert!(p1.starts_with(&p3));
    assert_eq!(Path::new("/home/caozhanhao/..//./../"), Path::new("/"));
}
