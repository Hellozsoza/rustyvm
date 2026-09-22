use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use crate::image::{DiskFormat, DiskImage, DiskImageInfo, VdiImageType};
use parking_lot::RwLock;
use std::collections::BTreeMap;

/// VDI magic values.
pub const VDI_MAGIC_V1: u32 = 0xBEEFCAFE;
pub const VDI_MAGIC_V2: u32 = 0xDEAD10CC;
pub const VDI_HEADER_VERSION_V1: u32 = 0x00010001;
pub const VDI_HEADER_VERSION_V2: u32 = 0x00010002;

/// VDI pre-header size.
pub const VDI_PRE_HEADER_SIZE: usize = 64;
/// VDI header size.
pub const VDI_HEADER_SIZE: usize = 64 * 1024;
/// VDI image table entry size.
pub const VDI_IMAGE_TABLE_ENTRY_SIZE: usize = 4;

/// VDI header structure (version 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdiHeader {
    pub pre_header: VdiPreHeader,
    pub version: u32,
    pub image_type: VdiImageType,
    pub flags: u32,
    pub image_size: u64,
    pub block_size: u32,
    pub drive_type: u32,
    pub image_format: u32,
    pub offset_type: u32,
    pub num_blocks: u64,
    pub block_data_offset: u64,
    pub block_extra_data_offset: u64,
    pub reserved1: [u8; 256],
    pub uuid_image: [u8; 16],
    pub uuid_modification: [u8; 16],
    pub uuid_link: [u8; 16],
    pub uuid_parent: [u8; 16],
    pub reserved2: [u8; 128],
    pub image_extra_data_offset: u64,
    pub image_extra_data_size: u64,
}

impl Default for VdiHeader {
    fn default() -> Self {
        Self {
            pre_header: VdiPreHeader::default(),
            version: VDI_HEADER_VERSION_V2,
            image_type: VdiImageType::Normal,
            flags: 0,
            image_size: 0,
            block_size: 1 << 20,
            drive_type: 0,
            image_format: 0,
            offset_type: 0,
            num_blocks: 0,
            block_data_offset: 0,
            block_extra_data_offset: 0,
            reserved1: [0u8; 256],
            uuid_image: [0u8; 16],
            uuid_modification: [0u8; 16],
            uuid_link: [0u8; 16],
            uuid_parent: [0u8; 16],
            reserved2: [0u8; 128],
            image_extra_data_offset: 0,
            image_extra_data_size: 0,
        }
    }
}

/// VDI pre-header structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VdiPreHeader {
    pub sz_file_info: [u8; 64],
    pub u32_signature: u32,
    pub u32_version: u32,
}

/// VDI image table entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VdiImageTableEntry {
    pub offset: u32,
}

/// VDI image.
pub struct VdiImage {
    header: VdiHeader,
    image_table: RwLock<Vec<VdiImageTableEntry>>,
    block_size: u32,
    num_blocks: u64,
    is_open: bool,
    info: DiskImageInfo,
}

#[derive(Error, Debug)]
pub enum VdiError {
    #[error("Invalid VDI signature")]
    InvalidSignature,
    #[error("Unsupported VDI version")]
    UnsupportedVersion,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl VdiImage {
    /// Open a VDI image from a file path.
    pub fn open(path: &str) -> VBoxResult<VdiImage> {
        let data = std::fs::read(path)?;
        let header = Self::parse_header(&data)?;
        let block_size = header.block_size as usize;
        let num_blocks = header.num_blocks;

        let mut image_table = Vec::with_capacity(num_blocks as usize);
        let table_offset = header.block_data_offset as usize - (num_blocks * 4);
        for i in 0..num_blocks {
            let offset = if table_offset + (i * 4) < data.len() {
                u32::from_le_bytes([
                    data[table_offset + i * 4],
                    data[table_offset + i * 4 + 1],
                    data[table_offset + i * 4 + 2],
                    data[table_offset + i * 4 + 3],
                ])
            } else {
                0
            };
            image_table.push(VdiImageTableEntry { offset });
        }

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VDI,
            num_blocks * (block_size as u64 / 512),
            512,
        );

        Ok(Self {
            header,
            image_table: RwLock::new(image_table),
            block_size,
            num_blocks,
            is_open: true,
            info,
        })
    }

