use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::{RwLock, Mutex};
use std::sync::Arc;
use uuid::Uuid;
use serde_json;

use crate::com::IID;
use crate::virtualbox::VirtualBox;
use crate::settings::VmSettings;
use crate::console::Console;
use crate::medium::Medium;

/// Machine state enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum MachineState {
    Null = 0,
    PoweredOff = 1,
    Saved = 2,
    Running = 3,
    Paused = 4,
    Stuck = 5,
}

impl MachineState {
    pub fn is_power_off(&self) -> bool {
        matches!(self, MachineState::PoweredOff | MachineState::Null)
    }
    pub fn is_running(&self) -> bool {
        matches!(self, MachineState::Running)
    }
}

/// VM settings structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmSettings {
    pub name: String,
    pub description: String,
    pub memory_size: u32,
    pub cpu_count: u32,
    pub os_type: String,
    pub firmware_type: FirmwareType,
    pub boot_order: Vec<u32>,
    pub display_settings: DisplaySettings,
    pub network_adapters: Vec<NetworkAdapterSettings>,
    pub storage_controllers: Vec<StorageControllerSettings>,
    pub extra_data: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FirmwareType {
    Bios,
    Uefi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub video_ram: u32,
    pub monitor_count: u32,
    pub triple_buffer: bool,
    pub accelerate_2d: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAdapterSettings {
    pub slot: u32,
    pub enabled: bool,
    pub adapter_type: String,
    pub_mac: [u8; 6],
    pub cable_connected: bool,
    pub speed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageControllerSettings {
    pub name: String,
    pub controller_type: String,
    pub bus_type: String,
    pub_max_device_count: u32,
    pub_use_host_io_cache: bool,
}

impl Default for VmSettings {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            memory_size: 1024,
            cpu_count: 1,
            os_type: "Other".to_string(),
            firmware_type: FirmwareType::Bios,
            boot_order: vec![1, 2, 3],
            display_settings: DisplaySettings {
                video_ram: 16,
                monitor_count: 1,
                triple_buffer: false,
                accelerate_2d: true,
            },
            network_adapters: Vec::new(),
            storage_controllers: Vec::new(),
            extra_data: serde_json::Map::new(),
        }
    }
}

impl VmSettings {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn deserialize(data: &str) -> VBoxResult<VmSettings> {
        let settings: VmSettings = serde_json::from_str(data)?;
        Ok(settings)
    }

    pub fn default_for(os_type: &str) -> VmSettings {
        let mut settings = VmSettings::default();
        settings.os_type = os_type.to_string();
        settings.memory_size = match os_type {
            "Windows10_64" => 4096,
            "Windows2019_64" => 8192,
            "Linux_64" => 2048,
            "Ubuntu_64" => 2048,
            _ => 1024,
        };
        settings
    }
}

/// Snapshot structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub timestamp: i64,
    pub state: MachineState,
    pub data: Vec<u8>,
}

/// Machine structure.
pub struct Machine {
    pub uuid: Uuid,
    pub name: String,
    pub settings: VmSettings,
    pub p_virtualbox: Arc<VirtualBox>,
    pub state: MachineState,
    pub p_console: Option<Arc<Console>>,
    pub is_registered: bool,
    pub session_lock: Mutex<bool>,
}

#[derive(Error, Debug)]
pub enum MachineError {
    #[error("Machine already running")]
    AlreadyRunning,
    #[error("Machine not powered off")]
    NotPoweredOff,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Machine {
    pub fn new(name: &str, uuid: Uuid, virtualbox: Arc<VirtualBox>) -> Machine {
        Machine {
            uuid,
            name: name.to_string(),
            settings: VmSettings::default(),
            p_virtualbox: virtualbox,
            state: MachineState::PoweredOff,
            p_console: None,
            is_registered: false,
            session_lock: Mutex::new(false),
        }
    }

    pub fn start(&mut self) -> VBoxResult<()> {
        if self.state == MachineState::Running {
            return Err(VBoxError::AlreadyExists);
        }
        self.state = MachineState::Running;
        Ok(())
    }

    pub fn power_off(&mut self) -> VBoxResult<()> {
        if self.state != MachineState::Running && self.state != MachineState::Paused {
            return Err(VBoxError::BadState);
        }
        self.state = MachineState::PoweredOff;
        Ok(())
    }

    pub fn pause(&mut self) -> VBoxResult<()> {
        if self.state != MachineState::Running {
            return Err(VBoxError::BadState);
        }
        self.state = MachineState::Paused;
        Ok(())
    }

    pub fn save_state(&self) -> VBoxResult<Vec<u8>> {
        let data = serde_json::to_vec(&self.settings)?;
        Ok(data)
    }

    pub fn restore_state(&mut self, data: &[u8]) -> VBoxResult<()> {
        let settings: VmSettings = serde_json::from_slice(data)?;
        self.settings = settings;
        Ok(())
    }

    pub fn get_state(&self) -> MachineState {
        self.state
    }

    pub fn get_settings(&self) -> &VmSettings {
        &self.settings
    }

    pub fn set_storage_controller(&mut self, name: &str, controller_type: &str) -> VBoxResult<()> {
        let settings = StorageControllerSettings {
            name: name.to_string(),
            controller_type: controller_type.to_string(),
            bus_type: "SATA".to_string(),
            max_device_count: 30,
            use_host_io_cache: true,
        };
        self.settings.storage_controllers.push(settings);
        Ok(())
    }

    pub fn set_network_adapter(&mut self, slot: u32, enabled: bool) -> VBoxResult<()> {
        let adapter = NetworkAdapterSettings {
            slot,
            enabled,
            adapter_type: "82540EM".to_string(),
            mac: [0u8; 6],
            cable_connected: true,
            speed: 1000,
        };
        while self.settings.network_adapters.len() <= slot as usize {
            self.settings.network_adapters.push(NetworkAdapterSettings::default());
        }
        self.settings.network_adapters[slot as usize] = adapter;
        Ok(())
    }

    pub fn take_snapshot(&self, name: &str) -> VBoxResult<Snapshot> {
        let snapshot = Snapshot {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            description: String::new(),
            timestamp: chrono::Utc::now().timestamp(),
            state: self.state,
            data: vec![],
        };
        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_create() {
        let vb = Arc::new(crate::virtualbox::VirtualBox::new());
        let machine = Machine::new("test", Uuid::new_v4(), vb);
        assert_eq!(machine.name, "test");
    }
}
