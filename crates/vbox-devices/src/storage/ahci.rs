use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// AHCI Command List Entry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AhciCmdListEntry {
    /// PRDT length (number of PRD entries).
    pub prdtl: u16,
    /// Command FIS length in DWORDS.
    pub cfl: u8,
    /// ATAPI flag.
    pub a: u8,
    /// Write flag.
    pub w: u8,
    /// Prefetchable flag.
    pub p: u8,
    /// Reset flag.
    pub r: u8,
    /// BIST flag.
    pub b: u8,
    /// Clear busy upon R_OK.
    pub c: u8,
    /// Next command list entry physical address.
    pub next: u32,
    /// PRD byte count transferred.
    pub prdbc: u32,
    /// Command table base address.
    pub ctba: u64,
}

impl Default for AhciCmdListEntry {
    fn default() -> Self {
        Self {
            prdtl: 0,
            cfl: 0,
            a: 0,
            w: 0,
            p: 0,
            r: 0,
            b: 0,
            c: 0,
            next: 0,
            prdbc: 0,
            ctba: 0,
        }
    }
}

/// AHCI Frame Information Structure (FIS).
#[derive(Debug, Clone, Copy)]
pub struct AhciFis {
    /// FIS type.
    pub fis_type: u8,
    /// FIS data.
    pub data: [u8; 255],
}

impl Default for AhciFis {
    fn default() -> Self {
        Self {
            fis_type: 0,
            data: [0; 255],
        }
    }
}

/// AHCI Port structure.
#[derive(Debug, Default)]
pub struct AhciPort {
    /// Command Issue Block.
    pub cib: [u8; 256],
    /// Command list entries (32 slots).
    pub cmd_list: [AhciCmdListEntry; 32],
    /// Frame Information Structures.
    pub fis: [AhciFis; 256],
    /// SATA status register.
    pub sata_status: u32,
    /// Task file registers.
    pub task_file: [u8; 8],
    /// Whether the port is active.
    pub active: bool,
}

impl AhciPort {
    /// Submits a SATA command to the port.
    pub fn submit_command(&mut self, command: u8, lba: u64, count: u32, buffer: &[u8]) -> Result<(), StorageError> {
        match command {
            0x25 => self.read_dma(lba, count, buffer),
            0x35 => self.write_dma(lba, count, buffer),
            0xEC => self.identify_device(buffer),
            _ => Err(StorageError::UnsupportedCommand(command)),
        }
    }

    /// Processes a READ DMA command.
    fn read_dma(&mut self, lba: u64, count: u32, buffer: &[u8]) -> Result<(), StorageError> {
        let _ = lba;
        let _ = count;
        let _ = buffer;
        Ok(())
    }

    /// Processes a WRITE DMA command.
    fn write_dma(&mut self, lba: u64, count: u32, buffer: &[u8]) -> Result<(), StorageError> {
        let _ = lba;
        let _ = count;
        let _ = buffer;
        Ok(())
    }

    /// Processes an IDENTIFY DEVICE command.
    fn identify_device(&mut self, buffer: &[u8]) -> Result<(), StorageError> {
        let _ = buffer;
        Ok(())
    }
}

/// Storage backend trait for reading/writing sectors.
pub trait StorageBackend: Send + Sync {
    /// Reads sectors from the storage device.
    fn read_sectors(&self, lba: u64, count: u32) -> Result<Vec<u8>, StorageError>;
    /// Writes sectors to the storage device.
    fn write_sectors(&mut self, lba: u64, count: u32, data: &[u8]) -> Result<(), StorageError>;
}

/// Error type for storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Unsupported command: 0x{0:X}")]
    UnsupportedCommand(u8),
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Device not ready")]
    NotReady,
    #[error("Invalid LBA: {0}")]
    InvalidLba(u64),
}

/// AHCI controller structure.
pub struct AhciController {
    /// SATA ports (up to 8).
    pub ports: [AhciPort; 8],
    /// MMIO base address.
    pub mmio_base: u64,
    /// Host capabilities.
    pub capabilities: u32,
    /// Global Host Control register.
    pub ghc: u32,
    /// Interrupt Status register.
    pub is_reg: u32,
    /// Number of ports.
    pub port_count: u8,
}

impl AhciController {
    /// Creates a new AHCI controller.
    pub fn new() -> Self {
        let mut ports = std::array::from_fn(|_| AhciPort::default());
        Self {
            ports,
            mmio_base: 0xE0000000,
            capabilities: 0x00000000,
            ghc: 0,
            is_reg: 0,
            port_count: 8,
        }
    }

    /// Reads from AHCI MMIO space.
    pub fn mmio_read(&self, offset: u64) -> u32 {
        match offset {
            0x00 => self.capabilities,
            0x08 => self.ghc,
            0x10 => self.is_reg,
            _ => 0,
        }
    }

    /// Writes to AHCI MMIO space.
    pub fn mmio_write(&mut self, offset: u64, value: u32) {
        match offset {
            0x08 => self.ghc = value,
            0x10 => self.is_reg = value,
            _ => {}
        }
    }
}

/// AHCI device combining controller and storage backend.
pub struct AhciDevice<B: StorageBackend> {
    /// The AHCI controller.
    pub controller: AhciController,
    /// Storage backend.
    pub backend: B,
}

impl<B: StorageBackend> AhciDevice<B> {
    /// Creates a new AHCI device with the given backend.
    pub fn new(backend: B) -> Self {
        Self {
            controller: AhciController::new(),
            backend,
        }
    }
}

impl Default for AhciController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ahci_controller_new() {
        let ctrl = AhciController::new();
        assert_eq!(ctrl.port_count, 8);
        assert_eq!(ctrl.mmio_base, 0xE0000000);
    }
}
