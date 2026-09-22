use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

use crate::machine::Machine;
use crate::machine::MachineState;

/// Display device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u32,
    pub monitor_count: u32,
    pub video_ram: u32,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            bits_per_pixel: 32,
            monitor_count: 1,
            video_ram: 16,
        }
    }
}

/// Audio device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audio {
    pub enabled: bool,
    pub driver: String,
    pub output_enabled: bool,
    pub input_enabled: bool,
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            enabled: false,
            driver: "default".to_string(),
            output_enabled: true,
            input_enabled: false,
        }
    }
}

/// Network device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub adapters: Vec<NetworkAdapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAdapter {
    pub slot: u32,
    pub enabled: bool,
    pub mac: [u8; 6],
    pub cable_connected: bool,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            adapters: vec![NetworkAdapter {
                slot: 0,
                enabled: true,
                mac: [0u8; 6],
                cable_connected: true,
            }],
        }
    }
}

/// Storage controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    pub controllers: Vec<StorageController>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageController {
    pub name: String,
    pub bus_type: String,
    pub controller_type: String,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            controllers: Vec::new(),
        }
    }
}

/// USB controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usb {
    pub enabled: bool,
    pub controller_type: String,
    pub ehci_enabled: bool,
    pub xhci_enabled: bool,
}

impl Default for Usb {
    fn default() -> Self {
        Self {
            enabled: false,
            controller_type: "EHCI".to_string(),
            ehci_enabled: true,
            xhci_enabled: false,
        }
    }
}

/// Console interface for VM interaction.
pub struct Console {
    pub p_machine: Arc<Machine>,
    pub p_vm: Option<Arc<VM>>,
    pub p_display: Display,
    pub p_audio: Audio,
    pub p_network: Network,
    pub p_storage: Storage,
    pub p_usb: Usb,
}

/// Virtual Machine runtime handle.
pub struct VM {
    pub uuid: Uuid,
    pub state: MachineState,
    pub memory_size: u64,
    pub cpu_count: u32,
}

impl VM {
    pub fn new(uuid: Uuid, memory_size: u64, cpu_count: u32) -> Self {
        Self {
            uuid,
            state: MachineState::PoweredOff,
            memory_size,
            cpu_count,
        }
    }
}

#[derive(Error, Debug)]
pub enum ConsoleError {
    #[error("Console already attached")]
    AlreadyAttached,
    #[error("Console not attached")]
    NotAttached,
    #[error("VM not running")]
    VmNotRunning,
}

impl Console {
    pub fn new(machine: Arc<Machine>) -> Console {
        Console {
            p_machine: machine,
            p_vm: None,
            p_display: Display::default(),
            p_audio: Audio::default(),
            p_network: Network::default(),
            p_storage: Storage::default(),
            p_usb: Usb::default(),
        }
    }

    pub fn attach(&mut self) -> VBoxResult<()> {
        Ok(())
    }

    pub fn detach(&mut self) -> VBoxResult<()> {
        Ok(())
    }

    pub fn power_up(&mut self) -> VBoxResult<()> {
        self.p_machine.start()?;
        Ok(())
    }

    pub fn power_down(&mut self) -> VBoxResult<()> {
        self.p_machine.power_off()?;
        Ok(())
    }

    pub fn get_display(&self) -> &Display {
        &self.p_display
    }

    pub fn get_audio(&self) -> &Audio {
        &self.p_audio
    }

    pub fn get_network(&self) -> &Network {
        &self.p_network
    }

    pub fn get_storage(&self) -> &Storage {
        &self.p_storage
    }

    pub fn get_usb(&self) -> &Usb {
        &self.p_usb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_new() {
        let machine = Arc::new(crate::machine::Machine::new(
            "test",
            uuid::Uuid::new_v4(),
            Arc::new(crate::virtualbox::VirtualBox::new()),
        ));
        let console = Console::new(machine);
        assert_eq!(console.p_display.width, 1024);
    }
}
