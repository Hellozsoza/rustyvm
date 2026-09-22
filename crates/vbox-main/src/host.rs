use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Host information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub cpus: u32,
    pub memory_size: u64,
    pub os_type: String,
    pub cpu_speed: u32,
    pub network_interfaces: Vec<NetworkInterface>,
    pub usb_controllers: Vec<UsbController>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub mac: [u8; 6],
    pub ip: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbController {
    pub name: String,
    pub controller_type: String,
    pub speed: u32,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            name: String::new(),
            cpus: 1,
            memory_size: 0,
            os_type: String::new(),
            cpu_speed: 0,
            network_interfaces: Vec::new(),
            usb_controllers: Vec::new(),
        }
    }
}

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
