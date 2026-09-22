use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// NVMe Namespace structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvmeNamespace {
    /// Namespace ID.
    pub ns_id: u32,
    /// Namespace size in logical blocks.
    pub size: u64,
    /// Namespace capabilities.
    pub capabilities: u32,
    /// Format of the namespace.
    pub format: NvmeFormat,
}

/// NVMe namespace format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NvmeFormat {
    NsFmt0,
    NsFmt1,
    NsFmt2,
    NsFmt3,
}

impl Default for NvmeNamespace {
    fn default() -> Self {
        Self {
            ns_id: 1,
            size: 0,
            capabilities: 0,
            format: NvmeFormat::NsFmt0,
        }
    }
}

/// NVMe Controller structure.
pub struct NvmeController {
    /// Controller ID.
    pub controller_id: u16,
    /// Vendor ID.
    pub vendor_id: u16,
    /// Submission queues.
    pub submission_queues: Vec<NvmeQueue>,
    /// Completion queues.
    pub completion_queues: Vec<NvmeQueue>,
    /// Namespaces managed by this controller.
    pub namespaces: Vec<NvmeNamespace>,
    /// MMIO base address.
    pub mmio_base: u64,
    /// Number of I/O queues.
    pub io_queue_count: u32,
    /// Controller status.
    pub status: NvmeControllerStatus,
    /// Serial number.
    pub serial_number: String,
    /// Model number.
    pub model_number: String,
}

/// NVMe queue structure.
#[derive(Debug, Default)]
pub struct NvmeQueue {
    /// Queue ID.
    pub id: u16,
    /// Number of entries.
    pub size: u16,
    /// Queue doorbell register offset.
    pub doorbell_offset: u64,
    /// Queue entries.
    pub entries: Vec<u64>,
}

/// NVMe controller status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NvmeControllerStatus {
    Unknown = 0,
    Ready = 1,
    Offline = 2,
    Faulted = 3,
}

impl Default for NvmeControllerStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl NvmeController {
    /// Creates a new NVMe controller.
    pub fn new() -> Self {
        Self {
            controller_id: 0,
            vendor_id: 0x80EE,
            submission_queues: Vec::new(),
            completion_queues: Vec::new(),
            namespaces: Vec::new(),
            mmio_base: 0xD0000000,
            io_queue_count: 32,
            status: NvmeControllerStatus::Unknown,
            serial_number: String::from("VB1234-56789"),
            model_number: String::from("ORCL-VBOX-NVME-VER12"),
        }
    }

    /// Submits a command to the NVMe controller.
    pub fn submit_command(&mut self, command: u8, namespace_id: u32, lba: u64, count: u32) -> Result<Vec<u8>, NvmeError> {
        match command {
            0x02 => self.read(namespace_id, lba, count),
            0x01 => self.write(namespace_id, lba, count),
            0x06 => self.identify(namespace_id),
            _ => Err(NvmeError::UnsupportedCommand(command)),
        }
    }

    /// Processes a READ command.
    fn read(&mut self, namespace_id: u32, lba: u64, count: u32) -> Result<Vec<u8>, NvmeError> {
        let _ = namespace_id;
        let _ = lba;
        let _ = count;
        Ok(vec![0u8; (count as usize) * 512])
    }

    /// Processes a WRITE command.
    fn write(&mut self, namespace_id: u32, lba: u64, count: u32) -> Result<Vec<u8>, NvmeError> {
        let _ = namespace_id;
        let _ = lba;
        let _ = count;
        Ok(vec![0u8; (count as usize) * 512])
    }

    /// Processes an IDENTIFY command.
    fn identify(&mut self, namespace_id: u32) -> Result<Vec<u8>, NvmeError> {
        let _ = namespace_id;
        Ok(vec![0u8; 4096])
    }
}

impl Default for NvmeController {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for NVMe operations.
#[derive(Debug, thiserror::Error)]
pub enum NvmeError {
    #[error("Unsupported command: 0x{0:X}")]
    UnsupportedCommand(u8),
    #[error("Namespace not found: {0}")]
    NamespaceNotFound(u32),
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Queue full")]
    QueueFull,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvme_controller_new() {
        let ctrl = NvmeController::new();
        assert_eq!(ctrl.vendor_id, 0x80EE);
        assert_eq!(ctrl.io_queue_count, 32);
    }
}
