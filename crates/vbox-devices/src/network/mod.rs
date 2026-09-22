pub mod e1000;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Network device trait.
pub trait NetworkDevice: Send + Sync {
    /// Attaches the device to a network.
    fn attach(&mut self) -> Result<(), NetworkError>;
    /// Detaches the device from a network.
    fn detach(&mut self) -> Result<(), NetworkError>;
    /// Resets the device.
    fn reset(&mut self);
    /// Sends a packet.
    fn send(&self, packet: &[u8]) -> Result<(), NetworkError>;
    /// Receives a packet.
    fn recv(&self) -> Result<Option<Vec<u8>>, NetworkError>;
}

/// Network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfiguration {
    /// MAC address.
    pub mac_address: [u8; 6],
    /// IP address.
    pub ip_address: [u8; 4],
    /// Subnet mask.
    pub subnet_mask: [u8; 4],
    /// Gateway IP address.
    pub gateway: [u8; 4],
    /// DNS server IP address.
    pub dns: [u8; 4],
    /// Network name.
    pub network_name: String,
}

impl Default for NetworkConfiguration {
    fn default() -> Self {
        Self {
            mac_address: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ip_address: [10, 0, 2, 15],
            subnet_mask: [255, 255, 255, 0],
            gateway: [10, 0, 2, 2],
            dns: [10, 0, 2, 3],
            network_name: String::from("intnet"),
        }
    }
}

/// Internal network switch for host-only networking.
#[derive(Debug, Default)]
pub struct IntNetSwitch {
    /// Connected devices by MAC address.
    pub connected_devices: Vec<NetworkConfiguration>,
    /// Switch name.
    pub name: String,
}

impl IntNetSwitch {
    /// Creates a new internal network switch.
    pub fn new(name: &str) -> Self {
        Self {
            connected_devices: Vec::new(),
            name: name.to_string(),
        }
    }

    /// Connects a device to the switch.
    pub fn connect_device(&mut self, config: NetworkConfiguration) {
        self.connected_devices.push(config);
    }
}

/// NAT (user-mode network stack) for external network access.
#[derive(Debug, Default)]
pub struct SlirpNat {
    /// NAT gateway address.
    pub gateway: [u8; 4],
    /// DNS proxy address.
    pub dns_proxy: [u8; 4],
    /// Port forwarding rules.
    pub port_forwards: Vec<PortForward>,
}

/// Port forwarding rule for NAT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    /// Host port.
    pub host_port: u16,
    /// Guest port.
    pub guest_port: u16,
    /// Guest IP.
    pub guest_ip: [u8; 4],
    /// Protocol (TCP/UDP).
    pub protocol: String,
}

impl SlirpNat {
    /// Creates a new NAT instance.
    pub fn new() -> Self {
        Self {
            gateway: [10, 0, 2, 2],
            dns_proxy: [10, 0, 2, 3],
            port_forwards: Vec::new(),
        }
    }

    /// Adds a port forwarding rule.
    pub fn add_port_forward(&mut self, pf: PortForward) {
        self.port_forwards.push(pf);
    }
}

/// Network service managing all network devices.
pub struct NetworkService {
    /// Registered network devices.
    pub devices: Vec<Box<dyn NetworkDevice>>,
    /// Internal network switch.
    pub intnet_switch: IntNetSwitch,
    /// NAT for user-mode networking.
    pub slirp_nat: SlirpNat,
}

impl NetworkService {
    /// Creates a new network service.
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            intnet_switch: IntNetSwitch::new("intnet"),
            slirp_nat: SlirpNat::new(),
        }
    }

    /// Initializes the network subsystem.
    pub fn init(&mut self) -> Result<(), NetworkError> {
        Ok(())
    }

    /// Registers a network device.
    pub fn register_device(&mut self, device: Box<dyn NetworkDevice>) {
        self.devices.push(device);
    }
}

impl Default for NetworkService {
    fn default() -> Self {
        Self::new()
    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_service_new() {
        let service = NetworkService::new();
        assert!(service.devices.is_empty());
    }

    #[test]
    fn test_intnet_switch() {
        let mut switch = IntNetSwitch::new("test");
        assert_eq!(switch.name, "test");
    }
}
