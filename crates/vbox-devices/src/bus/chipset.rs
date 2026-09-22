use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// PIIX3 chipset with SMBus, IDE, and ISA bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIIX3Chipset {
    /// SMBus controller.
    pub smb_controller: SmBusController,
    /// IDE controller.
    pub ide_controller: IdeController,
    /// ISA bridge.
    pub isa_bridge: IsaBridge,
    /// Southbridge revision.
    pub revision: u8,
}

/// SMBus controller.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SmBusController {
    /// SMBus base address.
    pub base_address: u32,
    /// SMBus interrupt line.
    pub irq: u8,
    /// SMBus status.
    pub status: u8,
}

/// IDE controller.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdeController {
    /// IDE base address.
    pub base_address: u32,
    /// IDE alternative base address.
    pub alt_base_address: u32,
    /// IDE IRQ.
    pub irq: u8,
    /// DMA channel.
    pub dma_channel: u8,
}

/// ISA bridge.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IsaBridge {
    /// ISA base address.
    pub base_address: u32,
    /// IRQ routing.
    pub irq_routing: [u8; 16],
    /// DMA routing.
    pub dma_routing: [u8; 8],
}

impl PIIX3Chipset {
    /// Creates a new PIIX3 chipset.
    pub fn new() -> Self {
        Self {
            smb_controller: SmBusController::default(),
            ide_controller: IdeController::default(),
            isa_bridge: IsaBridge::default(),
            revision: 0,
        }
    }
}

impl Default for PIIX3Chipset {
    fn default() -> Self {
        Self::new()
    }
}

/// ICH9 chipset with more modern features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ich9Chipset {
    /// SMBus controller.
    pub smb_controller: SmBusController,
    /// AHCI controller.
    pub ahci_controller: AhciController,
    /// PCI Express root port.
    pub pcie_root_port: PcieRootPort,
    /// HD Audio controller.
    pub hd_audio: HdAudioController,
    /// ISA bridge.
    pub isa_bridge: IsaBridge,
    /// LPC controller.
    pub lpc_controller: LpcController,
}

/// PCI Express root port.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PcieRootPort {
    /// PCIe base address.
    pub base_address: u64,
    /// Number of lanes.
    pub lanes: u8,
    /// PCIe speed.
    pub speed: PcieSpeed,
}

/// PCIe speed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PcieSpeed {
    Gen1 = 1,
    Gen2 = 2,
    Gen3 = 3,
}

impl Default for PcieSpeed {
    fn default() -> Self {
        Self::Gen1
    }
}

/// AHCI controller in ICH9.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AhciController {
    /// AHCI base address.
    pub base_address: u64,
    /// Number of ports.
    pub ports: u8,
}

/// HD Audio controller.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdAudioController {
    /// HD Audio base address.
    pub base_address: u32,
    /// HD Audio IRQ.
    pub irq: u8,
}

/// LPC (Low Pin Count) controller.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LpcController {
    /// LPC base address.
    pub base_address: u32,
    /// SuperIO configuration.
    pub superio: bool,
}

impl Ich9Chipset {
    /// Creates a new ICH9 chipset.
    pub fn new() -> Self {
        Self {
            smb_controller: SmBusController::default(),
            ahci_controller: AhciController::default(),
            pcie_root_port: PcieRootPort::default(),
            hd_audio: HdAudioController::default(),
            isa_bridge: IsaBridge::default(),
            lpc_controller: LpcController::default(),
        }
    }
}

impl Default for Ich9Chipset {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piiix3_new() {
        let chip = PIIX3Chipset::new();
        assert_eq!(chip.revision, 0);
    }

    #[test]
    fn test_ich9_new() {
        let chip = Ich9Chipset::new();
        assert!(!chip.lpc_controller.superio);
    }
}
