use std::borrow::Borrow;
use std::iter::FromIterator;
use std::ops::Deref;

// For the purpose of our VFS, _all_ paths will be considered absolute. We _may_ implement relative paths at some point or another.

#[derive(Debug, Eq, PartialEq)]
pub struct VfsPath(str);

impl VfsPath {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Self {
        // Copy pasta from std::path::Path
        // Converts `&str` to `&VfsPath`, a very thin wrapper.
        unsafe { &*(s.as_ref() as *const str as *const VfsPath) }
    }

    // TODO: do dot-files have extensions?
    // Windows says file: ".txt" is a text file
    // Do we need to worry about extensions anyway?
    pub fn extension(&self) -> Option<&str> {
        self.iter()
            .last()
            .and_then(|last| last.split('.').skip(1).last())
    }

    pub fn canonicalize(&self) -> VfsPathBuf {
        // Removes duplicate '/'s, '.' path selector, and adds a leading '/'
        // The leading forward slash is because all paths are absolute
        self.iter().collect()
    }

    pub fn validate(&self) -> Result<&Self, ()> {
        // Currently just checks for ".." path selector as it is invalid
        // ".." is not allowed because all paths are absolute
        if self.iter().any(|s| s == "..") {
            Err(())
        } else {
            Ok(self)
        }
    }

    // Takes the first folder of the path and returns the rest of the path if there is any left
    pub fn take_head(&self) -> (&str, Option<&Self>) {
        // find where in the parent a substring is
        fn offset_in(substr: &str, parent: &str) -> usize {
            let substr_ptr = substr.as_ptr() as usize;
            let parent_ptr = parent.as_ptr() as usize;
            substr_ptr - parent_ptr + substr.len()
        }

        let trimmed = Self::new(self.0.trim_start_matches('/'));

        match trimmed.iter().next() {
            Some(take) => {
                let tail = &trimmed.0[offset_in(take, trimmed.to_str())..];
                let tail = Self::new(tail);

                // If the tail has no more valid path, return none
                let tail = tail.iter().next().map(|_| tail);

                (take, tail)
            }
            None => ("", None),
        }
    }

    #[inline(always)]
    pub fn to_str(&self) -> &str {
        &self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.split('/').filter(|s| !s.is_empty() && *s != ".")
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
        // Ensure there is always a leading '/', even when the iterator is empty
        let mut buf = String::from("/");
        let mut iter = iter.into_iter();

        // If statement here to check if any string adding has occurred in order
        // to know if a truncation if necessary or not
        if let Some(s) = iter.next() {
            buf.push_str(s);
            buf.push('/');
            for s in iter {
                buf.push_str(s);
                buf.push('/');
            }
            // Remove trailing '/'
            buf.truncate(buf.len() - 1);
        }

        Self(buf)
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
        iter.into_iter().map(String::as_str).collect()
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
        // TODO: fully specify the expected output of `extension()`
        // assert_eq!(VfsPath::new(".").extension(), Some(""));
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

        let path = VfsPath::new("././././");
        assert_eq!(path.canonicalize(), VfsPathBuf::from("/"));
    }

    #[test]
    fn path_take_head() {
        // Trailing slashes are trimmed as in this case, our filesystem is more simplified
        let path = VfsPath::new("/path/./file.txt/");
        let (head, tail) = path.take_head();
        assert_eq!(head, "path");
        assert_eq!(tail, Some(VfsPath::new("/./file.txt/")));

        let (head, tail) = tail.unwrap().take_head();
        assert_eq!(head, "file.txt");
        assert_eq!(tail, None);

        let path = VfsPath::new("/././");
        let (head, tail) = path.take_head();
        assert_eq!(head, "");
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
