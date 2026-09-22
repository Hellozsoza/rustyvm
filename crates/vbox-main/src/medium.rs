use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use serde_json;

/// Medium type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum MediumType {
    HardDisk = 0,
    DVD = 1,
    Floppy = 2,
    Network = 3,
}

/// Medium registration state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum MediumRegistration {
    NotRegistered = 0,
    Registered = 1,
}

/// Medium structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medium {
    pub uuid: Uuid,
    pub medium_type: MediumType,
    pub location: String,
    pub size_bytes: u64,
    pub format: String,
    pub name: String,
    pub description: String,
    pub registered: bool,
    pub creation_timestamp: i64,
    pub modification_timestamp: i64,
    pub parent_uuid: Option<Uuid>,
    pub children: Vec<Uuid>,
}

#[derive(Error, Debug)]
pub enum MediumError {
    #[error("Medium not found: {0}")]
    NotFound(String),
    #[error("Medium already registered")]
    AlreadyRegistered,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Medium {
    pub fn register(&mut self) -> VBoxResult<()> {
        self.registered = true;
        Ok(())
    }

    pub fn unregister(&mut self) -> VBoxResult<()> {
        self.registered = false;
        Ok(())
    }

    pub fn open(path: &str) -> VBoxResult<Medium> {
        let metadata = std::fs::metadata(path)?;
        let medium = Medium {
            uuid: Uuid::new_v4(),
            medium_type: MediumType::HardDisk,
            location: path.to_string(),
            size_bytes: metadata.len(),
            format: String::new(),
            name: String::new(),
            description: String::new(),
            registered: true,
            creation_timestamp: 0,
            modification_timestamp: 0,
            parent_uuid: None,
            children: Vec::new(),
        };
        Ok(medium)
    }

    pub fn close(&mut self) -> VBoxResult<()> {
        self.registered = false;
        Ok(())
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_size(&self) -> u64 {
        self.size_bytes
    }

    pub fn get_format(&self) -> &str {
        &self.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_medium_open() {
        let result = Medium::open("/tmp/test.vdi");
        assert!(result.is_ok());
    }
}
