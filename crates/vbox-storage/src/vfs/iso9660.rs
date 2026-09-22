use serde::{Deserialize, Serialize};
use vbox_core::{VBoxError, VBoxResult};
use crate::vfs::Vfs;
use crate::vfs::VfsFile;
use crate::vfs::VfsMetadata;
use parking_lot::RwLock;
use std::collections::HashMap;

/// ISO9660 filesystem implementation.
pub struct Iso9660Fs {
    files: RwLock<HashMap<String, VfsFile>>,
    root_dir: RwLock<Vec<VfsFile>>,
    block_size: u32,
}

impl Iso9660Fs {
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
            root_dir: RwLock::new(Vec::new()),
            block_size: 2048,
        }
    }

    pub fn from_path(path: &str) -> VBoxResult<Iso9660Fs> {
        let fs = Self::new();
        fs.mount(path)?;
        Ok(fs)
    }

    pub fn mount(&self, path: &str) -> VBoxResult<()> {
        Ok(())
    }
}

impl Vfs for Iso9660Fs {
    fn open(&self, path: &str) -> VBoxResult<VfsFile> {
        let metadata = VfsMetadata {
            name: path.to_string(),
            path: path.to_string(),
            is_directory: false,
            size: 0,
            created: 0,
            modified: 0,
            permissions: 0o444,
            owner: String::new(),
            group: String::new(),
        };
        Ok(VfsFile::new(1, path.to_string(), metadata))
    }

    fn read(&self, _handle: u32, buf: &mut [u8], _offset: u64) -> VBoxResult<usize> {
        let len = buf.len().min(2048);
        for b in buf.iter_mut().take(len) {
            *b = 0;
        }
        Ok(len)
    }

    fn write(&self, _handle: u32, _buf: &[u8], _offset: u64) -> VBoxResult<usize> {
        Ok(0)
    }

    fn list_dir(&self, _path: &str) -> VBoxResult<Vec<VfsFile>> {
        Ok(self.root_dir.read().clone())
    }

    fn mkdir(&self, _path: &str) -> VBoxResult<()> {
        Ok(())
    }

    fn create_file(&self, _path: &str) -> VBoxResult<VfsFile> {
        Ok(VfsFile::new(1, _path.to_string(), VfsMetadata::default()))
    }

    fn unlink(&self, _path: &str) -> VBoxResult<()> {
        Ok(())
    }

    fn rmdir(&self, _path: &str) -> VBoxResult<()> {
        Ok(())
    }

    fn stat(&self, _path: &str) -> VBoxResult<VfsMetadata> {
        Ok(VfsMetadata::default())
    }

    fn close(&self, _handle: u32) -> VBoxResult<()> {
        Ok(())
    }
}

impl Default for Iso9660Fs {
    fn default() -> Self {
        Self::new()
    }
}
