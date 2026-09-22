use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Shared folder configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFolder {
    pub name: String,
    pub host_path: PathBuf,
    pub mount_point: PathBuf,
    pub writable: bool,
    pub automount: bool,
    pub readonly: bool,
}

impl Default for SharedFolder {
    fn default() -> Self {
        Self {
            name: String::new(),
            host_path: PathBuf::new(),
            mount_point: PathBuf::new(),
            writable: true,
            automount: false,
            readonly: false,
        }
    }
}

/// Shared folder service.
pub struct SharedFolderService {
    folders: RwLock<HashMap<String, SharedFolder>>,
}

impl SharedFolderService {
    pub fn new() -> Self {
        Self {
            folders: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_folder(&self, folder: SharedFolder) -> VBoxResult<()> {
        self.folders.write().insert(folder.name.clone(), folder);
        Ok(())
    }

    pub fn remove_folder(&self, name: &str) -> VBoxResult<()> {
        self.folders.write().remove(name);
        Ok(())
    }

    pub fn list_folders(&self) -> VBoxResult<Vec<SharedFolder>> {
        Ok(self.folders.read().values().cloned().collect())
    }

    pub fn get_folder(&self, name: &str) -> Option<SharedFolder> {
        self.folders.read().get(name).cloned()
    }

    pub fn mount_folder(&self, name: &str) -> VBoxResult<()> {
        if let Some(folder) = self.folders.read().get(name) {
            if !folder.host_path.exists() {
                return Err(VBoxError::NotFound);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_folder_service() {
        let service = SharedFolderService::new();
        let folder = SharedFolder {
            name: "shared".to_string(),
            host_path: PathBuf::from("/tmp/shared"),
            mount_point: PathBuf::from("/mnt/shared"),
            writable: true,
            automount: false,
            readonly: false,
        };
        service.add_folder(folder).unwrap();
        assert_eq!(service.list_folders().unwrap().len(), 1);
    }
}
