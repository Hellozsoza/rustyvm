use vbox_core::VBoxError;
use vbox_core::VBoxResult;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use parking_lot::Mutex;
use log::info;
use crate::vm::VM;
use crate::cpu::CpuState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbgState {
    pub halted: bool,
    pub breakpoint_count: u32,
    pub trace_enabled: bool,
    pub watchpoints: Vec<Watchpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchpoint {
    pub address: u64,
    pub size: u8,
    pub rw: u8,
}

impl DbgState {
    pub fn new() -> Self {
        DbgState {
            halted: false,
            breakpoint_count: 0,
            trace_enabled: false,
            watchpoints: Vec::new(),
        }
    }
}

pub fn DBGR3Init() -> Result<DbgState, VBoxError> {
    let state = DbgState::new();
    info!("DBG: Debugger initialized");
    Ok(state)
}

pub fn dbg_halt(pVM: &mut VM, reason: &str) -> Result<(), VBoxError> {
    pVM.enm_state = crate::em::EMState::DebugState;
    info!("DBG: VM halted - {}", reason);
    Ok(())
}

pub fn dbg_continue(pVM: &mut VM) -> Result<(), VBoxError> {
    pVM.enm_state = crate::em::EMState::Running;
    info!("DBG: VM resumed from debug");
    Ok(())
}

pub fn dbg_set_breakpoint(state: &mut DbgState, addr: u64) -> Result<(), VBoxError> {
    state.watchpoints.push(Watchpoint {
        address: addr,
        size: 1,
        rw: 0,
    });
    state.breakpoint_count += 1;
    info!("DBG: Breakpoint at 0x{:X}", addr);
    Ok(())
}

pub fn dbg_read_memory(pVM: &VM, addr: u64, size: usize) -> Result<Vec<u8>, VBoxError> {
    if (addr as usize) + size > pVM.p_guest_memory.len() {
        return Err(VBoxError::Fail);
    }
    Ok(pVM.p_guest_memory[addr as usize..addr as usize + size].to_vec())
}

pub fn dbg_write_memory(pVM: &mut VM, addr: u64, data: &[u8]) -> Result<(), VBoxError> {
    if (addr as usize) + data.len() > pVM.p_guest_memory.len() {
        return Err(VBoxError::Fail);
    }
    for (i, byte) in data.iter().enumerate() {
        pVM.p_guest_memory[(addr as usize) + i] = *byte;
    }
    Ok(())
}
