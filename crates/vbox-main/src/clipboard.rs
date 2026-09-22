use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::RwLock;
use std::collections::HashMap;

/// Clipboard data format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardFormat {
    Text,
    Html,
    Rtf,
    UriList,
    Image,
    Files,
}

/// Clipboard data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub format: ClipboardFormat,
    pub data: Vec<u8>,
    pub mime_type: String,
}

/// Clipboard service.
pub struct ClipboardService {
    clipboard_data: RwLock<HashMap<String, ClipboardData>>,
}

impl ClipboardService {
    pub fn new() -> Self {
        Self {
            clipboard_data: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_data(&self, format: &str) -> VBoxResult<Option<ClipboardData>> {
        Ok(self.clipboard_data.read().get(format).cloned())
    }

    pub fn set_data(&self, format: String, data: Vec<u8>, mime_type: String) -> VBoxResult<()> {
        let clipboard_data = ClipboardData {
            format: match format.as_str() {
                "text/plain" => ClipboardFormat::Text,
                "text/html" => ClipboardFormat::Html,
                "text/rtf" => ClipboardFormat::Rtf,
                "text/uri-list" => ClipboardFormat::UriList,
                "image/png" => ClipboardFormat::Image,
                _ => ClipboardFormat::Text,
            },
            data,
            mime_type,
        };
        self.clipboard_data.write().insert(format, clipboard_data);
        Ok(())
    }

    pub fn clear(&self) {
        self.clipboard_data.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_service() {
        let service = ClipboardService::new();
        service.set_data("text/plain".to_string(), b"hello".to_vec(), "text/plain".to_string()).unwrap();
        let data = service.get_data("text/plain").unwrap();
        assert!(data.is_some());
        assert_eq!(data.unwrap().data, b"hello");
    }
}
