use vbox_core::VBoxError;
use vbox_core::VBoxResult;
use serde::{Serialize, Deserialize};
use log::info;
use crate::vm::VM;
use crate::cpu::CpuState;
use crate::pgm::GuestMemory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSMHandle {
    pub data: Vec<u8>,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSMSavedState {
    pub cpu_state: CpuState,
    pub memory_snapshot: Vec<u8>,
    pub device_states: Vec<u8>,
    pub version: u32,
    pub pass: u32,
}

impl SSMHandle {
    pub fn new() -> Self {
        SSMHandle {
            data: Vec::new(),
            position: 0,
        }
    }
}

pub fn SSMR3Init() -> Result<SSMHandle, VBoxError> {
    let handle = SSMHandle::new();
    info!("SSM: Saved State Manager initialized");
    Ok(handle)
}

pub fn SSMR3Save(pVM: &VM) -> Result<Vec<u8>, VBoxError> {
    let state = SSMSavedState {
        cpu_state: pVM.cpus.first().cloned().unwrap_or_default(),
        memory_snapshot: pVM.p_guest_memory.to_vec(),
        device_states: Vec::new(),
        version: 1,
        pass: 0,
    };
    let serialized = bincode::serialize(&state)
        .map_err(|_| VBoxError::Fail)?;
    info!("SSM: Saved VM state ({} bytes)", serialized.len());
    Ok(serialized)
}

pub fn SSMR3Load(data: &[u8]) -> Result<SSMSavedState, VBoxError> {
    let state: SSMSavedState = bincode::deserialize(data)
        .map_err(|_| VBoxError::Fail)?;
    info!("SSM: Loaded VM state (version {})", state.version);
    Ok(state)
}

pub fn SSMR3Register(name: &str) -> Result<(), VBoxError> {
    info!("SSM: Registered component '{}'", name);
    Ok(())
}
