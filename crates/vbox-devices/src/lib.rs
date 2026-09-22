#![allow(non_camel_case_types)]

use uuid::Uuid;

pub mod network;
pub mod storage;
pub mod audio;
pub mod usb;
pub mod graphics;
pub mod vmmdev;
pub mod bus;
pub mod iommu;

pub use vbox_core::*;

pub trait Device {
    fn attach(&mut self, config: &DeviceConfig) -> VBoxResult<()>;
    fn detach(&mut self) -> VBoxResult<()>;
    fn reset(&mut self) -> VBoxResult<()>;
    fn power_off(&mut self) -> VBoxResult<()>;
    fn query_interface(&self, iid: &Uuid) -> Option<&dyn core::any::Any>;
    fn get_name(&self) -> &str;
    fn get_config(&self) -> &DeviceConfig;
}

pub trait Driver {
    fn init(&mut self, device: &dyn Device) -> VBoxResult<()>;
    fn power_off(&mut self) -> VBoxResult<()>;
    fn reset(&mut self) -> VBoxResult<()>;
    fn detach(&mut self) -> VBoxResult<()>;
}

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub pszName: String,
    pub pszDescription: String,
    pub pszMachine: String,
    pub pszParent: String,
    pub id: Uuid,
    pub fEnabled: bool,
    pub cMaxInstances: u32,
    pub cInstances: u32,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            pszName: String::new(),
            pszDescription: String::new(),
            pszMachine: String::new(),
            pszParent: String::new(),
            id: Uuid::new_v4(),
            fEnabled: true,
            cMaxInstances: 1,
            cInstances: 0,
        }
    }
}

pub struct DeviceManager {
    devices: parking_lot::Mutex<Vec<Box<dyn Device>>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: parking_lot::Mutex::new(Vec::new()),
        }
    }

    pub fn register(&mut self, device: Box<dyn Device>) -> VBoxResult<()> {
        self.devices.lock().push(device);
        Ok(())
    }

    pub fn find(&self, name: &str) -> Option<Box<dyn Device>> {
        self.devices.lock().iter().find(|d| d.get_name() == name).cloned()
    }

    pub fn enumerate(&self) -> Vec<Box<dyn Device>> {
        self.devices.lock().clone()
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PDMDevice {
    pub pDevIns: *mut core::ffi::c_void,
    pub pszName: String,
    pub pReg: Box<dyn Device>,
}

pub enum PDMQueryInterfaceResult {
    Found(Box<dyn Device>),
    NotFound,
}
