pub mod fat;
pub mod iso9660;
pub mod udf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;

/// VFS error types.
#[derive(Error, Debug)]
pub enum VfsError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not a directory: {0}")]
    NotADirectory(String),
    #[error("Unsupported filesystem")]
    UnsupportedFilesystem,
}

/// File metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsMetadata {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
    pub created: i64,
    pub modified: i64,
    pub permissions: u32,
    pub owner: String,
    pub group: String,
}

impl Default for VfsMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
            is_directory: false,
            size: 0,
            created: 0,
            modified: 0,
            permissions: 0o644,
            owner: String::new(),
            group: String::new(),
        }
    }
}

/// VFS file handle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsFile {
    pub handle: u32,
    pub path: String,
    pub position: u64,
    pub metadata: VfsMetadata,
    pub is_open: bool,
    pub read_only: bool,
}

impl VfsFile {
    pub fn new(handle: u32, path: String, metadata: VfsMetadata) -> Self {
        Self {
            handle,
            path,
            position: 0,
            metadata,
            is_open: true,
            read_only: false,
        }
    }
}

/// Virtual Filesystem trait.
pub trait Vfs: Send + Sync {
    fn open(&self, path: &str) -> VBoxResult<VfsFile>;
    fn read(&self, handle: u32, buf: &mut [u8], offset: u64) -> VBoxResult<usize>;
    fn write(&self, handle: u32, buf: &[u8], offset: u64) -> VBoxResult<usize>;
    fn list_dir(&self, path: &str) -> VBoxResult<Vec<VfsFile>>;
    fn mkdir(&self, path: &str) -> VBoxResult<()>;
    fn create_file(&self, path: &str) -> VBoxResult<VfsFile>;
    fn unlink(&self, path: &str) -> VBoxResult<()>;
    fn rmdir(&self, path: &str) -> VBoxResult<()>;
    fn stat(&self, path: &str) -> VBoxResult<VfsMetadata>;
    fn close(&self, handle: u32) -> VBoxResult<()>;
}

/// Virtual Filesystem instance.
pub struct VfsInstance {
    filesystems: RwLock<HashMap<String, Box<dyn Vfs>>>,
    next_handle: RwLock<u32>,
}

impl VfsInstance {
    pub fn new() -> Self {
        Self {
            filesystems: RwLock::new(HashMap::new()),
            next_handle: RwLock::new(1),
        }
    }

    pub fn register_filesystem(&self, name: String, fs: Box<dyn Vfs>) {
        self.filesystems.write().insert(name, fs);
    }

    pub fn get_filesystem(&self, name: &str) -> Option<Box<dyn Vfs>> {
        self.filesystems.read().get(name).cloned()
    }

    pub fn allocate_handle(&self) -> u32 {
        let mut handle = self.next_handle.write();
        let h = *handle;
        *handle = handle.wrapping_add(1);
        h
    }
}

impl Default for VfsInstance {
    fn default() -> Self {
        Self::new()
    }
}

/// Virtual Filesystem manager.
pub struct VfsManager {
    instance: VfsInstance,
}

impl VfsManager {
    pub fn new() -> Self {
        Self {
            instance: VfsInstance::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_manager() {
        let manager = VfsManager::new();
    }
}
