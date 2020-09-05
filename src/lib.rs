// We use io::Result as it's the most fitting for our purpose and no reason to reinvent the wheel
use std::io::{ErrorKind, Result};

pub mod path;
use path::VfsPath;

#[derive(Debug, Default)]
pub struct VFS {
    root: VirtualDir,
}
impl VFS {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn mount(&mut self, target: &VfsPath, mount: Box<dyn Mount>) {
        if target.iter().next().is_none() {
            // If there is no path, then root was pointed to, i.e. "/"
            self.root.nodes.push(Node::create_mount("", mount));
        } else {
            // If root was not selected, then mount within the directory structure
            self.root.mount(target, mount);
        }
    }
}

// VFS is a mount because it implements all the same functions anyway,
// and to create the possibility of recursive structures
impl Mount for VFS {
    fn open(&self, path: &VfsPath) -> Result<String> {
        self.root.open(path)
    }
}

#[derive(Debug, Default)]
struct VirtualDir {
    nodes: Vec<Node>,
}
impl VirtualDir {
    fn mount(&mut self, target: &VfsPath, mount: Box<dyn Mount>) {
        let (head, tail) = target.take_head();
        match tail {
            // If tail is none, then we are at the end of a path
            None => {
                // Mount here and we're done :)
                self.nodes.push(Node::create_mount(head, mount));
            }

            // There is still more path to resolve, continue...
            Some(tail) => {
                let maybe_dir = self.nodes.iter_mut().find(|n| n.is_dir() && n.name == head);

                match maybe_dir {
                    // The directory has already been created, continue traversing from here...
                    Some(dir) => dir.mount(tail, mount),

                    // The directory has not been made, create it and continue traversing...
                    None => {
                        let mut new_dir = Node::create_dir(head);
                        new_dir.mount(tail, mount);
                        self.nodes.push(new_dir);
                    }
                }
            }
        }
    }

    fn open(&self, path: &VfsPath) -> Result<String> {
        let mut file = Err(ErrorKind::NotFound.into());

        // If tail is none, then we are addressing a directory
        // TODO: possibly handle directory operations?
        if let (head, Some(mut tail)) = path.take_head() {
            // Find all nodes with that match the first path selector
            // Using `rev` because path resolution is FILO
            // Searching for names of empty paths to handle for the case of mounting at root
            let find = self
                .nodes
                .iter()
                .rev()
                .filter(|n| n.name == head || n.name.is_empty());

            for node in find {
                // If we are on an empty name, i.e. "/" path, use the full path, not trimmed path
                if node.name.is_empty() {
                    tail = path;
                }

                file = match &node.kind {
                    NodeKind::Mount(mount) => mount.open(tail),
                    NodeKind::Dir(dir) => dir.open(tail),
                };

                // If a file is found, return it and be done
                // If not, continue iterating. This allows for multiple mounts of the same name
                if file.is_ok() {
                    return file;
                }

                // TODO: determine if we should handle certain types of errors instead of just continuing
            }
        }
        // Return that last error held by file
        file
    }
}

#[derive(Debug)]
struct Node {
    name: String,
    kind: NodeKind,
}
#[derive(Debug)]
enum NodeKind {
    Dir(VirtualDir),
    Mount(Box<dyn Mount>),
}

impl Node {
    #[inline]
    fn is_dir(&self) -> bool {
        match self.kind {
            NodeKind::Dir(_) => true,
            NodeKind::Mount(_) => false,
        }
    }
    #[inline]
    fn is_mount(&self) -> bool {
        !self.is_dir()
    }

    fn mount(&mut self, target: &VfsPath, mount: Box<dyn Mount>) {
        if let NodeKind::Dir(dir) = &mut self.kind {
            dir.mount(target, mount);
        } else {
            panic!("Attempted to mount to a non-directory node!")
        }
    }

    fn create_dir(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            kind: NodeKind::Dir(VirtualDir::default()),
        }
    }
    fn create_mount(name: &str, mount: Box<dyn Mount>) -> Self {
        Self {
            name: name.to_owned(),
            kind: NodeKind::Mount(mount),
        }
    }
}

// The Debug trait bound may be removed in the future
pub trait Mount: std::fmt::Debug {
    // Returns a String only temporarily for testing purposes
    // We will pin down exactly what to return soon...
    fn open(&self, path: &VfsPath) -> Result<String>;
}

#[cfg(test)]
mod tests {
    use crate::path::*;
    use crate::*;

    // A testing mount that just echoes the paths given to it
    #[derive(Debug)]
    struct EchoMount;
    impl Mount for EchoMount {
        fn open(&self, path: &VfsPath) -> Result<String> {
            Ok(path.to_str().to_owned())
        }
    }

    // A mount that will always return "Not Found"
    #[derive(Debug)]
    struct EmptyMount;
    impl Mount for EmptyMount {
        fn open(&self, _path: &VfsPath) -> Result<String> {
            Err(ErrorKind::NotFound.into())
        }
    }

    #[test]
    fn vfs_path_resolution() {
        let mut vfs = VFS::new();
        vfs.mount(&VfsPath::new("/echo"), Box::new(EchoMount));
        let file = vfs.open(&VfsPath::new("/echo/hello/world"));
        assert_eq!(file.unwrap(), String::from("/hello/world"));
    }

    #[test]
    fn vfs_multiple_mounts() {
        let mut vfs = VFS::new();
        // Mount an echo at root to handle all not founds
        vfs.mount(&VfsPath::new("/"), Box::new(EchoMount));
        vfs.mount(&VfsPath::new("/empty"), Box::new(EmptyMount));

        let file = vfs.open(&VfsPath::new("/empty/hello/world"));

        // "/empty" should NOT be stripped because "/empty" will only return Err
        // The mount at "/" will thus be queried and return the untrimmed path
        assert_eq!(file.unwrap(), String::from("/empty/hello/world"));
    }
}
