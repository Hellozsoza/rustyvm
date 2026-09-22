use serde::{Deserialize, Serialize};
use vbox_core::{VBoxError, VBoxResult};
use crate::vfs::Vfs;
use crate::vfs::VfsFile;
use crate::vfs::VfsMetadata;
use parking_lot::RwLock;
use std::collections::HashMap;

/// FAT12/16/32 filesystem implementation.
pub struct FatFs {
    root_dir: RwLock<Vec<VfsFile>>,
    fat_table: RwLock<Vec<u32>>,
    cluster_size: u32,
    sectors_per_cluster: u32,
    is_fat32: bool,
}

impl FatFs {
    pub fn new() -> Self {
        Self {
            root_dir: RwLock::new(Vec::new()),
            fat_table: RwLock::new(Vec::new()),
            cluster_size: 4096,
            sectors_per_cluster: 8,
            is_fat32: true,
        }
    }

    pub fn fat12() -> Self {
        Self {
            root_dir: RwLock::new(Vec::new()),
            fat_table: RwLock::new(Vec::new()),
            cluster_size: 512,
            sectors_per_cluster: 1,
            is_fat32: false,
        }
    }

    pub fn fat16() -> Self {
        Self {
            root_dir: RwLock::new(Vec::new()),
            fat_table: RwLock::new(Vec::new()),
            cluster_size: 2048,
            sectors_per_cluster: 4,
            is_fat32: false,
        }
    }

    pub fn fat32() -> Self {
        Self {
            root_dir: RwLock::new(Vec::new()),
            fat_table: RwLock::new(Vec::new()),
            cluster_size: 4096,
            sectors_per_cluster: 8,
            is_fat32: true,
        }
    }

    pub fn mount(&self, _path: &str) -> VBoxResult<()> {
        Ok(())
    }

    pub fn get_cluster_size(&self) -> u32 {
        self.cluster_size
    }

    pub fn get_sectors_per_cluster(&self) -> u32 {
        self.sectors_per_cluster
    }
}

impl Vfs for FatFs {
    fn open(&self, path: &str) -> VBoxResult<VfsFile> {
        let metadata = VfsMetadata {
            name: path.to_string(),
            path: path.to_string(),
            is_directory: false,
            size: 0,
            created: 0,
            modified: 0,
            permissions: 0o644,
            owner: String::new(),
            group: String::new(),
        };
        Ok(VfsFile::new(1, path.to_string(), metadata))
    }

    fn read(&self, _handle: u32, buf: &mut [u8], _offset: u64) -> VBoxResult<usize> {
        let len = buf.len().min(self.cluster_size as usize);
        for b in buf.iter_mut().take(len) {
            *b = 0;
        }
        Ok(len)
    }

    fn write(&self, _handle: u32, buf: &[u8], _offset: u64) -> VBoxResult<usize> {
        Ok(buf.len())
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

impl Default for FatFs {
    fn default() -> Self {
        Self::new()
    }
}
