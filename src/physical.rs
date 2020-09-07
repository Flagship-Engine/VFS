use crate::path::VfsPath;
use crate::Mount;
use std::fs::File;
use std::io::{ErrorKind, Read, Result};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct PhysicalMount {
    folder: PathBuf,
}
impl PhysicalMount {
    pub fn new(folder: &Path) -> Result<Self> {
        if folder.is_dir() {
            Ok(Self {
                folder: folder.to_owned(),
            })
        } else {
            Err(ErrorKind::InvalidInput.into())
        }
    }
}

impl Mount for PhysicalMount {
    fn open(&self, path: &VfsPath) -> Result<Box<dyn Read>> {
        let joined = self.folder.join(path.to_path()?);
        Ok(Box::new(File::open(joined)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::VfsPath;
    use crate::VFS;

    #[test]
    fn physical_mount() {
        let mut vfs = VFS::new();
        vfs.mount_physical(VfsPath::new("/random/path"), Path::new("./src"))
            .unwrap();

        let mut vfs_file = vfs.open(VfsPath::new("/random/path/lib.rs")).unwrap();
        let mut vfs_contents = String::new();
        vfs_file.read_to_string(&mut vfs_contents).unwrap();

        let mut real_file = std::fs::File::open("./src/lib.rs").unwrap();
        let mut real_contents = String::new();
        real_file.read_to_string(&mut real_contents).unwrap();

        assert_eq!(vfs_contents, real_contents);
    }
}
