use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlCanvasElement, Window, Document, KeyboardEvent, MouseEvent, TouchEvent, DragEvent};
use std::cell::RefCell;
use std::rc::Rc;

use crate::BrowserConsole;
use crate::BrowserVmManager;

pub fn init() {
    console_log::init_with_level(log::LevelFilter::Info).unwrap();
    log::info!("VirtualBox Browser Edition starting...");
    vbox_runtime::init();
}

fn setup_event_listeners(console: &BrowserConsole) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let canvas = document.get_element_by_id("vbox-console")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let _ = console;
    }) as Box<dyn FnMut(KeyboardEvent)>);
    document.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();
}

fn start_event_loop() {
    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();
    let mut last_time = performance.now();

    let closure = Closure::wrap(Box::new(move |now: f64| {
        let _delta = now - last_time;
        last_time = now;
        start_event_loop();
    }) as Box<dyn FnMut(f64)>);

    window.request_animation_frame(closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_log::init()?;
    log::info!("VirtualBox Browser Edition starting...");

    vbox_runtime::init();

    let mut vm_manager = BrowserVmManager::new();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("vbox-console")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();
    let mut console = BrowserConsole::new(canvas)?;

    setup_event_listeners(&console);
    start_event_loop();

    Ok(())
}
