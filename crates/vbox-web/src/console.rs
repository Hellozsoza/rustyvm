use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, KeyboardEvent, MouseEvent, TouchEvent, DragEvent};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;

use crate::BrowserVmManager;
use crate::BrowserNetwork;
use crate::BrowserAudio;
use crate::BrowserStorage;

pub struct BrowserConsole {
    canvas: HtmlCanvasElement,
    context: Option<CanvasRenderingContext2d>,
    width: u32,
    height: u32,
    fullscreen: bool,
    scale: f64,
    mouse_x: f64,
    mouse_y: f64,
    key_state: HashMap<u32, bool>,
    mouse_grabbed: bool,
    vm_manager: BrowserVmManager,
    network: BrowserNetwork,
    audio: BrowserAudio,
    storage: BrowserStorage,
}

#[wasm_bindgen]
impl BrowserConsole {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<BrowserConsole, JsValue> {
        let width = canvas.width();
        let height = canvas.height();
        let context = canvas.get_context("2d")
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?
            .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok());

        let console = BrowserConsole {
            canvas,
            context,
            width,
            height,
            fullscreen: false,
            scale: 1.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            key_state: HashMap::new(),
            mouse_grabbed: false,
            vm_manager: BrowserVmManager::new(),
            network: BrowserNetwork::new().unwrap(),
            audio: BrowserAudio::new().unwrap(),
            storage: BrowserStorage::new().unwrap(),
        };

        Ok(console)
    }

    pub fn start(&mut self) {
        log::info!("BrowserConsole starting VM and rendering loop");
    }

    pub fn render_frame(&mut self) {
        if let Some(ref ctx) = self.context {
            let width = self.width as f64;
            let height = self.height as f64;
            ctx.clear_rect(0.0, 0.0, width, height);
            ctx.set_fill_style(&JsValue::from_str("#000000"));
            ctx.fill_rect(0.0, 0.0, width, height);
        }
    }

    pub fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        let key_code = event.key_code();
        let pressed = !event.get_default_prevented();
        self.key_state.insert(key_code as u32, pressed);
    }

    pub fn handle_mouse_event(&mut self, event: &MouseEvent) {
        self.mouse_x = event.client_x() as f64;
        self.mouse_y = event.client_y() as f64;
    }

    pub fn handle_touch_event(&mut self, _event: &TouchEvent) {
    }

    pub fn handle_drag_event(&mut self, _event: &DragEvent) {
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }

    pub fn take_screenshot(&self) -> Result<Vec<u8>, JsValue> {
        Ok(vec![])
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.canvas.set_width(width);
        self.canvas.set_height(height);
    }

    pub fn get_width(&self) -> u32 { self.width }
    pub fn get_height(&self) -> u32 { self.height }
    pub fn is_fullscreen(&self) -> bool { self.fullscreen }
    pub fn get_scale(&self) -> f64 { self.scale }
    pub fn get_mouse_x(&self) -> f64 { self.mouse_x }
    pub fn get_mouse_y(&self) -> f64 { self.mouse_y }
}

impl Default for BrowserConsole {
    fn default() -> Self {
        BrowserConsole::new(
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap()
        ).unwrap()
    }
}
