use vbox_core::VBoxError;
use vbox_core::VBoxResult;
use parking_lot::Mutex;
use std::sync::Arc;
use log::info;
use crate::cpu::CpuState;
use crate::pgm::{PgmState, GuestMemory};
use crate::iem::IemState;
use crate::em::EMState;

#[derive(Debug, Clone)]
pub struct VMCPU {
    pub cpu_state: CpuState,
    pub id: u32,
    pub enm_state: EMState,
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

impl VM {
    pub fn new(cCpus: u32) -> Result<Self, VBoxError> {
        let ram_size = 2 * 1024 * 1024 * 1024;
        let guest_memory = vec![0u8; ram_size].into_boxed_slice();
        let mut cpus = Vec::new();
        for i in 0..cCpus {
            let mut cpu = CpuState::default();
            cpu.cs = 0x08;
            cpu.ds = 0x10;
            cpu.es = 0x10;
            cpu.ss = 0x10;
            cpu.mode = crate::cpu::CpuMode::Protected;
            cpus.push(cpu);
        }
        let vm = VM {
            cpus,
            p_guest_memory: guest_memory,
            enm_state: EMState::Inactive,
            p_pgm: None,
            p_iem: None,
            cCpus,
        };
        info!("VM: Created VM with {} CPUs", cCpus);
        Ok(vm)
    }
}

pub fn VMR3Create(cCpus: u32) -> Result<VM, VBoxError> {
    let vm = VM::new(cCpus)?;
    info!("VMR3Create: VM created with {} CPUs", cCpus);
    Ok(vm)
}

pub fn VMR3Destroy(pVM: &mut VM) -> Result<(), VBoxError> {
    pVM.cpus.clear();
    pVM.p_guest_memory = vec![0u8; 0].into_boxed_slice();
    info!("VMR3Destroy: VM destroyed");
    Ok(())
}

pub fn VMR3PowerOff(pVM: &mut VM) -> Result<(), VBoxError> {
    pVM.enm_state = EMState::Inactive;
    for cpu in pVM.cpus.iter_mut() {
        *cpu = CpuState::default();
    }
    info!("VMR3PowerOff: VM powered off");
    Ok(())
}

pub fn VMR3Reset(pVM: &mut VM) -> Result<(), VBoxError> {
    for cpu in pVM.cpus.iter_mut() {
        *cpu = CpuState::default();
    }
    pVM.enm_state = EMState::Inactive;
    info!("VMR3Reset: VM reset");
    Ok(())
}

pub fn VMR3Init(pVM: &mut VM) -> Result<(), VBoxError> {
    pVM.enm_state = EMState::Inactive;
    info!("VMR3Init: VM initialized");
    Ok(())
}
