use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use crate::image::{DiskFormat, DiskImage, DiskImageInfo, VmdkImageType};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::collections::HashMap;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::Read;
use std::io::Write;

/// VMDK descriptor constants.
pub const VMDK_DESCRIPTOR_MAGIC: &str = "#!/usr/bin/vmware";
pub const VMDK_SECTOR_SIZE: u32 = 512;
pub const VMDK_DEFAULT_GRAIN_SIZE: u32 = 64 * 1024 * 1024; // 64MB
pub const VMDK_MAX_GRAIN_SIZE: u32 = 1024 * 1024 * 1024; // 1GB
pub const VMDK_HEADER_SIZE: usize = 512;

/// VMDK grain directory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmdkGrainDirEntry {
    pub offset: u64,
    pub size: u32,
    pub grain_size: u32,
    pub flags: u32,
}

/// VMDK grain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmdkGrain {
    pub data: Vec<u8>,
    pub compressed: bool,
}

/// VMDK grain directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmdkGrainDirectory {
    pub entries: Vec<VmdkGrainDirEntry>,
    pub total_grains: u64,
}

/// VMDK descriptor line.
#[derive(Debug, Clone)]
pub struct VmdkDescriptorLine {
    pub key: String,
    pub value: String,
}

/// VMDK image header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmdkHeader {
    pub descriptor: String,
    pub image_type: VmdkImageType,
    pub adapter_type: String,
    pub cylinders: u32,
    pub heads: u32,
    pub sectors_per_track: u32,
    pub geometry: String,
    pub tools_version: u32,
    pub disk_id: u32,
    pub parent_name: Option<String>,
    pub parent_uuid: Option<[u8; 16]>,
    pub uuid_image: [u8; 16],
    pub uuid_modification: [u8; 16],
    pub links: Vec<String>,
    pub sectors: u64,
    pub grain_size: u32,
    pub grain_directory: Option<VmdkGrainDirectory>,
    pub grains: RwLock<BTreeMap<u64, VmdkGrain>>,
}

impl Default for VmdkHeader {
    fn default() -> Self {
        Self {
            descriptor: String::new(),
            image_type: VmdkImageType::StreamOptimized,
            adapter_type: "lsisas1068".to_string(),
            cylinders: 0,
            heads: 0,
            sectors_per_track: 0,
            geometry: String::new(),
            tools_version: 0,
            disk_id: 0,
            parent_name: None,
            parent_uuid: None,
            uuid_image: [0u8; 16],
            uuid_modification: [0u8; 16],
            links: Vec::new(),
            sectors: 0,
            grain_size: VMDK_DEFAULT_GRAIN_SIZE,
            grain_directory: None,
            grains: RwLock::new(BTreeMap::new()),
        }
    }
}

/// VMDK image.
pub struct VmdkImage {
    header: VmdkHeader,
    data: RwLock<HashMap<u64, Vec<u8>>>,
    is_open: bool,
    info: DiskImageInfo,
}

#[derive(Error, Debug)]
pub enum VmdkError {
    #[error("Invalid VMDK descriptor")]
    InvalidDescriptor,
    #[error("Invalid VMDK header")]
    InvalidHeader,
    #[error("Compression error")]
    CompressionError,
    #[error("Grain {0} not found")]
    GrainNotFound(u64),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl VmdkImage {
    /// Open a VMDK image from a file path.
    pub fn open(path: &str) -> VBoxResult<VmdkImage> {
        let content = std::fs::read_to_string(path)?;
        let (header, descriptor_lines) = Self::parse_descriptor(&content)?;
        let sectors = header.sectors;
        let grain_size = header.grain_size;

        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VMDK,
            sectors,
            VMDK_SECTOR_SIZE,
        );

        Ok(Self {
            header,
            data: RwLock::new(HashMap::new()),
            is_open: true,
            info,
        })
    }

