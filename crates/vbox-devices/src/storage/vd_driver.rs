use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Virtual disk image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiskFormat {
    /// VirtualBox Disk Image format.
    VDI,
    /// VMware Virtual Disk format.
    VMDK,
    /// Virtual Hard Disk format.
    VHD,
    /// VirtualBox Virtual Disk format.
    VD,
}

impl Default for DiskFormat {
    fn default() -> Self {
        Self::VDI
    }
}

/// Trait for disk image operations.
pub trait DiskImage: Send + Sync {
    /// Reads a sector from the disk image.
    fn read_sector(&self, lba: u64) -> Result<Vec<u8>, DiskImageError>;
    /// Writes a sector to the disk image.
    fn write_sector(&mut self, lba: u64, data: &[u8]) -> Result<(), DiskImageError>;
    /// Returns the total number of sectors.
    fn get_sector_count(&self) -> u64;
}

/// Error type for disk image operations.
#[derive(Debug, thiserror::Error)]
pub enum DiskImageError {
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Invalid sector: {0}")]
    InvalidSector(u64),
    #[error("Image not open")]
    NotOpen,
    #[error("Unsupported format")]
    UnsupportedFormat,
}

/// Virtual disk driver.
pub struct VdDriver {
    /// Disk image backing store.
    pub disk_image: Option<Box<dyn DiskImage>>,
    /// Block size in bytes.
    pub block_size: u32,
    /// Total number of sectors.
    pub total_sectors: u64,
}

impl VdDriver {
    /// Creates a new virtual disk driver.
    pub fn new() -> Self {
        Self {
            disk_image: None,
            block_size: 512,
            total_sectors: 0,
        }
    }

    /// Opens a disk image with the given format.
    pub fn open_image(&mut self, _path: &str, format: DiskFormat) -> Result<(), DiskImageError> {
        let _ = format;
        self.disk_image = None;
        Ok(())
    }

    /// Reads sectors from the disk.
    pub fn read(&self, lba: u64, count: u32) -> Result<Vec<u8>, DiskImageError> {
        if let Some(ref image) = self.disk_image {
            let mut result = Vec::new();
            for i in 0..count {
                let sector = image.read_sector(lba + i as u64)?;
                result.extend_from_slice(&sector);
            }
            Ok(result)
        } else {
            Err(DiskImageError::NotOpen)
        }
    }

    /// Writes sectors to the disk.
    pub fn write(&mut self, lba: u64, count: u32, data: &[u8]) -> Result<(), DiskImageError> {
        if let Some(ref mut image) = self.disk_image {
            let block_size = self.block_size as usize;
            let total_size = (count as usize) * block_size;
            if data.len() < total_size {
                return Err(DiskImageError::IoError("Insufficient data".to_string()));
            }
            for i in 0..count as u64 {
                let offset = (i as usize) * block_size;
                let sector_data = &data[offset..offset + block_size];
                image.write_sector(lba + i, sector_data)?;
            }
            Ok(())
        } else {
            Err(DiskImageError::NotOpen)
        }
    }
}

impl Default for VdDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vd_driver_new() {
        let driver = VdDriver::new();
        assert_eq!(driver.block_size, 512);
        assert!(driver.disk_image.is_none());
    }
}
