use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;

use vbox_main::{VirtualBox, Machine, VmSettings};

#[wasm_bindgen]
#[derive(Clone)]
pub struct BrowserVmManager {
    virtualbox: VirtualBox,
    current_vm: Option<Uuid>,
    vm_states: HashMap<Uuid, JsValue>,
}

#[wasm_bindgen]
impl BrowserVmManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BrowserVmManager {
        BrowserVmManager {
            virtualbox: VirtualBox::new(),
            current_vm: None,
            vm_states: HashMap::new(),
        }
    }

    pub fn create_vm(&mut self, name: &str) -> Result<Uuid, JsValue> {
        self.virtualbox.init().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let uuid = self.virtualbox.create_machine(name)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        self.current_vm = Some(uuid);
        Ok(uuid)
    }

    pub fn start_vm(&mut self, uuid: Uuid) -> Result<(), JsValue> {
        if let Some(machine) = self.virtualbox.find_machine(uuid) {
            machine.start().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            self.current_vm = Some(uuid);
        }
        Ok(())
    }

    pub fn pause_vm(&mut self, uuid: Uuid) -> Result<(), JsValue> {
        if let Some(machine) = self.virtualbox.find_machine(uuid) {
            machine.pause().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        }
        Ok(())
    }

    pub fn stop_vm(&mut self, uuid: Uuid) -> Result<(), JsValue> {
        if let Some(machine) = self.virtualbox.find_machine(uuid) {
            machine.stop().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        }
        Ok(())
    }

    pub fn get_vm_state(&self, uuid: Uuid) -> Result<String, JsValue> {
        if let Some(machine) = self.virtualbox.machines.get(&uuid) {
            Ok(machine.get_state())
        } else {
            Ok("not_found".to_string())
        }
    }

    pub fn list_vms(&self) -> Vec<Uuid> {
        self.virtualbox.machines.keys().copied().collect()
    }

    pub fn load_vm(&mut self, data: &[u8]) -> Result<(), JsValue> {
        Ok(())
    }

    pub fn save_vm(&self, uuid: Uuid) -> Result<Vec<u8>, JsValue> {
        Ok(vec![])
    }

    pub fn create_snapshot(&mut self, uuid: Uuid, name: &str) -> Result<(), JsValue> {
        Ok(())
    }
}

impl Default for BrowserVmManager {
    fn default() -> Self {
        BrowserVmManager::new()
    }
}
