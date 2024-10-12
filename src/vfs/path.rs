use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::ops::{Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use core::slice::Iter;

#[derive(Debug, Eq, Ord, Clone)]
/// Path struct
pub struct Path
{
    names: Vec<String>,
}

impl Path
{
    /// New a Path from a path string
    ///
    /// ```rust
    ///  use cnfs::Path;
    ///  let p1 = Path::new("/home//caozhanhao/cnss");
    ///  let p2 = Path::new("/home/caozhanhao/cnss/dev/../");
    ///  assert_eq!(p1, p2);
    /// ```
    ///
    pub fn new(path_str: &str) -> Self
    {
        // Currently we only support absolute path
        assert!(path_str.starts_with('/'));
        let mut path = Self { names: Vec::<String>::new() };
        for part in path_str.split('/') {
            match part {
                "" | "." => continue,
                ".." => {
                    match path.names.last() {
                        Some(last) if last == "/" => {}
                        None => {}
                        _ => {
                            path.names.pop();
                        }
                    }
                }
                _ => {
                    if path.names.is_empty() {
                        path.names.push("/".into());
                    }
                    path.names.push(part.into());
                }
            }
        }
        if path.names.is_empty() {
            path.names.push("/".into());
        }
        path
    }

    /// Returns the parent path
    pub fn parent(&self) -> Option<Path>
    {
        if self.names.len() < 2 { return None; }
        let mut ret = self.clone();
        ret.names.pop();
        Some(ret)
    }

    /// Check if the path is starts with given path
    pub fn starts_with(&self, item: &Self) -> bool
    {
        self.names.starts_with(&item.names)
    }

    /// Returns the length of the path.
    pub fn len(&self) -> usize
    {
        self.names.len()
    }

    /// Iterator
    pub fn iter(&self) -> Iter<'_, String>
    {
        self.names.iter()
    }

    /// Convert the path to string.
    pub fn to_string(&self) -> String
    {
        let mut ret = self.names.join("/");
        if ret.len() != 1
        {
            assert_eq!(ret.remove(0), '/');
        }
        ret
    }
}

impl Index<usize> for Path {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        &self.names[index]
    }
}

impl Index<Range<usize>> for Path {
    type Output = [String];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.names[index.start..index.end]
    }
}

impl Index<RangeFull> for Path {
    type Output = [String];

    fn index(&self, _index: RangeFull) -> &Self::Output {
        &self.names
    }
}

impl Index<RangeFrom<usize>> for Path {
    type Output = [String];

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        &self.names[index.start..]
    }
}

impl Index<RangeTo<usize>> for Path {
    type Output = [String];

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self.names[..index.end]
    }
}

impl Index<RangeToInclusive<usize>> for Path {
    type Output = [String];

    fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
        &self.names[..index.end]
    }
}

impl Index<RangeInclusive<usize>> for Path {
    type Output = [String];

    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        &self.names[*index.start()..*index.end()]
    }
}

impl From<&[String]> for Path
{
    fn from(value: &[String]) -> Self {
        Self {
            names: value.to_vec()
        }
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.names.eq(&other.names)
    }
}

impl PartialOrd for Path
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.names.partial_cmp(&other.names)
    }
}