#![allow(non_camel_case_types)]

pub mod vd;
pub mod vmdk;
pub mod vdi;
pub mod vhd;
pub mod qcow;
pub mod qed;
pub mod iscsi;
pub mod dmg;
pub mod vhdx;
pub mod vfs;
pub mod image;

pub use vbox_core::*;

pub trait DiskImage {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>>;
    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()>;
    fn read_sectors(&self, lba: u64, count: u32) -> VBoxResult<Vec<Vec<u8>>>;
    fn write_sectors(&mut self, lba: u64, data: &[Vec<u8>]) -> VBoxResult<()>;
    fn get_size(&self) -> u64;
    fn get_sector_size(&self) -> u32;
    fn get_type(&self) -> DiskType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskType {
    VDI,
    VMDK,
    VHD,
    VHDX,
    QCOW,
    QED,
    VD,
    ISCSI,
    DMG,
    Raw,
}

#[repr(C)]
pub struct VDImage {
    pub pszFilename: String,
    pub u32ImageType: DiskType,
    pub cbSize: u64,
    pub cbSector: u32,
    pub fReadOnly: bool,
    pub u32Flags: u32,
}

pub struct VDiskImage {
    image: VDImage,
    data: Vec<u8>,
}

impl VDiskImage {
    pub fn open(filename: &str) -> VBoxResult<Self> {
        Ok(Self {
            image: VDImage {
                pszFilename: filename.to_string(),
                u32ImageType: DiskType::VD,
                cbSize: 0,
                cbSector: 512,
                fReadOnly: false,
                u32Flags: 0,
            },
            data: Vec::new(),
        })
    }
}

impl DiskImage for VDiskImage {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>> {
        let offset = (lba * 512) as usize;
        if offset >= self.data.len() {
            return Ok(vec![0u8; 512]);
        }
        let end = core::cmp::min(offset + 512, self.data.len());
        Ok(self.data[offset..end].to_vec())
    }

    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()> {
        let offset = (lba * 512) as usize;
        if offset + data.len() > self.data.len() {
            self.data.resize(offset + data.len(), 0);
        }
        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    fn read_sectors(&self, lba: u64, count: u32) -> VBoxResult<Vec<Vec<u8>>> {
        (0..count).map(|i| self.read_sector(lba + i as u64)).collect()
    }

    fn write_sectors(&mut self, lba: u64, data: &[Vec<u8>]) -> VBoxResult<()> {
        for (i, sector) in data.iter().enumerate() {
            self.write_sector(lba + i as u64, sector)?;
        }
        Ok(())
    }

    fn get_size(&self) -> u64 {
        self.image.cbSize
    }

    fn get_sector_size(&self) -> u32 {
        self.image.cbSector
    }

    fn get_type(&self) -> DiskType {
        self.image.u32ImageType
    }
}
