#![allow(non_camel_case_types)]
#![allow(clippy::all)]

pub mod host;
pub mod machine;
pub mod virtualbox;
pub mod medium;
pub mod console;
pub mod settings;
pub mod com;
pub mod guest_control;
pub mod shared_folders;
pub mod clipboard;
pub mod guest_properties;

pub use vbox_core::*;
pub use vbox_runtime::*;
pub use vbox_storage::*;

use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

pub struct VboxMain {
    initialized: bool,
}

impl VboxMain {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub fn init(&mut self) -> VBoxResult<()> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for VboxMain {
    fn default() -> Self {
        Self::new()
    }
}
