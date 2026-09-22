pub mod ahci;
pub mod nvme;
pub mod vd_driver;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Storage backend trait for reading/writing sectors.
pub trait StorageBackend: Send + Sync {
    /// Reads sectors from the storage device.
    fn read_sectors(&self, lba: u64, count: u32) -> Result<Vec<u8>, StorageError>;
    /// Writes sectors to the storage device.
    fn write_sectors(&mut self, lba: u64, count: u32, data: &[u8]) -> Result<(), StorageError>;
}

/// Error type for storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Unsupported command: 0x{0:X}")]
    UnsupportedCommand(u8),
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Device not ready")]
    NotReady,
    #[error("Invalid LBA: {0}")]
    InvalidLba(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_backend_trait() {
        // StorageBackend is a trait, just verify it compiles.
    }
}
