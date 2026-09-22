use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use crate::image::{DiskFormat, DiskImage, DiskImageInfo, VhdImageType};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// VHD constants.
pub const VHD_SECTOR_SIZE: u32 = 512;
pub const VHD_BLOCK_SIZE: u32 = 2 * 1024 * 1024; // 2MB
pub const VHD_FOOTER_SIZE: u32 = 512;
pub const VHD_MAX_SECTORS: u64 = (2u64 * 1024 * 1024 * 1024 * 1024) / 512; // 2TB
pub const VHD_FOOTER_COOKIE: &str = "conectix";
pub const VHD_FOOTER_VERSION: u32 = 0x00010000;

/// VHD footer structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C, packed)]
pub struct VhdFooter {
    pub cookie: [u8; 8],
    pub features: u32,
    pub version: u32,
    pub data_offset: u64,
    pub timestamp: u32,
    pub creator_app: [u8; 4],
    pub creator_ver: u32,
    pub creator_os: u32,
    pub orig_size: u64,
    pub cur_size: u64,
    pub disk_geometry_cylinder: u16,
    pub disk_geometry_heads: u8,
    pub disk_geometry_sectors: u8,
    pub disk_type: u32,
    pub checksum: u32,
    pub unique_id: [u8; 16],
    pub saved_state: u8,
    pub reserved: [u8; 427],
}

impl Default for VhdFooter {
    fn default() -> Self {
        let mut cookie = [0u8; 8];
        cookie.copy_from_slice(b"conectix");
        Self {
            cookie,
            features: 0,
            version: VHD_FOOTER_VERSION,
            data_offset: 0xFFFFFFFFFFFFFFFF,
            timestamp: 0,
            creator_app: [0u8; 4],
            creator_ver: 0,
            creator_os: 0,
            orig_size: 0,
            cur_size: 0,
            disk_geometry_cylinder: 0,
            disk_geometry_heads: 0,
            disk_geometry_sectors: 0,
            disk_type: 0,
            checksum: 0,
            unique_id: [0u8; 16],
            saved_state: 0,
            reserved: [0u8; 427],
        }
    }
}

/// VHD block array entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhdBlockArrayEntry {
    pub offset: u32,
    pub type_: u32,
    pub timestamp: u32,
    pub file_name: [u8; 256],
}

impl Default for VhdBlockArrayEntry {
    fn default() -> Self {
        Self {
            offset: 0xFFFFFFFF,
            type_: 0,
            timestamp: 0,
            file_name: [0u8; 256],
        }
    }
}

/// VHD BAT (Block Allocation Table) entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhdBatEntry {
    pub block_offset: u32,
    pub block_type: u32,
}

/// VHD header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VhdHeader {
    pub footer: VhdFooter,
    pub block_array: Vec<VhdBlockArrayEntry>,
    pub bat: RwLock<Vec<VhdBatEntry>>,
    pub block_size: u32,
    pub total_blocks: u64,
    pub disk_type: VhdImageType,
}

impl Default for VhdHeader {
    fn default() -> Self {
        Self {
            footer: VhdFooter::default(),
            block_array: Vec::new(),
            bat: RwLock::new(Vec::new()),
            block_size: VHD_BLOCK_SIZE,
            total_blocks: 0,
            disk_type: VhdImageType::Dynamic,
        }
    }
}

/// VHD image.
pub struct VhdImage {
    header: VhdHeader,
    data: RwLock<HashMap<u64, Vec<u8>>>,
    is_open: bool,
    info: DiskImageInfo,
}

#[derive(Error, Debug)]
pub enum VhdError {
    #[error("Invalid VHD cookie")]
    InvalidCookie,
    #[error("Invalid VHD checksum")]
    InvalidChecksum,
    #[error("Unsupported VHD disk type")]
    UnsupportedDiskType,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl VhdImage {
    /// Open a VHD image from a file path.
    pub fn open(path: &str) -> VBoxResult<VhdImage> {
        let data = std::fs::read(path)?;
        let header = Self::parse_header(&data)?;
        let total_blocks = header.total_blocks;
        let block_size = header.block_size;

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VHD,
            total_blocks,
            VHD_SECTOR_SIZE,
        );

        let mut bat = Vec::with_capacity(total_blocks as usize);
        for _ in 0..total_blocks {
            bat.push(VhdBatEntry::default());
        }

        Ok(Self {
            header,
            data: RwLock::new(HashMap::new()),
            is_open: true,
            info,
        })
    }

    /// Parse the VHD footer from a byte slice.
    pub fn parse_footer(data: &[u8]) -> VBoxResult<VhdFooter> {
        if data.len() < VHD_FOOTER_SIZE as usize {
            return Err(VBoxError::BadFile);
        }
        let footer: VhdFooter = unsafe {
            let ptr = data.as_ptr() as *const VhdFooter;
            (*ptr).clone()
        };

        let cookie_str = std::str::from_utf8(&footer.cookie).map_err(|_| VBoxError::BadFile)?;
        if cookie_str != VHD_FOOTER_COOKIE {
            return Err(VBoxError::BadFile);
        }

        let mut sum: u32 = 0;
        for i in (0..VHD_FOOTER_SIZE as usize).step_by(4) {
            sum += u32::from_le_bytes([
                data[i], data[i+1], data[i+2], data[i+3],
            ]);
        }
        if sum != 0xFFFFFFFF {
            return Err(VBoxError::BadFile);
        }

        Ok(footer)
    }

