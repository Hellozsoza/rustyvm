#![allow(non_camel_case_types)]

pub mod vm;
pub mod em;
pub mod cpu;
pub mod pgm;
pub mod tm;
pub mod pdm;
pub mod ssm;
pub mod iem;

pub use vbox_core::*;

pub struct VBoxVM {
    pub id: u32,
    pub name: String,
    pub state: VMState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMState {
    PoweredOff,
    Running,
    Paused,
    Saved,
    Teleporting,
    FaultError,
    Stuck,
    PoweredOffLingering,
}

impl VBoxVM {
    pub fn new(name: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            state: VMState::PoweredOff,
        }
    }

    pub fn start(&mut self) -> VBoxResult<()> {
        self.state = VMState::Running;
        Ok(())
    }

    pub fn stop(&mut self) -> VBoxResult<()> {
        self.state = VMState::PoweredOff;
        Ok(())
    }

    pub fn pause(&mut self) -> VBoxResult<()> {
        self.state = VMState::Paused;
        Ok(())
    }

    pub fn save(&mut self) -> VBoxResult<()> {
        self.state = VMState::Saved;
        Ok(())
    }

    pub fn reset(&mut self) -> VBoxResult<()> {
        self.state = VMState::PoweredOff;
        Ok(())
    }
}

impl Default for VBoxVM {
    fn default() -> Self {
        Self::new("default")
    }
}
