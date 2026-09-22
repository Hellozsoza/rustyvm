use parking_lot::Mutex;
use smallvec::SmallVec;
use serde::{Deserialize, Serialize};
use std::fmt;

/// VGA graphics display modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VgaGraphicsMode {
    TextMode,
    GraphicsMode,
    VGACompatible,
    SVGA,
}

/// VGA state containing registers and configuration.
#[derive(Debug, Clone)]
pub struct VgaState {
    /// CRTC registers (256 bytes).
    pub crtc_regs: [u8; 256],
    /// Graphics controller registers.
    pub grc_regs: [u8; 8],
    /// Sequencer registers.
    pub seq_regs: [u8; 5],
    /// Attribute controller registers.
    pub att_regs: [u8; 21],
    /// Graphics mode.
    pub graphics_mode: VgaGraphicsMode,
    /// Whether display is enabled.
    pub display_enabled: bool,
    /// Current video mode index.
    pub mode_index: u8,
}

impl Default for VgaState {
    fn default() -> Self {
        Self {
            crtc_regs: [0; 256],
            grc_regs: [0; 8],
            seq_regs: [0; 5],
            att_regs: [0; 21],
            graphics_mode: VgaGraphicsMode::TextMode,
            display_enabled: false,
            mode_index: 0,
        }
    }
}

/// VGA device emulation.
pub struct VgaDevice {
    /// Framebuffer pixel buffer (RGBA, 1920x1080).
    pub framebuffer: Box<[u32]>,
    /// Framebuffer width.
    pub width: u32,
    /// Framebuffer height.
    pub height: u32,
    /// Bits per pixel.
    pub bpp: u32,
    /// MMIO base address.
    pub mmio_base: u64,
    /// MMIO size.
    pub mmio_size: u64,
    /// VGA state registers.
    pub state: VgaState,
    /// Whether the framebuffer is dirty and needs re-rendering.
    pub dirty: bool,
}

impl VgaDevice {
    /// Creates a new VGA device with a 4MB framebuffer.
    pub fn new() -> Self {
        let width = 720u32;
        let height = 400u32;
        let fb_size = (width * height) as usize;
        Self {
            framebuffer: vec![0u32; fb_size].into_boxed_slice(),
            width,
            height,
            bpp: 32,
            mmio_base: 0xD0000000,
            mmio_size: 0x00800000,
            state: VgaState::default(),
            dirty: true,
        }
    }

    /// Writes a value to a VGA register.
    pub fn write_register(&mut self, reg_index: u8, value: u8) {
        match reg_index {
            0..=255 => {
                self.state.crtc_regs[reg_index as usize] = value;
                self.dirty = true;
            }
            _ => {}
        }
    }

    /// Reads a value from a VGA register.
    pub fn read_register(&self, reg_index: u8) -> u8 {
        self.state.crtc_regs.get(reg_index as usize).copied().unwrap_or(0)
    }

    /// Sets the display mode.
    pub fn set_mode(&mut self, mode: VgaGraphicsMode, width: u32, height: u32) {
        self.state.graphics_mode = mode;
        self.width = width;
        self.height = height;
        let fb_size = (width * height) as usize;
        if fb_size > self.framebuffer.len() {
            self.framebuffer = vec![0u32; fb_size].into_boxed_slice();
        } else {
            for pixel in self.framebuffer.iter_mut() {
                *pixel = 0;
            }
        }
        self.dirty = true;
    }

    /// Renders the framebuffer to a RGBA byte buffer.
    pub fn render(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity((self.width * self.height * 4) as usize);
        for &pixel in self.framebuffer.iter() {
            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;
            let a = ((pixel >> 24) & 0xFF) as u8;
            buf.extend_from_slice(&[r, g, b, a]);
        }
        buf
    }

    /// Reads from a VGA MMIO register.
    pub fn mmio_read(&self, offset: u64) -> u32 {
        let idx = (offset % self.mmio_size) as usize;
        if idx < self.framebuffer.len() {
            let pixel = self.framebuffer[idx];
            pixel.to_le()
        } else {
            0xFFFFFFFF
        }
    }

    /// Writes to a VGA MMIO register.
    pub fn mmio_write(&mut self, offset: u64, value: u32) {
        let idx = (offset % self.mmio_size) as usize;
        if idx < self.framebuffer.len() {
            self.framebuffer[idx] = value;
            self.dirty = true;
        }
    }

    /// Marks the framebuffer as dirty.
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }
}

impl Default for VgaDevice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vga_new() {
        let vga = VgaDevice::new();
        assert_eq!(vga.width, 720);
        assert_eq!(vga.height, 400);
        assert!(vga.dirty);
    }

    #[test]
    fn test_vga_set_mode() {
        let mut vga = VgaDevice::new();
        vga.set_mode(VgaGraphicsMode::GraphicsMode, 1920, 1080);
        assert_eq!(vga.width, 1920);
        assert_eq!(vga.height, 1080);
        assert_eq!(vga.state.graphics_mode, VgaGraphicsMode::GraphicsMode);
    }
}
