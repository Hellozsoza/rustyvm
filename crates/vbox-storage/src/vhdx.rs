use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use crate::image::{DiskFormat, DiskImage, DiskImageInfo};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// VHDX constants.
pub const VHDX_SECTOR_SIZE: u32 = 512;
pub const VHDX_BLOCK_SIZE: u32 = 1 << 20; // 1MB
pub const VHDX_LOG_SECTOR_SIZE: u32 = 512;
pub const VHDX_MAX_BLOCKS: u64 = 1 << 32;
pub const VHDX_MAX_SECTORS: u64 = 1 << 32;

/// VHDX signature constants.
pub const VHDX_SIGNATURE_BLOCK: u64 = 0x424C4B464D545848; // "FHXTMBLK"
pub const VHDX_SIGNATURE_DYNAMIC_DISK: u64 = 0x4244495358484456; // "VHDXSDIB"
pub const VHDX_SIGNATURE_METADATA: u64 = 0x4D44444B544F4C45; // "ELODTDM"
pub const VHDX_SIGNATURE_REGION_TABLE: u64 = 0x414C474552544144; // "DATEGRTLA"
pub const VHDX_SIGNATURE_LOG: u64 = 0x474F4C524547414C; // "LARGELG"

/// VHDX file type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum VhdxFileType {
    DynamicDisk = 0,
    FixedDisk = 1,
    DifferencingDisk = 2,
    Backup = 3,
}

/// VHDX header structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhdxHeader {
    pub signature: u64,
    pub file_type: VhdxFileType,
    pub version: u32,
    pub log_page_offset: u32,
    pub max_log_size: u32,
    pub block_size: u32,
    pub data_offset: u64,
    pub log_offset: u64,
    pub total_blocks: u64,
    pub logical_block_size: u32,
    pub physical_block_size: u32,
    pub parent_locator_size: u32,
    pub reserved: [u8; 256],
    pub checksum: u32,
    pub encoded_checksum: u32,
    pub flags: u32,
}

impl Default for VhdxHeader {
    fn default() -> Self {
        Self {
            signature: VHDX_SIGNATURE_DYNAMIC_DISK,
            file_type: VhdxFileType::DynamicDisk,
            version: 1,
            log_page_offset: 0,
            max_log_size: 0,
            block_size: VHDX_BLOCK_SIZE,
            data_offset: 0,
            log_offset: 0,
            total_blocks: 0,
            logical_block_size: VHDX_SECTOR_SIZE,
            physical_block_size: VHDX_BLOCK_SIZE,
            parent_locator_size: 0,
            reserved: [0u8; 256],
            checksum: 0,
            encoded_checksum: 0,
            flags: 0,
        }
    }
}

/// VHDX region table entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhdxRegionEntry {
    pub offset: u64,
    pub size: u64,
    pub guid: [u8; 16],
}

/// VHDX BAT (Block Allocation Table) entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhdxBatEntry {
    pub block_offset: u64,
    pub block_type: u8,
    pub reserved: [u8; 7],
}

impl Default for VhdxBatEntry {
    fn default() -> Self {
        Self {
            block_offset: 0,
            block_type: 0,
            reserved: [0u8; 7],
        }
    }
}

/// VHDX metadata item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhdxMetadataItem {
    pub item_id: u16,
    pub item_size: u32,
    pub data: Vec<u8>,
}

/// VHDX image.
pub struct VhdxImage {
    header: VhdxHeader,
    region_table: RwLock<Vec<VhdxRegionEntry>>,
    bat: RwLock<Vec<VhdxBatEntry>>,
    metadata: RwLock<Vec<VhdxMetadataItem>>,
    blocks: RwLock<HashMap<u64, Vec<u8>>>,
    is_open: bool,
    info: DiskImageInfo,
}

#[derive(Error, Debug)]
pub enum VhdxError {
    #[error("Invalid VHDX signature")]
    InvalidSignature,
    #[error("Unsupported VHDX version")]
    UnsupportedVersion,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl VhdxImage {
    /// Open a VHDX image from a file path.
    pub fn open(path: &str) -> VBoxResult<VhdxImage> {
        let data = std::fs::read(path)?;
        let header = Self::parse_header(&data)?;
        let total_blocks = header.total_blocks;
        let block_size = header.block_size;

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VHDX,
            total_blocks * (block_size / VHDX_SECTOR_SIZE),
            VHDX_SECTOR_SIZE,
        );

        let bat = vec![VhdxBatEntry::default(); total_blocks as usize];

        Ok(Self {
            header,
            region_table: RwLock::new(Vec::new()),
            bat: RwLock::new(bat),
            metadata: RwLock::new(Vec::new()),
            blocks: RwLock::new(HashMap::new()),
            is_open: true,
            info,
        })
    }