    /// Parse the VMDK descriptor text.
    pub fn parse_descriptor(content: &str) -> VBoxResult<(VmdkHeader, Vec<VmdkDescriptorLine>)> {
        let mut lines = Vec::new();
        let mut image_type = VmdkImageType::StreamOptimized;
        let mut adapter_type = String::new();
        let mut cylinders = 0u32;
        let mut heads = 0u32;
        let mut sectors_per_track = 0u32;
        let mut disk_id = 0u32;
        let mut uuid_image = [0u8; 16];
        let mut uuid_modification = [0u8; 16];
        let mut parent_name = None;
        let mut parent_uuid = None;
        let mut tools_version = 0u32;
        let mut sectors = 0u64;
        let mut grain_size = VMDK_DEFAULT_GRAIN_SIZE;
        let mut links = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                lines.push(VmdkDescriptorLine { key, value });

                match key.as_str() {
                    "ddb.virtualHWVersion" => tools_version = value.parse().unwrap_or(0),
                    "ddb.geometry.cylinders" => cylinders = value.parse().unwrap_or(0),
                    "ddb.geometry.heads" => heads = value.parse().unwrap_or(0),
                    "ddb.geometry.sectors" => sectors_per_track = value.parse().unwrap_or(0),
                    "ddb.adapterType" => adapter_type = value,
                    "ddb.diskId" => disk_id = value.parse().unwrap_or(0),
                    "ddb.uuid.image" => {
                        if value.len() >= 32 {
                            let bytes = hex::decode(&value[..32]).unwrap_or([0u8; 16]);
                            uuid_image.copy_from_slice(&bytes[..16]);
                        }
                    }
                    "ddb.uuid.modification" => {
                        if value.len() >= 32 {
                            let bytes = hex::decode(&value[..32]).unwrap_or([0u8; 16]);
                            uuid_modification.copy_from_slice(&bytes[..16]);
                        }
                    }
                    "ddb.uuid.parent" => {
                        parent_uuid = Some([0u8; 16]);
                    }
                    "ddb.size" => sectors = value.parse().unwrap_or(0),
                    "ddb.compression" => {
                        if value == "deflate" {
                            // compression handled at grain level
                        }
                    }
                    "parentName" => parent_name = Some(value),
                    _ => {}
                }
            }
        }

        let image_type = match adapter_type.as_str() {
            _ => VmdkImageType::StreamOptimized,
        };

        let header = VmdkHeader {
            descriptor: content.to_string(),
            image_type,
            adapter_type,
            cylinders,
            heads,
            sectors_per_track,
            geometry: format!("{}:{}:{}", cylinders, heads, sectors_per_track),
            tools_version,
            disk_id,
            parent_name,
            parent_uuid,
            uuid_image,
            uuid_modification,
            links,
            sectors,
            grain_size,
            grain_directory: None,
            grains: RwLock::new(BTreeMap::new()),
        };

        Ok((header, lines))
    }

    /// Create a new VMDK image.
    pub fn create(path: &str, size_bytes: u64) -> VBoxResult<VmdkImage> {
        let sectors = size_bytes / VMDK_SECTOR_SIZE as u64;
        let mut header = VmdkHeader::default();
        header.sectors = sectors;
        header.sectors = sectors;
        let info = DiskImageInfo::new(
            path.to_string(),
            DiskFormat::VMDK,
            sectors,
            VMDK_SECTOR_SIZE,
        );

        Ok(Self {
            header,
            data: RwLock::new(HashMap::new()),
            is_open: true,
            info,
        })
    }

    /// Compress grain data using deflate.
    fn compress_grain(data: &[u8]) -> VBoxResult<Vec<u8>> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        let compressed = encoder.finish()?;
        Ok(compressed)
    }

    /// Decompress grain data.
    fn decompress_grain(data: &[u8]) -> VBoxResult<Vec<u8>> {
        let mut decoder = DeflateDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Get a grain for a given grain index.
    fn get_grain(&self, grain_index: u64) -> VBoxResult<Vec<u8>> {
        let grains = self.header.grains.read();
        if let Some(grain) = grains.get(&grain_index) {
            if grain.compressed {
                return Self::decompress_grain(&grain.data);
            }
            return Ok(grain.data.clone());
        }
        let grain_size = self.header.grain_size as usize;
        Ok(vec![0u8; grain_size])
    }
}

impl DiskImage for VmdkImage {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>> {
        let grain_size_sectors = self.header.grain_size / VMDK_SECTOR_SIZE;
        let grain_index = lba / grain_size_sectors;
        let grain_data = self.get_grain(grain_index)?;
        let sector_offset = (lba % grain_size_sectors) * VMDK_SECTOR_SIZE as u64;
        let sector_data = &grain_data[sector_offset as usize..];
        Ok(sector_data[..VMDK_SECTOR_SIZE as usize].to_vec())
    }

    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()> {
        if data.len() != VMDK_SECTOR_SIZE as usize {
            return Err(VBoxError::BadParam);
        }
        let grain_size_sectors = self.header.grain_size / VMDK_SECTOR_SIZE;
        let grain_index = lba / grain_size_sectors;
        let mut grains = self.header.grains.write();
        let entry = grains.entry(grain_index).or_insert_with(|| {
            VmdkGrain {
                data: vec![0u8; self.header.grain_size as usize],
                compressed: false,
            }
        });
        let sector_offset = (lba % grain_size_sectors) * VMDK_SECTOR_SIZE as u64;
        let dest = &mut entry.data[sector_offset as usize..];
        dest[..data.len()].copy_from_slice(data);
        Ok(())
    }

    fn get_sector_count(&self) -> u64 {
        self.header.sectors
    }

    fn get_sector_size(&self) -> u32 {
        VMDK_SECTOR_SIZE
    }

    fn get_type(&self) -> DiskFormat {
        DiskFormat::VMDK
    }

    fn close(self: Box<Self>) -> VBoxResult<()> {
        let mut grains = self.header.grains.write();
        grains.clear();
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
    fn test_parse_descriptor() {
        let content = r#"# Disk DescriptorFile
version=1
CID=12345678
parentCID=ffffffff
createType="streamOptimized"
# Extent description
RW 204800 SPARSE "test.vmdk"
ddb.virtualHWVersion = "8"
ddb.adapterType = "lsisas1068"
ddb.geometry.cylinders = "20"
ddb.geometry.heads = "64"
ddb.geometry.sectors = "32"
ddb.diskId = "12345"
"#;
        let result = VmdkImage::parse_descriptor(content);
        assert!(result.is_ok());
    }
}
