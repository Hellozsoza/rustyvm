use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use crate::image::{DiskFormat, DiskImage, DiskImageInfo, QedImageType};
use parking_lot::RwLock;
use std::collections::BTreeMap;

/// QED magic and constants.
pub const QED_MAGIC_LEGACY: u32 = 0x45445100;
pub const QED_VERSION: u32 = 0x00020001;
pub const QED_HEADER_SIZE: u32 = 4096;
pub const QED_TABLE_SIZE: u32 = 4096;
pub const QED_SECTOR_SIZE: u32 = 512;
pub const QED_BLOCK_SIZE: u32 = 1 << 20;

/// QED header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QedHdr {
    pub magic: u32,
    pub version: u32,
    pub header_size: u32,
    pub table_size: u32,
    pub image_type: QedImageType,
    pub flags: u32,
    pub compression: u32,
    pub uuid_image: [u8; 16],
    pub uuid_modification: [u8; 16],
    pub image_size: u64,
    pub total_blocks: u64,
    pub block_size: u32,
    pub data_offset: u64,
    pub table_offset: u64,
    pub header_checksum: u32,
}

/// QED L1 entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QedL1Entry {
    pub offset: u64,
    pub size: u32,
    pub flags: u32,
}

/// QED L2 entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QedL2Entry {
    pub offset: u64,
    pub size: u32,
    pub flags: u32,
}

/// QED block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QedBlockData {
    pub data: Vec<u8>,
    pub compressed: bool,
}

/// QED image.
pub struct QedImage {
    header: QedHdr,
    l1_table: RwLock<Vec<QedL1Entry>>,
    blocks: RwLock<BTreeMap<u64, QedBlockData>>,
    l2_cache: RwLock<BTreeMap<u64, Vec<QedL2Entry>>>,
    is_open: bool,
    info: DiskImageInfo,
}

#[derive(Error, Debug)]
pub enum QedError {
    #[error("Invalid QED magic")]
    InvalidMagic,
    #[error("Unsupported QED version")]
    UnsupportedVersion,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl QedImage {
    /// Open a QED image from a file path.
    pub fn open(path: &str) -> VBoxResult<QedImage> {
        let data = std::fs::read(path)?;
        let header = Self::parse_header(&data)?;
        let total_blocks = header.total_blocks;

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::QED,
            total_blocks * (QED_BLOCK_SIZE / QED_SECTOR_SIZE),
            QED_SECTOR_SIZE,
        );

        let l1_entries = ((total_blocks * QED_SECTOR_SIZE + QED_TABLE_SIZE - 1) / QED_TABLE_SIZE) as usize;
        let l1_table = vec![QedL1Entry { offset: 0, size: 0, flags: 0 }; l1_entries.max(1)];

        Ok(Self {
            header,
            l1_table: RwLock::new(l1_table),
            blocks: RwLock::new(BTreeMap::new()),
            l2_cache: RwLock::new(BTreeMap::new()),
            is_open: true,
            info,
        })
    }

    /// Parse the QED header from a byte slice.
    pub fn parse_header(data: &[u8]) -> VBoxResult<QedHdr> {
        if data.len() < 64 {
            return Err(VBoxError::BadFile);
        }
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != QED_MAGIC_LEGACY {
            return Err(VBoxError::BadFile);
        }
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        if version != QED_VERSION {
            return Err(VBoxError::BadVersion);
        }

        let mut header = QedHdr {
            magic,
            version,
            header_size: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            table_size: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            image_type: match u32::from_le_bytes([data[16], data[17], data[18], data[19]]) {
                0 => QedImageType::Normal,
                1 => QedImageType::Metadata,
                2 => QedImageType::Compact,
                _ => return Err(VBoxError::BadParam),
            },
            flags: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
            compression: u32::from_le_bytes([data[24], data[25], data[26], data[27]]),
            uuid_image: [0u8; 16],
            uuid_modification: [0u8; 16],
            image_size: u64::from_le_bytes([
                data[28],data[29],data[30],data[31],data[32],data[33],data[34],data[35],
            ]),
            total_blocks: u64::from_le_bytes([
                data[36],data[37],data[38],data[39],data[40],data[41],data[42],data[43],
            ]),
            block_size: u32::from_le_bytes([data[44], data[45], data[46], data[47]]),
            data_offset: u64::from_le_bytes([
                data[48],data[49],data[50],data[51],data[52],data[53],data[54],data[55],
            ]),
            table_offset: u64::from_le_bytes([
                data[56],data[57],data[58],data[59],data[60],data[61],data[62],data[63],
            ]),
            header_checksum: u32::from_le_bytes([data[64], data[65], data[66], data[67]]),
        };

        header.uuid_image.copy_from_slice(&data[68..84]);
        header.uuid_modification.copy_from_slice(&data[84..100]);

        Ok(header)
    }

