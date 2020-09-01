use std::borrow::Borrow;
use std::iter::FromIterator;
use std::ops::Deref;

// For the purpose of our VFS, _all_ paths will be considered absolute. We _may_ imeplement relative paths at some point or another.

#[derive(Debug, Eq, PartialEq)]
pub struct VfsPath(str);

impl VfsPath {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Self {
        // Copy pasta from std::path::Path
        // Converts `&str` to `&VfsPath`, a very thin wrapper.
        unsafe { &*(s.as_ref() as *const str as *const VfsPath) }
    }

    // TODO: do dotfiles have extensions?
    // Windows says file: ".txt" is a text file
    // Do we need to worry about extensions anyway?
    pub fn extension(&self) -> Option<&str> {
        self.iter()
            .last()
            .and_then(|last| last.split('.').skip(1).last())
    }

    pub fn canonicalize(&self) -> VfsPathBuf {
        // Removes duplicate '/'s, '.' path selector, and adds a leading '/'
        // The leading forward slash is becuse all paths are
        self.iter().filter(|s| *s != ".").collect()
    }

    pub fn validate(&self) -> Result<&Self, ()> {
        // Currently just checks for ".." path selector as it is invalid
        // ".." is not allowed becuase all paths are absolute
        if self.iter().any(|s| s == "..") {
            Err(())
        } else {
            Ok(self)
        }
    }

    // Takes the first folder of the path and resturns the rest of the path if there is any left
    pub fn take_head(&self) -> (&str, Option<&VfsPath>) {
        let trimmed = Self::new(self.0.trim_start_matches('/'));
        let take = trimmed.iter().next().unwrap();
        let (_, tail) = trimmed.0.split_at(take.len());
        let tail = if tail.is_empty() {
            None
        } else {
            Some(VfsPath::new(tail))
        };
        (take, tail)
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.split('/').filter(|s| !s.is_empty())
    }
}

impl ToOwned for VfsPath {
    type Owned = VfsPathBuf;
    fn to_owned(&self) -> Self::Owned {
        VfsPathBuf(self.0.to_owned())
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct VfsPathBuf(String);

impl VfsPathBuf {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Deref for VfsPathBuf {
    type Target = VfsPath;
    fn deref(&self) -> &Self::Target {
        VfsPath::new(&self.0)
    }
}

impl Borrow<VfsPath> for VfsPathBuf {
    fn borrow(&self) -> &VfsPath {
        self.deref()
    }
}

impl<'a> From<&'a str> for VfsPathBuf {
    fn from(string: &'a str) -> Self {
        Self(string.to_owned())
    }
}
impl From<String> for VfsPathBuf {
    fn from(string: String) -> Self {
        Self(string)
    }
}

impl<'a> FromIterator<&'a str> for VfsPathBuf {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a str>,
    {
        let mut buf = Self::new();
        for s in iter {
            buf.0.push('/');
            buf.0.push_str(s);
        }
        buf
    }
}
impl<'a> FromIterator<&'a &'a str> for VfsPathBuf {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a &'a str>,
    {
        iter.into_iter().cloned().collect()
    }
}
impl<'a> FromIterator<&'a String> for VfsPathBuf {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a String>,
    {
        iter.into_iter().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_extension_basic() {
        assert_eq!(VfsPath::new("hello/world.txt").extension(), Some("txt"));
    }

    #[test]
    fn path_extension_is_empty() {
        assert_eq!(VfsPath::new("hello/world.").extension(), Some(""));
        assert_eq!(VfsPath::new(".").extension(), Some(""));
    }

    #[test]
    fn path_extension_is_none() {
        assert_eq!(VfsPath::new("hello/world").extension(), None);
        // See TODO on VfsPath::extension
        // let path = VfsPath::new("hello/.world");
        // assert_eq!(path.extension(), None);
    }

    #[test]
    fn path_canonicalize() {
        let path = VfsPath::new("./hello/.//world/././file.txt");
        assert_eq!(
            path.canonicalize(),
            VfsPathBuf::from("/hello/world/file.txt")
        );
    }

    #[test]
    fn path_take_head() {
        let path = VfsPath::new("/path/file.txt");
        let (head, tail) = path.take_head();
        assert_eq!(head, "path");
        assert_eq!(tail, Some(VfsPath::new("/file.txt")));

        let (head, tail) = tail.unwrap().take_head();
        assert_eq!(head, "file.txt");
        assert_eq!(tail, None);
    }

    #[test]
    fn iter_splits_path_correctly() {
        let path = VfsPath::new("//really/long///path.rs");
        let collect: Vec<&str> = path.iter().collect();
        assert_eq!(collect, vec!["really", "long", "path.rs"])
    }

    #[test]
    fn collect_into_vfs_path_buf() {
        let vec = vec!["hello", "world", "file.txt"];
        let path: VfsPathBuf = vec.iter().collect();
        assert_eq!(path.deref(), VfsPath::new("/hello/world/file.txt"))
    }

    #[test]
    fn path_to_and_from_iter() {
        let path = VfsPathBuf::from("/this/is/a/path.txt");
        assert_eq!(path, path.iter().collect());
    }
}
