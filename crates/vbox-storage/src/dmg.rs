use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use crate::image::{DiskFormat, DiskImage, DiskImageInfo};
use parking_lot::RwLock;
use std::collections::BTreeMap;

/// DMG constants.
pub const DMG_MAGIC: u32 = 0x444D4721; // "!DMG"
pub const DMG_VERSION: u32 = 4;
pub const DMG_SECTOR_SIZE: u32 = 512;
pub const DMG_BLOCK_SIZE: u32 = 1 << 20;

/// DMG header structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmgHeader {
    pub magic: u32,
    pub version: u32,
    pub table_of_contents: [u32; 4],
    pub reserved1: [u8; 16],
    pub segment_number: u32,
    pub segment_count: u32,
    pub segment_id: [u8; 16],
    pub data_checksum: [u8; 20],
    pub image_size: u64,
    pub segment_size: u32,
    pub reserved2: [u8; 12],
    pub partition_count: u32,
    pub reserved3: [u8; 16],
}

impl Default for DmgHeader {
    fn default() -> Self {
        Self {
            magic: DMG_MAGIC,
            version: DMG_VERSION,
            table_of_contents: [0u32; 4],
            reserved1: [0u8; 16],
            segment_number: 0,
            segment_count: 0,
            segment_id: [0u8; 16],
            data_checksum: [0u8; 20],
            image_size: 0,
            segment_size: 0,
            reserved2: [0u8; 12],
            partition_count: 0,
            reserved3: [0u8; 16],
        }
    }
}

/// DMG partition entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmgPartition {
    pub offset: u64,
    pub size: u64,
    pub name: [u8; 128],
    pub type_: [u8; 128],
    pub permissions: u32,
    pub reserved: [u8; 12],
}

/// DMG image.
pub struct DmgImage {
    header: DmgHeader,
    partitions: RwLock<Vec<DmgPartition>>,
    blocks: RwLock<BTreeMap<u64, Vec<u8>>>,
    is_open: bool,
    info: DiskImageInfo,
}

#[derive(Error, Debug)]
pub enum DmgError {
    #[error("Invalid DMG magic")]
    InvalidMagic,
    #[error("Unsupported DMG version")]
    UnsupportedVersion,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl DmgImage {
    /// Open a DMG image from a file path.
    pub fn open(path: &str) -> VBoxResult<DmgImage> {
        let data = std::fs::read(path)?;
        let header = Self::parse_header(&data)?;
        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::DMG,
            header.image_size / DMG_SECTOR_SIZE,
            DMG_SECTOR_SIZE,
        );

        Ok(Self {
            header,
            partitions: RwLock::new(Vec::new()),
            blocks: RwLock::new(BTreeMap::new()),
            is_open: true,
            info,
        })
    }

    /// Parse the DMG header.
    pub fn parse_header(data: &[u8]) -> VBoxResult<DmgHeader> {
        if data.len() < 64 {
            return Err(VBoxError::BadFile);
        }
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != DMG_MAGIC {
            return Err(VBoxError::BadFile);
        }

        let mut header = DmgHeader::default();
        header.magic = magic;
        header.version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        header.table_of_contents.copy_from_slice(&data[8..24]);
        header.reserved1.copy_from_slice(&data[24..40]);
        header.segment_number = u32::from_le_bytes([data[40], data[41], data[42], data[43]]);
        header.segment_count = u32::from_le_bytes([data[44], data[45], data[46], data[47]]);
        header.segment_id.copy_from_slice(&data[48..64]);
        if data.len() >= 84 {
            header.data_checksum.copy_from_slice(&data[64..84]);
            header.image_size = u64::from_le_bytes([
                data[84],data[85],data[86],data[87],data[88],data[89],data[90],data[91],
            ]);
            header.segment_size = u32::from_le_bytes([data[92], data[93], data[94], data[95]]);
            header.partition_count = u32::from_le_bytes([data[104], data[105], data[106], data[107]]);
        }

        Ok(header)
    }

    /// Create a new DMG image.
    pub fn create(path: &str, size_bytes: u64) -> VBoxResult<DmgImage> {
        let mut header = DmgHeader::default();
        header.magic = DMG_MAGIC;
        header.image_size = size_bytes;
        header.segment_size = DMG_BLOCK_SIZE;

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::DMG,
            size_bytes / DMG_SECTOR_SIZE,
            DMG_SECTOR_SIZE,
        );

        Ok(Self {
            header,
            partitions: RwLock::new(Vec::new()),
            blocks: RwLock::new(BTreeMap::new()),
            is_open: true,
            info,
        })
    }
}

impl DiskImage for DmgImage {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>> {
        let sector_size = DMG_SECTOR_SIZE as usize;
        Ok(vec![0u8; sector_size])
    }

    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()> {
        if data.len() != DMG_SECTOR_SIZE as usize {
            return Err(VBoxError::BadParam);
        }
        Ok(())
    }

    fn get_sector_count(&self) -> u64 {
        self.header.image_size / DMG_SECTOR_SIZE
    }

    fn get_sector_size(&self) -> u32 {
        DMG_SECTOR_SIZE
    }

    fn get_type(&self) -> DiskFormat {
        DiskFormat::DMG
    }

    fn close(self: Box<Self>) -> VBoxResult<()> {
        Ok(())
    }

    fn get_info(&self) -> DiskImageInfo {
        self.info.clone()
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dmg() {
        let img = DmgImage::create("/tmp/test.dmg", 1 << 30);
        assert!(img.is_ok());
    }
}
