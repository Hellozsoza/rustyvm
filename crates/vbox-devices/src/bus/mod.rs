pub mod pci;
pub mod chipset;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// PCI BAR type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PciBar {
    /// 32-bit memory-mapped I/O bar.
    Memory32(u32),
    /// 64-bit memory-mapped I/O bar.
    Memory64(u64),
    /// I/O port bar.
    Io(u16),
}

impl Default for PciBar {
    fn default() -> Self {
        PciBar::Io(0)
    }
}

/// PCI device header structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PciHeader {
    /// Vendor ID.
    pub vendor_id: u16,
    /// Device ID.
    pub device_id: u16,
    /// Command register.
    pub command: u16,
    /// Status register.
    pub status: u16,
    /// Revision ID.
    pub revision_id: u8,
    /// Class code.
    pub class_code: u8,
    /// Subclass code.
    pub subclass: u8,
    /// Programming interface.
    pub prog_if: u8,
    /// BIST.
    pub bist: u8,
    /// Base address registers.
    pub bars: [PciBar; 6],
    /// Cardbus CIS pointer.
    pub cis_pointer: u32,
    /// Subsystem vendor ID.
    pub subsystem_vendor_id: u16,
    /// Subsystem ID.
    pub subsystem_id: u16,
    /// Expansion ROM base address.
    pub expansion_rom_base: u32,
    /// Capabilities pointer.
    pub capabilities_pointer: u8,
    /// Interrupt line.
    pub interrupt_line: u8,
    /// Interrupt pin.
    pub interrupt_pin: u8,
}

impl Default for PciHeader {
    fn default() -> Self {
        Self {
            vendor_id: 0,
            device_id: 0,
            command: 0,
            status: 0,
            revision_id: 0,
            class_code: 0,
            subclass: 0,
            prog_if: 0,
            bist: 0,
            bars: [PciBar::Io(0); 6],
            cis_pointer: 0,
            subsystem_vendor_id: 0,
            subsystem_id: 0,
            expansion_rom_base: 0,
            capabilities_pointer: 0,
            interrupt_line: 0,
            interrupt_pin: 0,
        }
    }
}

/// PCI device trait.
pub trait PciDevice: Send + Sync {
    /// Reads from the PCI configuration space.
    fn read_config(&self, offset: u8, width: u8) -> u32;
    /// Writes to the PCI configuration space.
    fn write_config(&mut self, offset: u8, width: u8, value: u32);
    /// Returns the BAR for the given index.
    fn bar(&self, index: usize) -> Option<PciBar>;
    /// Returns the PCI header.
    fn header(&self) -> &PciHeader;
}

/// PCI bus structure.
pub struct PciBus {
    /// PCI devices indexed by device number (0-255).
    pub devices: [Option<Box<dyn PciDevice>>; 256],
    /// Bus number.
    pub bus_number: u8,
}

impl PciBus {
    /// Creates a new PCI bus.
    pub fn new(bus_number: u8) -> Self {
        Self {
            devices: Default::default(),
            bus_number,
        }
    }

    /// Registers a PCI device on the bus.
    pub fn register_device(&mut self, device_num: usize, device: Box<dyn PciDevice>) {
        if device_num < 256 {
            self.devices[device_num] = Some(device);
        }
    }

    /// Reads from PCI configuration space.
    pub fn read_config_space(&self, device_num: usize, offset: u8, width: u8) -> u32 {
        if let Some(ref dev) = self.devices.get(device_num).and_then(|d| d.as_ref()) {
            dev.read_config(offset, width)
        } else {
            0xFFFFFFFF
        }
    }

    /// Writes to PCI configuration space.
    pub fn write_config_space(&mut self, device_num: usize, offset: u8, width: u8, value: u32) {
        if let Some(ref mut dev) = self.devices.get_mut(device_num).and_then(|d| d.as_mut()) {
            dev.write_config(offset, width, value);
        }
    }
}

impl Default for PciBus {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pci_bus_new() {
        let bus = PciBus::new(0);
        assert_eq!(bus.bus_number, 0);
    }
}
