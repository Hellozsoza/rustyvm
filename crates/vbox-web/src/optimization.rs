use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use uuid::Uuid;

thread_local! {
    static PERFORMANCE_DATA: RefCell<PerformanceData> = RefCell::new(PerformanceData::new());
}

struct PerformanceData {
    fps_history: Vec<f64>,
    frame_times: Vec<f64>,
    cpu_usage: f64,
    memory_usage: u64,
    last_frame_time: f64,
    frame_count: u64,
}

impl PerformanceData {
    fn new() -> Self {
        Self {
            fps_history: Vec::new(),
            frame_times: Vec::new(),
            cpu_usage: 0.0,
            memory_usage: 0,
            last_frame_time: 0.0,
            frame_count: 0,
        }
    }
}

pub struct WasmOptimizer {
    optimization_level: u8,
    enable_lto: bool,
    strip_debug: bool,
}

impl WasmOptimizer {
    pub fn new() -> Self {
        WasmOptimizer {
            optimization_level: 4,
            enable_lto: true,
            strip_debug: true,
        }
    }

    pub fn optimize(&mut self, wasm_data: &mut Vec<u8>) -> Result<Vec<u8>, String> {
        if self.strip_debug {
            wasm_data.retain(|&b| b != 0x00);
        }
        Ok(wasm_data.clone())
    }
}

pub struct PerformanceMonitor {
    fps: u32,
    cpu_usage: f64,
    memory_usage: u64,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            fps: 0,
            cpu_usage: 0.0,
            memory_usage: 0,
        }
    }

    pub fn track_frame(&mut self, frame_time_ms: f64) {
        self.fps = (1000.0 / frame_time_ms.max(1.0)) as u32;
        self.frame_count += 1;
    }

    pub fn get_fps(&self) -> u32 { self.fps }
    pub fn get_cpu_usage(&self) -> f64 { self.cpu_usage }
    pub fn get_memory_usage(&self) -> u64 { self.memory_usage }
}

pub struct MemoryOptimizer {
    guest_ram_size: usize,
    ram_pool: Vec<u8>,
    pre_allocated: bool,
}

impl MemoryOptimizer {
    pub fn new(guest_ram_size: usize) -> Self {
        let mut ram_pool = Vec::with_capacity(guest_ram_size);
        unsafe { ram_pool.set_len(guest_ram_size); }
        MemoryOptimizer {
            guest_ram_size,
            ram_pool,
            pre_allocated: true,
        }
    }

    pub fn pre_allocate(&mut self) {
        self.pre_allocated = true;
    }

    pub fn allocate_page(&mut self, page_num: usize) -> Option<&mut [u8]> {
        let start = page_num * 4096;
        let end = start + 4096;
        if end <= self.ram_pool.len() {
            Some(&mut self.ram_pool[start..end])
        } else {
            None
        }
    }
}

pub struct FrameSkipper {
    frames_skipped: u64,
    frames_rendered: u64,
    target_fps: u32,
    max_skip: u32,
}

impl FrameSkipper {
    pub fn new(target_fps: u32) -> Self {
        FrameSkipper {
            frames_skipped: 0,
            frames_rendered: 0,
            target_fps,
            max_skip: 3,
        }
    }

    pub fn should_skip(&mut self, elapsed_ms: f64) -> bool {
        let target_ms = 1000.0 / self.target_fps as f64;
        if elapsed_ms > target_ms * 2.0 {
            self.frames_skipped += 1;
            true
        } else {
            self.frames_rendered += 1;
            false
        }
    }
}

pub struct DynamicResize {
    min_width: u32,
    min_height: u32,
    max_width: u32,
    max_height: u32,
    current_width: u32,
    current_height: u32,
    quality_level: u32,
}

impl DynamicResize {
    pub fn new(min_width: u32, min_height: u32, max_width: u32, max_height: u32) -> Self {
        DynamicResize {
            min_width,
            min_height,
            max_width,
            max_height,
            current_width: min_width,
            current_height: min_height,
            quality_level: 1,
        }
    }

    pub fn adjust(&mut self, perf_score: f64) {
        if perf_score < 30.0 && self.current_width > self.min_width {
            self.current_width /= 2;
            self.current_height /= 2;
            self.quality_level = self.quality_level.saturating_sub(1);
        } else if perf_score > 80.0 && self.current_width < self.max_width {
            self.current_width = (self.current_width * 3) / 2;
            self.current_height = (self.current_height * 3) / 2;
            self.quality_level = self.quality_level.saturating_add(1);
        }
    }

    pub fn get_width(&self) -> u32 { self.current_width }
    pub fn get_height(&self) -> u32 { self.current_height }
}
