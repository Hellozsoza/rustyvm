use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Guest event types for VMMDev.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuestEvent {
    /// Mouse movement event.
    MouseMove { x: i32, y: i32 },
    /// Key down event.
    KeyDown { key_code: u32 },
    /// Key up event.
    KeyUp { key_code: u32 },
    /// Clipboard changed event.
    ClipboardChanged { text: String },
    /// Screen resolution changed event.
    ScreenChanged { width: u32, height: u32, bpp: u32 },
    /// Mouse capture state changed.
    MouseCaptureChanged { captured: bool },
}

/// HGCM (Host-Guest Communication) service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HgcService {
    /// Service name.
    pub name: String,
    /// Service version.
    pub version: u32,
    /// Service ID.
    pub service_id: u32,
}

/// VMMDev device for guest-host communication.
pub struct VmmDevDevice {
    /// HGCM services.
    pub hgcm_services: Vec<HgcService>,
    /// Guest events queue.
    pub events: Vec<GuestEvent>,
    /// Whether clipboard sharing is enabled.
    pub clipboard_enabled: bool,
    /// Whether shared folders are enabled.
    pub shared_folders_enabled: bool,
    /// Monitor width.
    pub monitor_width: u32,
    /// Monitor height.
    pub monitor_height: u32,
    /// Monitor bits per pixel.
    pub monitor_bpp: u32,
}

impl VmmDevDevice {
    /// Creates a new VMMDev device.
    pub fn new() -> Self {
        Self {
            hgcm_services: Vec::new(),
            events: Vec::new(),
            clipboard_enabled: false,
            shared_folders_enabled: false,
            monitor_width: 1024,
            monitor_height: 768,
            monitor_bpp: 32,
        }
    }

    /// Reports a monitor mode change to the host.
    pub fn report_monitor_mode(&mut self, width: u32, height: u32, bpp: u32) {
        self.monitor_width = width;
        self.monitor_height = height;
        self.monitor_bpp = bpp;
        self.events.push(GuestEvent::ScreenChanged {
            width,
            height,
            bpp,
        });
    }

    /// Handles a host card insert event.
    pub fn host_card_insert(&mut self) {
        self.events.push(GuestEvent::MouseCaptureChanged { captured: true });
    }

    /// Handles a host card remove event.
    pub fn host_card_remove(&mut self) {
        self.events.push(GuestEvent::MouseCaptureChanged { captured: false });
    }

    /// Sets the host screen parameters.
    pub fn set_host_screen(&mut self, width: u32, height: u32, bpp: u32) {
        self.monitor_width = width;
        self.monitor_height = height;
        self.monitor_bpp = bpp;
    }

    /// Returns the list of supported HGCM services.
    pub fn query_interface(&self) -> Vec<HgcService> {
        self.hgcm_services.clone()
    }
}

impl Default for VmmDevDevice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vmmdev_new() {
        let dev = VmmDevDevice::new();
        assert!(!dev.clipboard_enabled);
        assert!(dev.hgcm_services.is_empty());
    }

    #[test]
    fn test_report_monitor_mode() {
        let mut dev = VmmDevDevice::new();
        dev.report_monitor_mode(1920, 1080, 32);
        assert_eq!(dev.monitor_width, 1920);
        assert_eq!(dev.monitor_height, 1080);
        assert_eq!(dev.events.len(), 1);
    }
}
