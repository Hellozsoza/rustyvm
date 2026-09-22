use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Document};
use std::collections::HashMap;
use uuid::Uuid;

use vbox_main::VmSettings;

#[wasm_bindgen]
pub struct SettingsEditor {
    settings: VmSettings,
    elements: HashMap<String, HtmlElement>,
    document: Option<Document>,
}

#[wasm_bindgen]
impl SettingsEditor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<SettingsEditor, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let document = window.document().ok_or_else(|| JsValue::from_str("No document"))?;

        let editor = SettingsEditor {
            settings: VmSettings::default(),
            elements: HashMap::new(),
            document: Some(document),
        };

        Ok(editor)
    }

    pub fn render(&self) -> Result<HtmlElement, JsValue> {
        let document = self.document.as_ref().ok_or_else(|| JsValue::from_str("No document"))?;
        let div = document.create_element("div").map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        div.set_id("vbox-settings");
        Ok(div.dyn_into::<HtmlElement>().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?)
    }

    pub fn handle_change(&mut self, element_id: &str, value: &str) -> Result<(), JsValue> {
        match element_id {
            "name" => self.settings.name = value.to_string(),
            "memory" => self.settings.memory_size = value.parse().unwrap_or(1024),
            "cpus" => self.settings.cpu_count = value.parse().unwrap_or(1),
            _ => {}
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), JsValue> {
        if self.settings.name.is_empty() {
            return Err(JsValue::from_str("VM name cannot be empty"));
        }
        if self.settings.memory_size == 0 {
            return Err(JsValue::from_str("Memory size must be greater than 0"));
        }
        Ok(())
    }

    pub fn get_settings(&self) -> VmSettings {
        self.settings.clone()
    }
}

impl Default for SettingsEditor {
    fn default() -> Self {
        SettingsEditor::new().unwrap()
    }
}