    /// Parse the VHDX header.
    pub fn parse_header(data: &[u8]) -> VBoxResult<VhdxHeader> {
        if data.len() < 128 {
            return Err(VBoxError::BadFile);
        }
        let signature = u64::from_le_bytes([
            data[0],data[1],data[2],data[3],data[4],data[5],data[6],data[7],
        ]);
        if signature != VHDX_SIGNATURE_DYNAMIC_DISK && signature != VHDX_SIGNATURE_BLOCK {
            return Err(VBoxError::BadFile);
        }

        let mut header = VhdxHeader::default();
        header.signature = signature;
        header.file_type = match signature {
            VHDX_SIGNATURE_DYNAMIC_DISK => VhdxFileType::DynamicDisk,
            VHDX_SIGNATURE_BLOCK => VhdxFileType::FixedDisk,
            _ => VhdxFileType::DynamicDisk,
        };
        header.version = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        header.block_size = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        header.data_offset = u64::from_le_bytes([
            data[40],data[41],data[42],data[43],data[44],data[45],data[46],data[47],
        ]);
        header.total_blocks = u64::from_le_bytes([
            data[48],data[49],data[50],data[51],data[52],data[53],data[54],data[55],
        ]);
        header.logical_block_size = u32::from_le_bytes([data[56], data[57], data[58], data[59]]);
        header.physical_block_size = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
        header.flags = u32::from_le_bytes([data[64], data[65], data[66], data[67]]);
        header.checksum = u32::from_le_bytes([data[68], data[69], data[70], data[71]]);
        header.encoded_checksum = u32::from_le_bytes([data[72], data[73], data[74], data[75]]);

        Ok(header)
    }

    /// Create a new VHDX image.
    pub fn create(path: &str, size_bytes: u64, file_type: VhdxFileType) -> VBoxResult<VhdxImage> {
        let block_size = VHDX_BLOCK_SIZE;
        let total_blocks = size_bytes / (block_size as u64);
        let mut header = VhdxHeader::default();
        header.file_type = file_type;
        header.total_blocks = total_blocks;
        header.block_size = block_size;
        header.logical_block_size = VHDX_SECTOR_SIZE;
        header.physical_block_size = block_size;

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VHDX,
            total_blocks * (block_size / VHDX_SECTOR_SIZE),
            VHDX_SECTOR_SIZE,
        );

        let bat = vec![VhdxBatEntry::default(); total_blocks as usize];

        Ok(Self {
            header,
            region_table: RwLock::new(Vec::new()),
            bat: RwLock::new(bat),
            metadata: RwLock::new(Vec::new()),
            blocks: RwLock::new(HashMap::new()),
            is_open: true,
            info,
        })
    }

    /// Get the block data for a given block index.
    fn get_block_data(&self, block_index: u64) -> VBoxResult<Vec<u8>> {
        let blocks = self.blocks.read();
        blocks.get(&block_index)
            .cloned()
            .ok_or_else(|| VBoxError::NotFound)
    }
}

impl DiskImage for VhdxImage {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>> {
        let sectors_per_block = (VHDX_BLOCK_SIZE / VHDX_SECTOR_SIZE) as u64;
        if lba >= self.header.total_blocks * sectors_per_block {
            return Err(VBoxError::BadParam);
        }
        let sector_size = VHDX_SECTOR_SIZE as usize;
        Ok(vec![0u8; sector_size])
    }

    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()> {
        if data.len() != VHDX_SECTOR_SIZE as usize {
            return Err(VBoxError::BadParam);
        }
        Ok(())
    }

    fn get_sector_count(&self) -> u64 {
        self.header.total_blocks * (VHDX_BLOCK_SIZE / VHDX_SECTOR_SIZE)
    }

    fn get_sector_size(&self) -> u32 {
        VHDX_SECTOR_SIZE
    }

    fn get_type(&self) -> DiskFormat {
        DiskFormat::VHDX
    }

    fn close(self: Box<Self>) -> VBoxResult<()> {
        let mut blocks = self.blocks.write();
        blocks.clear();
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
    fn test_create_vhdx() {
        let img = VhdxImage::create("/tmp/test.vhdx", 1 << 30, VhdxFileType::DynamicDisk);
        assert!(img.is_ok());
    }
}
