use vbox_core::VBoxError;
use vbox_core::VBoxResult;
// serde not needed for non-serializable types
use parking_lot::Mutex;
use std::sync::Arc;
use log::info;
use crate::pgm::MmioRegion;

#[derive(Debug, Clone)]
pub struct IomState {
    pub mmio_regions: Vec<MmioRegion>,
    pub io_port_regions: Vec<IoPortRegion>,
}

#[derive(Debug, Clone)]
pub struct IoPortRegion {
    pub base: u16,
    pub size: u16,
    pub handler: fn(u16, u32) -> u32,
}

impl IomState {
    pub fn new() -> Self {
        IomState {
            mmio_regions: Vec::new(),
            io_port_regions: Vec::new(),
        }
    }
}

pub fn IOMR3Init() -> Result<IomState, VBoxError> {
    let state = IomState::new();
    info!("IOM: IO/MEM Manager initialized");
    Ok(state)
}

pub fn iom_register_mmio(state: &mut IomState, region: MmioRegion) -> Result<(), VBoxError> {
    state.mmio_regions.push(region);
    info!("IOM: Registered MMIO region at 0x{:X}", region.base);
    Ok(())
}

pub fn iom_register_io_port(state: &mut IomState, base: u16, size: u16, handler: fn(u16, u32) -> u32) -> Result<(), VBoxError> {
    let region = IoPortRegion { base, size, handler };
    state.io_port_regions.push(region);
    info!("IOM: Registered IO port region at 0x{:04X}", base);
    Ok(())
}

pub fn iom_read_mmio(state: &IomState, addr: u64, size: u32) -> Result<u32, VBoxError> {
    for region in &state.mmio_regions {
        if addr >= region.base && addr < region.base + region.size {
            return Ok((region.handler)(addr, size));
        }
    }
    Err(VBoxError::Fail)
}

pub fn iom_write_mmio(state: &mut IomState, addr: u64, size: u32, value: u32) -> Result<(), VBoxError> {
    for region in &state.mmio_regions {
        if addr >= region.base && addr < region.base + region.size {
            return Ok(());
        }
    }
    Err(VBoxError::Fail)
}
