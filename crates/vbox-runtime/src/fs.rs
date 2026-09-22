use vbox_core::VBoxError;
use std::collections::HashMap;

#[derive(Debug)]
pub struct RTFileSystem {
    root: HashMap<String, Vec<u8>>,
}

impl RTFileSystem {
    pub fn new() -> Self {
        RTFileSystem { root: HashMap::new() }
    }

    pub fn create_file(&mut self, path: &str, data: Vec<u8>) -> Result<(), VBoxError> {
        self.root.insert(path.to_string(), data);
        Ok(())
    }

    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, VBoxError> {
        self.root.get(path)
            .cloned()
            .ok_or(VBoxError::FileNotFound)
    }

    pub fn remove(&mut self, path: &str) -> Result<(), VBoxError> {
        if self.root.remove(path).is_some() {
            Ok(())
        } else {
            Err(VBoxError::FileNotFound)
        }
    }

    pub fn exists(&self, path: &str) -> bool {
        self.root.contains_key(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_create_read() {
        let mut fs = RTFileSystem::new();
        fs.create_file("/test.txt", b"hello".to_vec()).unwrap();
        let data = fs.read_file("/test.txt").unwrap();
        assert_eq!(data, b"hello");
    }
}
