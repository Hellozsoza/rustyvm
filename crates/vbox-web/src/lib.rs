#![allow(non_camel_case_types)]

pub mod glue;
pub mod canvas;
pub mod input;
pub mod audio;
pub mod display;
pub mod event;

pub use vbox_core::*;
pub use vbox_runtime::*;
pub use vbox_vmm::*;
pub use vbox_devices::*;
pub use vbox_storage::*;
pub use vbox_main::*;

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[cfg(target_arch = "wasm32")]
use console_log;

#[wasm_bindgen]
pub struct VBoxWeb {
    canvas: Option<HtmlCanvasElement>,
    vm: Option<VBoxVM>,
}

#[wasm_bindgen]
impl VBoxWeb {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            canvas: None,
            vm: None,
        }
    }

    pub fn init(&mut self) -> VBoxResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            console_log::init_with_level(log::LevelFilter::Debug).unwrap();
        }
        Ok(())
    }

    pub fn create_vm(&mut self, name: &str) -> VBoxResult<()> {
        let main = vbox_main::VirtualBox::new();
        let vm = main.create_vm(name)?;
        self.vm = Some(vm);
        Ok(())
    }

    pub fn start_vm(&mut self) -> VBoxResult<()> {
        if let Some(ref mut vm) = self.vm {
            vm.start()?;
        }
        Ok(())
    }

    pub fn stop_vm(&mut self) -> VBoxResult<()> {
        if let Some(ref mut vm) = self.vm {
            vm.stop()?;
        }
        Ok(())
    }

    pub fn set_canvas(&mut self, canvas: HtmlCanvasElement) {
        self.canvas = Some(canvas);
    }
}

#[wasm_bindgen]
pub fn vbox_web_init() -> VBoxResult<VBoxWeb> {
    let mut web = VBoxWeb::new();
    web.init()?;
    Ok(web)
}

pub struct CanvasRenderer {
    canvas: Option<HtmlCanvasElement>,
    context: Option<web_sys::CanvasRenderingContext2D>,
    width: u32,
    height: u32,
}

impl CanvasRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> VBoxResult<Self> {
        let context = canvas.get_context("2d")
            .map_err(|_| VBoxError::NotImplemented)?
            .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2D>().ok());
        let width = canvas.width();
        let height = canvas.height();
        Ok(Self {
            canvas: Some(canvas),
            context,
            width,
            height,
        })
    }

    pub fn render(&self, data: &[u8], width: u32, height: u32) -> VBoxResult<()> {
        if let Some(ref ctx) = self.context {
            let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                wasm_bindgen::Clamped(data),
                width as i32,
                height as i32,
            ).map_err(|_| VBoxError::InvalidParam(String::new()))?;
            ctx.put_image_data(&image_data, 0.0, 0.0)
                .map_err(|_| VBoxError::Unknown("Render failed".to_string()))?;
        }
        Ok(())
    }

    pub fn clear(&self) -> VBoxResult<()> {
        if let Some(ref ctx) = self.context {
            ctx.clear_rect(0.0, 0.0, self.width as f64, self.height as f64);
        }
        Ok(())
    }
}
