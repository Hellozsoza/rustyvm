pub use crate::{DiskImage, DiskType, VDImage, VDiskImage, VBoxResult, VBoxError};

pub struct DiskImageInfo {
    pub format: DiskType,
    pub size: u64,
    pub sector_size: u32,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskFormat {
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

impl From<DiskFormat> for DiskType {
    fn from(f: DiskFormat) -> Self {
        match f {
            DiskFormat::VDI => DiskType::VDI,
            DiskFormat::VMDK => DiskType::VMDK,
            DiskFormat::VHD => DiskType::VHD,
            DiskFormat::VHDX => DiskType::VHDX,
            DiskFormat::QCOW => DiskType::QCOW,
            DiskFormat::QED => DiskType::QED,
            DiskFormat::VD => DiskType::VD,
            DiskFormat::ISCSI => DiskType::ISCSI,
            DiskFormat::DMG => DiskType::DMG,
            DiskFormat::Raw => DiskType::Raw,
        }
    }
}
