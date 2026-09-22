use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::{RwLock, Mutex};
use std::sync::Arc;
use uuid::Uuid;

use crate::com::IID;
use crate::machine::Machine;
use crate::settings::VmSettings;
use crate::console::Console;
use crate::medium::Medium;
use crate::host::Host;
use crate::guest_properties::GuestPropertyService;
use crate::shared_folders::SharedFolderService;
use crate::clipboard::ClipboardService;
use crate::guest_control::GuestControlService;

/// VirtualBox settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualBoxSettings {
    pub machine_folder: String,
    pub system_folder: String,
    pub log_folder: String,
    pub default_frontend: String,
    pub video_adapter: String,
    pub network_adapters_count: u32,
    pub extra_data: serde_json::Map<String, serde_json::Value>,
}

impl Default for VirtualBoxSettings {
    fn default() -> Self {
        Self {
            machine_folder: String::new(),
            system_folder: String::new(),
            log_folder: String::new(),
            default_frontend: String::new(),
            video_adapter: String::new(),
            network_adapters_count: 4,
            extra_data: serde_json::Map::new(),
        }
    }
}

/// Host information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub cpus: u32,
    pub memory_size: u64,
    pub os_type: String,
    pub cpu_speed: u32,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            name: String::new(),
            cpus: 1,
            memory_size: 0,
            os_type: String::new(),
            cpu_speed: 0,
        }
    }
}

/// VirtualBox object - the main entry point for all VM management.
pub struct VirtualBox {
    pub machines: RwLock<Vec<Arc<Machine>>>,
    pub host: Host,
    pub settings: VirtualBoxSettings,
    pub guest_properties: GuestPropertyService,
    pub shared_folders: SharedFolderService,
    pub clipboard: ClipboardService,
    pub guest_control: GuestControlService,
}

#[derive(Error, Debug)]
pub enum VirtualBoxError {
    #[error("Machine not found: {0}")]
    MachineNotFound(String),
    #[error("Machine already exists")]
    MachineExists,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl VirtualBox {
    pub fn new() -> VirtualBox {
        VirtualBox {
            machines: RwLock::new(Vec::new()),
            host: Host::default(),
            settings: VirtualBoxSettings::default(),
            guest_properties: GuestPropertyService::new(),
            shared_folders: SharedFolderService::new(),
            clipboard: ClipboardService::new(),
            guest_control: GuestControlService::new(),
        }
    }

    pub fn create_machine(&mut self, name: &str) -> VBoxResult<Arc<Machine>> {
        let uuid = Uuid::new_v4();
        let machine = Arc::new(Machine::new(name, uuid, Arc::new(VirtualBox::new())));
        self.machines.write().push(machine.clone());
        Ok(machine)
    }

    pub fn find_machine(&self, uuid: Uuid) -> Option<Arc<Machine>> {
        self.machines.read().iter()
            .find(|m| m.uuid == uuid)
            .cloned()
    }

    pub fn get_machines(&self) -> Vec<Arc<Machine>> {
        self.machines.read().clone()
    }

    pub fn register_machine(&mut self, machine: Arc<Machine>) {
        self.machines.write().push(machine);
    }

    pub fn unregister_machine(&mut self, uuid: Uuid) -> VBoxResult<()> {
        let mut machines = self.machines.write();
        let pos = machines.iter().position(|m| m.uuid == uuid);
        if let Some(pos) = pos {
            machines.remove(pos);
            Ok(())
        } else {
            Err(VBoxError::FileNotFound)
        }
    }

    pub fn get_machine_count(&self) -> usize {
        self.machines.read().len()
    }
}

impl Default for VirtualBox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtualbox_new() {
        let vb = VirtualBox::new();
        assert_eq!(vb.get_machine_count(), 0);
    }

    #[test]
    fn test_create_machine() {
        let mut vb = VirtualBox::new();
        let machine = vb.create_machine("test_vm");
        assert!(machine.is_ok());
        assert_eq!(vb.get_machine_count(), 1);
    }
}
