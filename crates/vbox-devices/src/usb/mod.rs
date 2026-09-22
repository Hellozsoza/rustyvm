use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// USB host controller.
pub struct UsbController {
    /// Controller type.
    pub controller_type: UsbControllerType,
    /// Number of ports.
    pub ports: u8,
    /// Connected devices.
    pub devices: Vec<UsbDevice>,
}

/// USB controller type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbControllerType {
    UHCI,
    OHCI,
    EHCI,
    XHCI,
}

/// USB device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    /// Device address on the bus.
    pub address: u8,
    /// Vendor ID.
    pub vendor_id: u16,
    /// Product ID.
    pub product_id: u16,
    /// Device class.
    pub class: u8,
    /// Number of configurations.
    pub num_configurations: u8,
}

impl UsbController {
    /// Creates a new USB controller.
    pub fn new(controller_type: UsbControllerType, ports: u8) -> Self {
        Self {
            controller_type,
            ports,
            devices: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_controller_new() {
        let ctrl = UsbController::new(UsbControllerType::XHCI, 8);
        assert_eq!(ctrl.ports, 8);
    }
}
