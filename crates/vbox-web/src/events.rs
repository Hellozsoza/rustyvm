use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{KeyboardEvent, MouseEvent, TouchEvent, DragEvent, ClipboardEvent};
use uuid::Uuid;
use std::collections::HashMap;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key_code: u32,
    pub key: String,
    pub pressed: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub x: f64,
    pub y: f64,
    pub buttons: u16,
    pub clicked: bool,
    pub dbl_click: bool,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct TouchEvent {
    pub x: f64,
    pub y: f64,
    pub touch_count: u32,
    pub pressure: f64,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ClipboardData {
    pub text: String,
    pub types: Vec<String>,
    pub data: Vec<u8>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct DragData {
    pub files: Vec<String>,
    pub types: Vec<String>,
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
pub struct EventProcessor;

#[wasm_bindgen]
impl EventProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> EventProcessor {
        EventProcessor
    }

    pub fn handle_keyboard(event: &KeyboardEvent) -> KeyEvent {
        KeyEvent {
            key_code: event.key_code() as u32,
            key: event.key(),
            pressed: !event.get_default_prevented(),
            ctrl: event.ctrl_key(),
            shift: event.shift_key(),
            alt: event.alt_key(),
        }
    }

    pub fn handle_mouse(event: &MouseEvent) -> MouseEvent {
        MouseEvent {
            x: event.client_x() as f64,
            y: event.client_y() as f64,
            buttons: event.buttons(),
            clicked: event.type_() == "click",
            dbl_click: event.type_() == "dblclick",
        }
    }

    pub fn handle_touch(event: &TouchEvent) -> TouchEvent {
        TouchEvent {
            x: event.touches().get(0).map(|t| t.client_x() as f64).unwrap_or(0.0),
            y: event.touches().get(0).map(|t| t.client_y() as f64).unwrap_or(0.0),
            touch_count: event.touches().length() as u32,
            pressure: 0.5,
        }
    }

    pub fn handle_clipboard(event: &ClipboardEvent) -> ClipboardData {
        ClipboardData {
            text: String::new(),
            types: vec![],
            data: vec![],
        }
    }

    pub fn handle_drag(event: &DragEvent) -> DragData {
        DragData {
            files: vec![],
            types: vec![],
            x: event.client_x() as f64,
            y: event.client_y() as f64,
        }
    }
}

#[wasm_bindgen]
pub fn setup_event_listeners(console: &BrowserConsole) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let canvas = document.get_element_by_id("vbox-console")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let _ = EventProcessor::handle_keyboard(&event);
    }) as Box<dyn FnMut(KeyboardEvent)>);
    document.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();
}