    /// Parse the full VHD header.
    pub fn parse_header(data: &[u8]) -> VBoxResult<VhdHeader> {
        let footer = Self::parse_footer(data)?;
        let disk_type = match footer.disk_type {
            2 => VhdImageType::Fixed,
            3 => VhdImageType::Dynamic,
            4 => VhdImageType::Differencing,
            _ => return Err(VBoxError::BadParam),
        };

        let cur_size = footer.cur_size;
        let total_blocks = cur_size / (footer.block_size.max(VHD_BLOCK_SIZE)) / VHD_SECTOR_SIZE as u64;
        let block_size = VHD_BLOCK_SIZE;

        let mut header = VhdHeader::default();
        header.footer = footer;
        header.disk_type = disk_type;
        header.block_size = block_size;
        header.total_blocks = total_blocks.max(1);

        let num_array_entries = (VHD_FOOTER_SIZE + 511) / 512;
        for _ in 0..num_array_entries {
            header.block_array.push(VhdBlockArrayEntry::default());
        }

        Ok(header)
    }

    /// Create a new VHD image.
    pub fn create(path: &str, size_bytes: u64, disk_type: VhdImageType) -> VBoxResult<VhdImage> {
        let total_sectors = size_bytes / VHD_SECTOR_SIZE as u64;
        let block_size = VHD_BLOCK_SIZE;
        let num_blocks = total_sectors / (block_size / VHD_SECTOR_SIZE);

        let mut footer = VhdFooter::default();
        footer.cookie.copy_from_slice(b"conectix");
        footer.disk_type = disk_type as u32;
        footer.orig_size = size_bytes;
        footer.cur_size = size_bytes;
        footer.version = VHD_FOOTER_VERSION;
        footer.data_offset = 0xFFFFFFFFFFFFFFFF;

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VHD,
            total_sectors,
            VHD_SECTOR_SIZE,
        );

        let mut header = VhdHeader::default();
        header.footer = footer;
        header.disk_type = disk_type;
        header.block_size = block_size;
        header.total_blocks = num_blocks;
        header.block_array = vec![VhdBlockArrayEntry::default(); num_blocks as usize];

        Ok(Self {
            header,
            data: RwLock::new(HashMap::new()),
            is_open: true,
            info,
        })
    }

    /// Read data from a block.
    fn read_block_data(&self, block_index: u64) -> VBoxResult<Vec<u8>> {
        let block_size = self.header.block_size as usize;
        let bat = self.header.bat.read();
        let entry = bat.get(block_index as usize)
            .ok_or(VBoxError::NotFound)?;
        if entry.block_offset == 0xFFFFFFFF {
            return Ok(vec![0u8; block_size]);
        }
        Ok(vec![0u8; block_size])
    }
}

impl DiskImage for VhdImage {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>> {
        let sectors_per_block = (self.header.block_size / VHD_SECTOR_SIZE) as u64;
        let block_index = lba / sectors_per_block;
        let offset_in_block = (lba % sectors_per_block) * VHD_SECTOR_SIZE as u64;
        let block_data = self.read_block_data(block_index)?;
        let sector_size = VHD_SECTOR_SIZE as usize;
        let start = offset_in_block as usize;
        let end = start + sector_size;
        if end > block_data.len() {
            return Err(VBoxError::BadParam);
        }
        Ok(block_data[start..end].to_vec())
    }

    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()> {
        if data.len() != VHD_SECTOR_SIZE as usize {
            return Err(VBoxError::BadParam);
        }
        let sectors_per_block = (self.header.block_size / VHD_SECTOR_SIZE) as u64;
        let block_index = lba / sectors_per_block;
        let mut bat = self.header.bat.write();
        if let Some(entry) = bat.get_mut(block_index as usize) {
            entry.block_offset = 1;
        }
        Ok(())
    }

    fn get_sector_count(&self) -> u64 {
        self.header.total_blocks * (self.header.block_size / VHD_SECTOR_SIZE)
    }

    fn get_sector_size(&self) -> u32 {
        VHD_SECTOR_SIZE
    }

    fn get_type(&self) -> DiskFormat {
        DiskFormat::VHD
    }

    fn close(self: Box<Self>) -> VBoxResult<()> {
        let mut bat = self.header.bat.write();
        bat.clear();
        Ok(())
    }

    fn get_info(&self) -> DiskImageInfo {
        self.info.clone()
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

impl VhdImage {
    /// Create a fixed VHD disk.
    pub fn create_fixed(path: &str, size_bytes: u64) -> VBoxResult<VhdImage> {
        Self::create(path, size_bytes, VhdImageType::Fixed)
    }

    /// Create a dynamic VHD disk.
    pub fn create_dynamic(path: &str, size_bytes: u64) -> VBoxResult<VhdImage> {
        Self::create(path, size_bytes, VhdImageType::Dynamic)
    }

    /// Create a differencing VHD disk.
    pub fn create_differencing(path: &str, size_bytes: u64, parent_path: &str) -> VBoxResult<VhdImage> {
        Self::create(path, size_bytes, VhdImageType::Differencing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_footer_valid() {
        let mut data = vec![0u8; 512];
        data[0..8].copy_from_slice(b"conectix");
        let footer = VhdImage::parse_footer(&data);
        assert!(footer.is_ok());
    }

    #[test]
    fn test_invalid_cookie() {
        let data = vec![0u8; 512];
        let result = VhdImage::parse_footer(&data);
        assert!(result.is_err());
    }
}
