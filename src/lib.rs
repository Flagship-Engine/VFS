// We use io::Result as it's the most fitting for our purpose and no reason to reinvent the wheel
use std::io::{Error, ErrorKind, Result};

pub mod path;
use path::{VfsPath, VfsPathBuf};

#[derive(Default)]
pub struct VFS {
    root: VirtualDir,
}
impl VFS {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn mount(&mut self, target: &VfsPath, mount: Box<dyn Mount>) -> Result<()> {
        let (head, _) = target.take_head();
        
        // Check if we are mounting to this root
        if head == "" {
            self.root.nodes.push(Node::create_mount("", mount));
            Ok(())
        } else {
            // If root was not selected, then mount within a directory
            self.root.mount(target, mount)
        }
    }
}

#[derive(Default)]
struct VirtualDir {
    nodes: Vec<Node>,
}
impl VirtualDir {
    fn mount(&mut self, target: &VfsPath, mount: Box<dyn Mount>) -> Result<()> {
        let (head, tail) = target.take_head();
        match tail {
            // If tail is none, then we are at the end of a path
            None => {
                // Mount here and we're done :)
                self.nodes.push(Node::create_mount(head, mount));
                Ok(())
            },
            
            // There is still more path to resolve, continue...
            Some(tail) => {
                let maybe_dir = self.nodes.iter_mut().find(|n| n.is_dir() && n.name == head);
                
                match maybe_dir {
                    // The directory has already been created, continue traversing from here...
                    Some(dir) => {
                        Ok(())
                    },
                    
                    // The directory has not been made, create it and continue traversing...
                    None => {
                        Ok(())
                    }
                }
            }
        }
    }
}

struct Node {
    name: String,
    kind: NodeKind,
}
enum NodeKind {
    Dir(VirtualDir),
    Mount(Box<dyn Mount>),
}

impl Node {
    fn is_dir(&self) -> bool {
        match self.kind {
            NodeKind::Dir(_) => true,
            NodeKind::Mount(_) => false,
        }
    }
    fn is_mount(&self) -> bool {
        !self.is_dir()
    }

    
    fn create_dir(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            kind: NodeKind::Dir(VirtualDir::default())
        }
    } 
    fn create_mount(name: &str, mount: Box<dyn Mount>) -> Self {
        Self {
            name: name.to_owned(),
            kind: NodeKind::Mount(mount),
        }
    }
}

pub trait Mount {
    fn open(&self, path: &VfsPath) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use crate::path::*;
    use crate::*;
}
