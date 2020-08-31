
pub struct VfsPath<'a>(&'a str);

impl<'a> VfsPath<'a> {
    pub fn new(string: &'a str) -> Self {
        Self(string)
    }
    
    // TODO: do dot-files have extensions?
    // Windows says file: ".txt" is a text file
    pub fn extension(&self) -> Option<&'a str> {
        self.iter()
            .last()
            .and_then(|last| {
                last.split('.')
                    .skip(1)
                    .last()
            })
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &'a str> {
        self.0
            .split('/')
            .filter(|s| !s.is_empty())
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
}