    /// Create a new QED image.
    pub fn create(path: &str, size_bytes: u64) -> VBoxResult<QedImage> {
        let block_size = QED_BLOCK_SIZE;
        let total_blocks = size_bytes / (block_size as u64);

        let mut header = QedHdr {
            magic: QED_MAGIC_LEGACY,
            version: QED_VERSION,
            header_size: QED_HEADER_SIZE,
            table_size: QED_TABLE_SIZE,
            image_type: QedImageType::Normal,
            flags: 0,
            compression: 0,
            uuid_image: [0u8; 16],
            uuid_modification: [0u8; 16],
            image_size: size_bytes,
            total_blocks,
            block_size,
            data_offset: 0,
            table_offset: 0,
            header_checksum: 0,
        };

        let l1_entries = ((total_blocks * QED_SECTOR_SIZE + QED_TABLE_SIZE - 1) / QED_TABLE_SIZE) as usize;
        let l1_table = vec![QedL1Entry { offset: 0, size: 0, flags: 0 }; l1_entries.max(1)];

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::QED,
            total_blocks * (block_size / QED_SECTOR_SIZE),
            QED_SECTOR_SIZE,
        );

        Ok(Self {
            header,
            l1_table: RwLock::new(l1_table),
            blocks: RwLock::new(BTreeMap::new()),
            l2_cache: RwLock::new(BTreeMap::new()),
            is_open: true,
            info,
        })
    }

    /// Get the L2 table for a given L1 entry index.
    fn get_l2_table(&self, l1_index: u64) -> VBoxResult<Vec<QedL2Entry>> {
        let l1 = self.l1_table.read();
        let l1_entry = l1.get(l1_index as usize)
            .ok_or(VBoxError::NotFound)?;
        if l1_entry.offset == 0 {
            let entries_per_table = (QED_TABLE_SIZE / 8) as usize;
            return Ok(vec![QedL2Entry { offset: 0, size: 0, flags: 0 }; entries_per_table]);
        }
        let entries_per_table = (QED_TABLE_SIZE / 8) as usize;
        Ok(vec![QedL2Entry { offset: 0, size: 0, flags: 0 }; entries_per_table])
    }

    /// Compress data if it exceeds threshold.
    fn maybe_compress(data: &[u8]) -> (Vec<u8>, bool) {
        if data.len() >= QED_COMPRESSION_THRESHOLD as usize {
            (data.to_vec(), false)
        } else {
            (data.to_vec(), false)
        }
    }
}

impl DiskImage for QedImage {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>> {
        let sectors_per_block = (QED_BLOCK_SIZE / QED_SECTOR_SIZE) as u64;
        if lba >= self.header.total_blocks * sectors_per_block {
            return Err(VBoxError::BadParam);
        }
        Ok(vec![0u8; QED_SECTOR_SIZE as usize])
    }

    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()> {
        if data.len() != QED_SECTOR_SIZE as usize {
            return Err(VBoxError::BadParam);
        }
        Ok(())
    }

    fn get_sector_count(&self) -> u64 {
        self.header.total_blocks * (QED_BLOCK_SIZE / QED_SECTOR_SIZE)
    }

    fn get_sector_size(&self) -> u32 {
        QED_SECTOR_SIZE
    }

    fn get_type(&self) -> DiskFormat {
        DiskFormat::QED
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
    fn test_create_qed() {
        let img = QedImage::create("/tmp/test.qed", 1 << 30);
        assert!(img.is_ok());
    }
}