    /// Parse the VDI header from a byte slice.
    pub fn parse_header(data: &[u8]) -> VBoxResult<VdiHeader> {
        if data.len() < VDI_PRE_HEADER_SIZE {
            return Err(VBoxError::BadFile);
        }

        let mut pre_header = VdiPreHeader::default();
        pre_header.sz_file_info.copy_from_slice(&data[0..64]);
        pre_header.u32_signature = u32::from_le_bytes([data[64], data[65], data[66], data[67]]);
        pre_header.u32_version = u32::from_le_bytes([data[68], data[69], data[70], data[71]]);

        if pre_header.u32_signature != VDI_MAGIC_V1 && pre_header.u32_signature != VDI_MAGIC_V2 {
            return Err(VBoxError::BadFile);
        }

        let header_offset = VDI_PRE_HEADER_SIZE;
        let version = u32::from_le_bytes([data[header_offset], data[header_offset+1], data[header_offset+2], data[header_offset+3]]);
        let image_type_val = u32::from_le_bytes([data[header_offset+4], data[header_offset+5], data[header_offset+6], data[header_offset+7]]);
        let flags = u32::from_le_bytes([data[header_offset+8], data[header_offset+9], data[header_offset+10], data[header_offset+11]]);
        let image_size = u64::from_le_bytes([
            data[header_offset+12], data[header_offset+13], data[header_offset+14], data[header_offset+15],
            data[header_offset+16], data[header_offset+17], data[header_offset+18], data[header_offset+19],
        ]);
        let block_size = u32::from_le_bytes([
            data[header_offset+20], data[header_offset+21], data[header_offset+22], data[header_offset+23],
        ]);
        let drive_type = u32::from_le_bytes([
            data[header_offset+24], data[header_offset+25], data[header_offset+26], data[header_offset+27],
        ]);
        let image_format = u32::from_le_bytes([
            data[header_offset+28], data[header_offset+29], data[header_offset+30], data[header_offset+31],
        ]);
        let offset_type = u32::from_le_bytes([
            data[header_offset+32], data[header_offset+33], data[header_offset+34], data[header_offset+35],
        ]);
        let num_blocks = u64::from_le_bytes([
            data[header_offset+36], data[header_offset+37], data[header_offset+38], data[header_offset+39],
            data[header_offset+40], data[header_offset+41], data[header_offset+42], data[header_offset+43],
        ]);
        let block_data_offset = u64::from_le_bytes([
            data[header_offset+44], data[header_offset+45], data[header_offset+46], data[header_offset+47],
            data[header_offset+48], data[header_offset+49], data[header_offset+50], data[header_offset+51],
        ]);

        let mut header = VdiHeader::default();
        header.pre_header = pre_header;
        header.version = version;
        header.image_type = match image_type_val {
            0 => VdiImageType::Normal,
            1 => VdiImageType::Difference,
            2 => VdiImageType::Parent,
            _ => return Err(VBoxError::BadParam),
        };
        header.flags = flags;
        header.image_size = image_size;
        header.block_size = block_size;
        header.drive_type = drive_type;
        header.image_format = image_format;
        header.offset_type = offset_type;
        header.num_blocks = num_blocks;
        header.block_data_offset = block_data_offset;
        header.block_extra_data_offset = 0;

        Ok(header)
    }

    /// Create a new VDI image.
    pub fn create(path: &str, size_bytes: u64, block_size: u32) -> VBoxResult<VdiImage> {
        let num_blocks = size_bytes / (block_size as u64);
        let mut header = VdiHeader::default();
        header.image_size = size_bytes;
        header.block_size = block_size;
        header.num_blocks = num_blocks;
        header.block_data_offset = VDI_HEADER_SIZE as u64 + (num_blocks * 4);

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VDI,
            num_blocks * (block_size as u64 / 512),
            512,
        );

        let image_table = vec![VdiImageTableEntry { offset: 0 }; num_blocks as usize];

        Ok(Self {
            header,
            image_table: RwLock::new(image_table),
            block_size,
            num_blocks,
            is_open: true,
            info,
        })
    }

    /// Get the image table entry for a block.
    fn get_image_table_entry(&self, block_num: u64) -> VBoxResult<VdiImageTableEntry> {
        let table = self.image_table.read();
        table.get(block_num as usize)
            .copied()
            .ok_or(VBoxError::NotFound)
    }
}

impl DiskImage for VdiImage {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>> {
        let sectors_per_block = (self.block_size / 512) as u64;
        let block_num = lba / sectors_per_block;
        let offset_in_block = (lba % sectors_per_block) * 512;
        let entry = self.get_image_table_entry(block_num)?;
        let sector_size = 512usize;
        let block_data = vec![0u8; self.block_size as usize];
        let start = offset_in_block as usize;
        let end = start + sector_size;
        if end > block_data.len() {
            return Err(VBoxError::BadParam);
        }
        Ok(block_data[start..end].to_vec())
    }

    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()> {
        if data.len() != 512 {
            return Err(VBoxError::BadParam);
        }
        let sectors_per_block = (self.block_size / 512) as u64;
        let block_num = lba / sectors_per_block;
        let mut table = self.image_table.write();
        if let Some(entry) = table.get_mut(block_num as usize) {
            entry.offset = 1;
        }
        Ok(())
    }

    fn get_sector_count(&self) -> u64 {
        self.num_blocks * (self.block_size / 512)
    }

    fn get_sector_size(&self) -> u32 {
        512
    }

    fn get_type(&self) -> DiskFormat {
        DiskFormat::VDI
    }

    fn close(self: Box<Self>) -> VBoxResult<()> {
        let mut table = self.image_table.write();
        table.clear();
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
    fn test_parse_header_valid() {
        let mut data = vec![0u8; VDI_HEADER_SIZE];
        data[64..68].copy_from_slice(&VDI_MAGIC_V2.to_le_bytes());
        data[68..72].copy_from_slice(&VDI_HEADER_VERSION_V2.to_le_bytes());
        data[72..76].copy_from_slice(&0u32.to_le_bytes());
        let header = VdiImage::parse_header(&data);
        assert!(header.is_ok());
    }

    #[test]
    fn test_create_vdi() {
        let img = VdiImage::create("/tmp/test.vdi", 1 << 30, 1 << 20);
        assert!(img.is_ok());
        let img = img.unwrap();
        assert_eq!(img.num_blocks, 1024);
    }
}
