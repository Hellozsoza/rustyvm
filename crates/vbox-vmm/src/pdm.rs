use vbox_core::VBoxError;
use vbox_core::VBoxResult;
use parking_lot::Mutex;
use std::sync::Arc;
use log::info;
use crate::pgm::MmioRegion;

pub trait PdmDeviceTrait: Send {
    fn attach(&mut self, config: &str) -> Result<(), VBoxError>;
    fn detach(&mut self) -> Result<(), VBoxError>;
    fn reset(&mut self) -> Result<(), VBoxError>;
    fn power_off(&mut self) -> Result<(), VBoxError>;
    fn query_interface(&self, iid: &str) -> Option<String>;
}

pub trait PdmDriverTrait: Send {
    fn init(&mut self, device: &dyn PdmDeviceTrait) -> Result<(), VBoxError>;
    fn power_off(&mut self) -> Result<(), VBoxError>;
    fn reset(&mut self) -> Result<(), VBoxError>;
}

#[derive(Debug, Clone)]
pub struct PdmDeviceHelper {
    pub mmio_regions: Vec<MmioRegion>,
    pub io_port_ranges: Vec<(u16, u16)>,
    pub device_name: String,
}

impl PdmDeviceHelper {
    pub fn new(name: &str) -> Self {
        PdmDeviceHelper {
            mmio_regions: Vec::new(),
            io_port_ranges: Vec::new(),
            device_name: name.to_string(),
        }
    }

    pub fn register_mmio(&mut self, region: MmioRegion) {
        self.mmio_regions.push(region);
    }

    pub fn register_io_port(&mut self, port_start: u16, port_end: u16) {
        self.io_port_ranges.push((port_start, port_end));
    }
}

pub struct PdmState {
    pub device_list: Vec<Box<dyn PdmDeviceTrait>>,
    pub driver_list: Vec<Box<dyn PdmDriverTrait>>,
    pub device_helpers: Vec<PdmDeviceHelper>,
}

pub fn PDMR3Init() -> Result<PdmState, VBoxError> {
    let state = PdmState {
        device_list: Vec::new(),
        driver_list: Vec::new(),
        device_helpers: Vec::new(),
    };
    info!("PDM: Pluggable Device Manager initialized");
    Ok(state)
}

pub fn PDMR3RegisterDevice(state: &mut PdmState, name: &str) -> Result<(), VBoxError> {
    info!("PDM: Registered device '{}'", name);
    Ok(())
}

pub fn PDMR3CreateDevice(state: &mut PdmState, name: &str) -> Result<Box<dyn PdmDeviceTrait>, VBoxError> {
    let helper = PdmDeviceHelper::new(name);
    state.device_helpers.push(helper);
    info!("PDM: Created device '{}'", name);
    Ok(Box::new(PdmDeviceHelper::new(name)))
}

impl PdmDeviceTrait for PdmDeviceHelper {
    fn attach(&mut self, config: &str) -> Result<(), VBoxError> {
        self.device_name = format!("{}:{}", self.device_name, config);
        Ok(())
    }
    fn detach(&mut self) -> Result<(), VBoxError> { Ok(()) }
    fn reset(&mut self) -> Result<(), VBoxError> { Ok(()) }
    fn power_off(&mut self) -> Result<(), VBoxError> { Ok(()) }
    fn query_interface(&self, _iid: &str) -> Option<String> { None }
}
