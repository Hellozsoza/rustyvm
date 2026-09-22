use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Ethernet transmit descriptor.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TxDescriptor {
    /// Physical address of the descriptor/data.
    pub addr: u64,
    /// Length of data.
    pub length: u16,
    /// Command flags.
    pub cmd: u16,
    /// Status flags.
    pub status: u16,
    /// VLAN tag.
    pub vlan: u16,
}

impl Default for TxDescriptor {
    fn default() -> Self {
        Self {
            addr: 0,
            length: 0,
            cmd: 0,
            status: 0,
            vlan: 0,
        }
    }
}

/// Ethernet receive descriptor.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RxDescriptor {
    /// Physical address of the descriptor/data.
    pub addr: u64,
    /// Length of data.
    pub length: u16,
    /// Status flags.
    pub status: u16,
    /// Error flags.
    pub error: u16,
    /// VLAN tag.
    pub vlan: u16,
    /// RSS hash result.
    pub rss: u32,
}

impl Default for RxDescriptor {
    fn default() -> Self {
        Self {
            addr: 0,
            length: 0,
            status: 0,
            error: 0,
            vlan: 0,
            rss: 0,
        }
    }
}

/// Network backend trait for sending and receiving packets.
pub trait NetworkBackend: Send + Sync {
    /// Sends a packet through the backend.
    fn send(&self, packet: &[u8]) -> Result<(), NetworkError>;
    /// Receives a packet from the backend.
    fn recv(&self) -> Result<Option<Vec<u8>>, NetworkError>;
}

/// Error type for network operations.
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Backend error: {0}")]
    Backend(String),
    #[error("Link not connected")]
    LinkDown,
    #[error("Buffer overflow")]
    BufferOverflow,
    #[error("Invalid descriptor")]
    InvalidDescriptor,
}

/// E1000 Ethernet device emulation (Intel 82540EM).
pub struct E1000Device {
    /// MAC address (6 bytes).
    pub mac_address: [u8; 6],
    /// MMIO base address.
    pub mmio_base: u64,
    /// IO base port.
    pub io_base: u16,
    /// Receive buffer.
    pub rx_buffer: Vec<u8>,
    /// Transmit buffer.
    pub tx_buffer: Vec<u8>,
    /// Transmit descriptor ring (4096 entries).
    pub tx_desc: Box<[TxDescriptor; 4096]>,
    /// Receive descriptor ring (4096 entries).
    pub rx_desc: Box<[RxDescriptor; 4096]>,
    /// Control register.
    pub ctl_reg: u32,
    /// Status register.
    pub status_reg: u32,
    /// Whether link is up.
    pub is_link_up: bool,
    /// Network backend.
    pub backend: Option<Box<dyn NetworkBackend>>,
}

impl E1000Device {
    /// Creates a new E1000 device with default MAC address.
    pub fn new() -> Self {
        let mac: [u8; 6] = [0x00, 0x0C, 0x29, 0x00, 0x00, 0x01];
        Self {
            mac_address: mac,
            mmio_base: 0xE0000000,
            io_base: 0x1000,
            rx_buffer: Vec::new(),
            tx_buffer: Vec::new(),
            tx_desc: vec![TxDescriptor::default(); 4096].into_boxed_slice(),
            rx_desc: vec![RxDescriptor::default(); 4096].into_boxed_slice(),
            ctl_reg: 0,
            status_reg: 0,
            is_link_up: false,
            backend: None,
        }
    }

    /// Reads from device MMIO space.
    pub fn mmio_read(&self, offset: u64) -> u32 {
        match offset {
            0x0000 => self.status_reg,
            0x0004 => self.ctl_reg,
            _ => 0,
        }
    }

    /// Writes to device MMIO space.
    pub fn mmio_write(&mut self, offset: u64, value: u32) {
        match offset {
            0x0000 => self.status_reg = value,
            0x0004 => self.ctl_reg = value,
            _ => {}
        }
    }

    /// Reads from an IO port.
    pub fn io_read(&self, port: u16) -> u8 {
        match port {
            0x0000 => (self.status_reg & 0xFF) as u8,
            _ => 0,
        }
    }

    /// Writes to an IO port.
    pub fn io_write(&mut self, port: u16, value: u8) {
        match port {
            0x0000 => self.status_reg = (self.status_reg & 0xFFFFFF00) | (value as u32),
            _ => {}
        }
    }

    /// Sends a packet to the network backend.
    pub fn send_packet(&mut self, packet: &[u8]) -> Result<(), NetworkError> {
        if !self.is_link_up {
            return Err(NetworkError::LinkDown);
        }
        if let Some(ref backend) = self.backend {
            backend.send(packet)?;
        }
        self.tx_buffer.extend_from_slice(packet);
        Ok(())
    }

    /// Receives a packet from the network backend.
    pub fn recv_packet(&mut self) -> Result<Option<Vec<u8>>, NetworkError> {
        if let Some(ref backend) = self.backend {
            backend.recv()
        } else {
            Ok(None)
        }
    }

    /// Resets the E1000 device to its initial state.
    pub fn reset(&mut self) {
        self.ctl_reg = 0;
        self.status_reg = 0;
        self.is_link_up = false;
        self.tx_buffer.clear();
        self.rx_buffer.clear();
        for desc in self.tx_desc.iter_mut() {
            *desc = TxDescriptor::default();
        }
        for desc in self.rx_desc.iter_mut() {
            *desc = RxDescriptor::default();
        }
    }
}

impl Default for E1000Device {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e1000_new() {
        let e1000 = E1000Device::new();
        assert_eq!(e1000.mac_address, [0x00, 0x0C, 0x29, 0x00, 0x00, 0x01]);
        assert!(!e1000.is_link_up);
    }

    #[test]
    fn test_e1000_reset() {
        let mut e1000 = E1000Device::new();
        e1000.ctl_reg = 0xFFFF;
        e1000.is_link_up = true;
        e1000.reset();
        assert_eq!(e1000.ctl_reg, 0);
        assert!(!e1000.is_link_up);
    }
}
