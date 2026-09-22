use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use crate::image::{DiskFormat, DiskImage, DiskImageInfo, VdImageType};
use parking_lot::RwLock;
use std::collections::BTreeMap;

/// VD (VirtualBox Disk) legacy format constants.
pub const VD_LEGACY_MAGIC: u32 = 0x42444944;
pub const VD_LEGACY_VERSION: u32 = 0x00010000;
pub const VD_LEGACY_BLOCK_SIZE: u32 = 1 << 20;
pub const VD_LEGACY_SECTOR_SIZE: u32 = 512;

/// VD legacy header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdLegacyHeader {
    pub magic: u32,
    pub version: u32,
    pub image_type: u32,
    pub flags: u32,
    pub image_size: u64,
    pub block_size: u32,
    pub total_blocks: u64,
    pub data_offset: u64,
    pub uuid_image: [u8; 16],
    pub uuid_modified: [u8; 16],
    pub uuid_parent: [u8; 16],
    pub reserved: [u8; 256],
}

/// VD block descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdBlockDescriptor {
    pub block_number: u64,
    pub offset: u64,
    pub size: u32,
    pub type_: u32,
    pub modified: bool,
}

/// VD legacy image.
pub struct VdLegacyImage {
    header: VdLegacyHeader,
    blocks: RwLock<BTreeMap<u64, VdBlockDescriptor>>,
    block_size: u32,
    total_blocks: u64,
    is_open: bool,
    info: DiskImageInfo,
}

impl VdLegacyImage {
    pub fn open(path: &str) -> VBoxResult<VdLegacyImage> {
        let data = std::fs::read(path)?;
        let mut header = VdLegacyHeader::default();
        if data.len() >= 348 {
            header.magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            header.version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            header.image_type = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
            header.flags = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
            header.image_size = u64::from_le_bytes([
                data[16],data[17],data[18],data[19],data[20],data[21],data[22],data[23],
            ]);
            header.block_size = u32::from_le_bytes([data[24],data[25],data[26],data[27]]);
            header.total_blocks = u64::from_le_bytes([
                data[28],data[29],data[30],data[31],data[32],data[33],data[34],data[35],
            ]);
            header.data_offset = u64::from_le_bytes([
                data[36],data[37],data[38],data[39],data[40],data[41],data[42],data[43],
            ]);
            header.uuid_image.copy_from_slice(&data[44..60]);
            header.uuid_modified.copy_from_slice(&data[60..76]);
            header.uuid_parent.copy_from_slice(&data[76..92]);
        }

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VD,
            header.total_blocks * (header.block_size / VD_LEGACY_SECTOR_SIZE),
            VD_LEGACY_SECTOR_SIZE,
        );

        Ok(Self {
            header,
            blocks: RwLock::new(BTreeMap::new()),
            block_size: header.block_size,
            total_blocks: header.total_blocks,
            is_open: true,
            info,
        })
    }

    pub fn create(path: &str, size_bytes: u64) -> VBoxResult<VdLegacyImage> {
        let block_size = VD_LEGACY_BLOCK_SIZE;
        let total_blocks = size_bytes / (block_size as u64);
        let mut header = VdLegacyHeader::default();
        header.magic = VD_LEGACY_MAGIC;
        header.version = VD_LEGACY_VERSION;
        header.image_size = size_bytes;
        header.block_size = block_size;
        header.total_blocks = total_blocks;
        header.data_offset = 348 + (total_blocks * 32);

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VD,
            total_blocks * (block_size / VD_LEGACY_SECTOR_SIZE),
            VD_LEGACY_SECTOR_SIZE,
        );

        Ok(Self {
            header,
            blocks: RwLock::new(BTreeMap::new()),
            block_size,
            total_blocks,
            is_open: true,
            info,
        })
    }
}

impl DiskImage for VdLegacyImage {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>> {
        let sectors_per_block = (self.block_size / VD_LEGACY_SECTOR_SIZE) as u64;
        if lba >= self.total_blocks * sectors_per_block {
            return Err(VBoxError::BadParam);
        }
        let sector_size = VD_LEGACY_SECTOR_SIZE as usize;
        Ok(vec![0u8; sector_size])
    }

    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()> {
        if data.len() != VD_LEGACY_SECTOR_SIZE as usize {
            return Err(VBoxError::BadParam);
        }
        Ok(())
    }

    fn get_sector_count(&self) -> u64 {
        self.total_blocks * (self.block_size / VD_LEGACY_SECTOR_SIZE)
    }

    fn get_sector_size(&self) -> u32 {
        VD_LEGACY_SECTOR_SIZE
    }

    fn get_type(&self) -> DiskFormat {
        DiskFormat::VD
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

impl Default for VdLegacyHeader {
    fn default() -> Self {
        Self {
            magic: 0,
            version: 0,
            image_type: 0,
            flags: 0,
            image_size: 0,
            block_size: 0,
            total_blocks: 0,
            data_offset: 0,
            uuid_image: [0u8; 16],
            uuid_modified: [0u8; 16],
            uuid_parent: [0u8; 16],
            reserved: [0u8; 256],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_legacy() {
        let img = VdLegacyImage::create("/tmp/test.vd", 1 << 30);
        assert!(img.is_ok());
    }
}
