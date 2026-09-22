use vbox_core::VBoxError;
use vbox_core::VBoxResult;
use serde::{Serialize, Deserialize};
use parking_lot::Mutex;
use std::sync::Arc;
use log::info;
use crate::pgm::PgmState;
use crate::iem::IemState;
use crate::cpu::CpuState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EMState {
    Inactive,
    Running,
    Paused,
    DebugState,
    Resume,
    FatalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EMCPU {
    pub cpu_state: CpuState,
    pub enm_state: EMState,
    pub id: u32,
}

#[derive(Debug, Clone)]
pub struct VM {
    pub cpus: Vec<CpuState>,
    pub p_guest_memory: Box<[u8]>,
    pub enm_state: EMState,
    pub p_pgm: Option<PgmState>,
    pub p_iem: Option<IemState>,
    pub cCpus: u32,
}

pub type VBOXSTRICTRC = VBoxError;

pub fn EMR3Init() -> Result<(), VBoxError> {
    info!("EM: Execution Manager initialized");
    Ok(())
}

pub fn EMR3Shutdown() -> Result<(), VBoxError> {
    info!("EM: Execution Manager shutdown");
    Ok(())
}

pub fn EMR3Suspend(pVM: &mut VM) -> Result<(), VBoxError> {
    pVM.enm_state = EMState::Paused;
    info!("EM: VM suspended");
    Ok(())
}

pub fn EMR3Resume(pVM: &mut VM) -> Result<(), VBoxError> {
    pVM.enm_state = EMState::Running;
    info!("EM: VM resumed");
    Ok(())
}

pub fn EMR3Pause(pVM: &mut VM) -> Result<(), VBoxError> {
    pVM.enm_state = EMState::Paused;
    info!("EM: VM paused");
    Ok(())
}

pub fn em_execute_vm(pVM: &mut VM) -> Result<(), VBoxError> {
    while pVM.enm_state == EMState::Running {
        for vcpu in pVM.cpus.iter_mut() {
            let mem = &pVM.p_guest_memory;
            let _ = vcpu;
            let _ = mem;
        }
    }
    Ok(())
}
