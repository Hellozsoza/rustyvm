use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use cfg_if::cfg_if;

pub const SHADER_SOURCE: &str = r#"
[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>, [[location(1)]] tex_coord: vec2<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main([[location(0)]] tex_coord: vec2<f32>) -> [[location(0)]] vec4<f32> {
    return textureSample(frame_texture, frame_sampler, tex_coord);
}
"#;

pub struct WebGpuRenderer {
    instance: Option<GpuInstance>,
    adapter: Option<GpuAdapter>,
    device: Option<GpuDevice>,
    queue: Option<GpuQueue>,
    surface: Option<GpuSurface>,
    config: Option<GpuSurfaceConfiguration>,
    render_pipeline: Option<GpuRenderPipeline>,
    vertex_buffer: Option<GpuBuffer>,
    framebuffers: Vec<GpuTexture>,
    current_frame: usize,
    width: u32,
    height: u32,
}

impl WebGpuRenderer {
    pub fn new() -> Result<WebGpuRenderer, JsValue> {
        Ok(WebGpuRenderer {
            instance: None,
            adapter: None,
            device: None,
            queue: None,
            surface: None,
            config: None,
            render_pipeline: None,
            vertex_buffer: None,
            framebuffers: Vec::new(),
            current_frame: 0,
            width: 0,
            height: 0,
        })
    }

    pub fn init_surface(&mut self) -> Result<(), JsValue> {
        Ok(())
    }

    pub fn render(&mut self, _framebuffer: &[u8], width: u32, height: u32) -> Result<(), JsValue> {
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub fn present(&mut self) -> Result<(), JsValue> {
        self.current_frame += 1;
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

pub struct Canvas2DRenderer {
    context: web_sys::CanvasRenderingContext2d,
    width: u32,
    height: u32,
}

impl Canvas2DRenderer {
    pub fn new(context: web_sys::CanvasRenderingContext2d, width: u32, height: u32) -> Self {
        Self { context, width, height }
    }

    pub fn render(&mut self, framebuffer: &[u8], width: u32, height: u32) {
        let _ = self.context;
        let _ = framebuffer;
        let _ = width;
        let _ = height;
    }
}

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub type RendererBackend = WebGpuRenderer;
    } else {
        pub type RendererBackend = Canvas2DRenderer;
    }
}

pub fn create_render_pipeline(
    _device: &GpuDevice,
    _layout: &GpuPipelineLayout,
    _shader_module: &GpuShaderModule,
) -> Result<GpuRenderPipeline, JsValue> {
    Ok(GpuRenderPipeline::new())
}
