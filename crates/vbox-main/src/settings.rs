use serde::{Deserialize, Serialize};
use vbox_core::VBoxResult;
use serde_json;

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
    pub mac: [u8; 6],
    pub cable_connected: bool,
    pub speed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageControllerSettings {
    pub name: String,
    pub controller_type: String,
    pub bus_type: String,
    pub max_device_count: u32,
    pub use_host_io_cache: bool,
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
