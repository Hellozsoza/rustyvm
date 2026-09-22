use vbox_core::VBoxError;
use vbox_core::VBoxResult;
use serde::{Serialize, Deserialize};
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TRPMState {
    pub idt_base: u64,
    pub idt_limit: u16,
    pub idt_valid: bool,
    pub exceptions: Vec<ExceptionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionEntry {
    pub vector: u8,
    pub handler: u64,
    pub type_attr: u8,
    pub segment: u16,
    pub present: bool,
}

impl TRPMState {
    pub fn new() -> Self {
        TRPMState {
            idt_base: 0,
            idt_limit: 0,
            idt_valid: false,
            exceptions: vec![ExceptionEntry {
                vector: 0,
                handler: 0,
                type_attr: 0,
                segment: 0,
                present: false,
            }; 256],
        }
    }
}

pub fn TRPM3Init() -> Result<TRPMState, VBoxError> {
    let state = TRPMState::new();
    info!("TRPM: Trap Manager initialized");
    Ok(state)
}

pub fn trpm_set_idt(state: &mut TRPMState, base: u64, limit: u16) -> Result<(), VBoxError> {
    state.idt_base = base;
    state.idt_limit = limit;
    state.idt_valid = true;
    info!("TRPM: IDT set at 0x{:X}:0x{:X}", base, limit);
    Ok(())
}

pub fn trpm_inject_interrupt(state: &mut TRPMState, vector: u8) -> Result<(), VBoxError> {
    if vector < state.exceptions.len() as u8 {
        info!("TRPM: Injected interrupt vector {}", vector);
    }
    Ok(())
}

pub fn trpm_get_idt(state: &TRPMState) -> (u64, u16) {
    (state.idt_base, state.idt_limit)
}

pub fn trpm_check_interrupts(state: &TRPMState) -> bool {
    state.idt_valid
}
