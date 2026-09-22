use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// PS/2 keyboard device.
pub struct Ps2Keyboard {
    /// Keyboard scan code buffer.
    pub scan_code_buffer: Vec<u8>,
    /// Whether keyboard is enabled.
    pub enabled: bool,
    /// LED states.
    pub leds: Ps2Leds,
}

/// PS/2 LED states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ps2Leds {
    None = 0,
    NumLock = 1,
    CapsLock = 2,
    ScrollLock = 4,
}

impl Default for Ps2Keyboard {
    fn default() -> Self {
        Self {
            scan_code_buffer: Vec::new(),
            enabled: true,
            leds: Ps2Leds::None,
        }
    }
}

impl Ps2Keyboard {
    /// Creates a new PS/2 keyboard.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sends a scan code to the keyboard.
    pub fn send_scan_code(&mut self, code: u8) {
        self.scan_code_buffer.push(code);
    }
}

/// PS/2 mouse device.
pub struct Ps2Mouse {
    /// Mouse movement delta X.
    pub delta_x: i8,
    /// Mouse movement delta Y.
    pub delta_y: i8,
    /// Mouse button states.
    pub buttons: u8,
    /// Whether mouse is enabled.
    pub enabled: bool,
}

impl Default for Ps2Mouse {
    fn default() -> Self {
        Self {
            delta_x: 0,
            delta_y: 0,
            buttons: 0,
            enabled: true,
        }
    }
}

impl Ps2Mouse {
    /// Creates a new PS/2 mouse.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates mouse movement.
    pub fn update(&mut self, dx: i8, dy: i8, buttons: u8) {
        self.delta_x = dx;
        self.delta_y = dy;
        self.buttons = buttons;
    }
}

/// USB tablet device for absolute positioning.
pub struct UsbTablet {
    /// X position.
    pub x: u32,
    /// Y position.
    pub y: u32,
    /// Pressure value.
    pub pressure: u32,
    /// Whether tablet is active.
    pub active: bool,
}

impl Default for UsbTablet {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            pressure: 0,
            active: false,
        }
    }
}

impl UsbTablet {
    /// Creates a new USB tablet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tablet position.
    pub fn set_position(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
        self.active = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps2_keyboard_new() {
        let kb = Ps2Keyboard::new();
        assert!(kb.enabled);
    }

    #[test]
    fn test_usb_tablet_position() {
        let mut tablet = UsbTablet::new();
        tablet.set_position(100, 200);
        assert_eq!(tablet.x, 100);
        assert_eq!(tablet.y, 200);
    }
}
