use std::borrow::Borrow;
use std::iter::FromIterator;
use std::ops::Deref;

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
    pub fn extension(&self) -> Option<&str> {
        self.iter()
            .last()
            .and_then(|last| {
                last.split('.')
                    .skip(1)
                    .last()
            })
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0
            .split('/')
            .filter(|s| !s.is_empty())
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
    pub fn new() -> Self { Default::default() }
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

impl From<&str> for VfsPathBuf {
    fn from(string: &str) -> Self {
        Self(string.to_owned())
    }
}
impl From<String> for VfsPathBuf {
    fn from(string: String) -> Self {
        Self(string)
    }
}
impl<'a, S: Deref<Target = &'a str>> FromIterator<S> for VfsPathBuf {
    fn from_iter<T> (iter: T) -> Self
        where T: IntoIterator<Item = S>
    {
        let mut buf = Self::new();
        for s in iter {
            buf.0.push('/');
            buf.0.push_str(&s);
        }
        buf
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn path_extension_basic() {
        assert_eq!(
            VfsPath::new("hello/world.txt").extension(),
            Some("txt")
        );
    }
    
    #[test]
    fn path_extension_is_empty() {
        assert_eq!(
            VfsPath::new("hello/world.").extension(),
            Some("")
        );
        assert_eq!(
            VfsPath::new(".").extension(),
            Some("")
        );
    }
    
    #[test]
    fn path_extension_is_none() {
        assert_eq!(
            VfsPath::new("hello/world").extension(),
            None
        );
        // See TODO on VfsPath::extension
        // let path = VfsPath::new("hello/.world");
        // assert_eq!(path.extension(), None);
    }
    
    #[test]
    fn iterator_splits_path_correctly() {
        let path = VfsPath::new("//really/long///path.rs");
        let collect = path.iter().collect::<Vec<&str>>();
        assert_eq!(collect, vec!["really", "long", "path.rs"])
    }
    
    #[test]
    fn collect_into_vfs_path_buf() {
        let vec = vec!["hello", "world", "file.txt"];
        let path: VfsPathBuf = vec.iter().collect();
        assert_eq!(path, VfsPathBuf::from("/hello/world/file.txt"))
    }
}
