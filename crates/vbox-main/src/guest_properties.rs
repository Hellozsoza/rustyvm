use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Guest property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestProperty {
    pub name: String,
    pub value: String,
    pub timestamp: i64,
    pub flags: u32,
}

impl GuestProperty {
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            timestamp: 0,
            flags: 0,
        }
    }
}

/// Guest property service.
pub struct GuestPropertyService {
    properties: RwLock<HashMap<String, GuestProperty>>,
}

impl GuestPropertyService {
    pub fn new() -> Self {
        Self {
            properties: RwLock::new(HashMap::new()),
        }
    }

    pub fn set(&self, name: &str, value: &str) -> VBoxResult<()> {
        let prop = GuestProperty::new(name.to_string(), value.to_string());
        self.properties.write().insert(name.to_string(), prop);
        Ok(())
    }

    pub fn get(&self, name: &str) -> VBoxResult<Option<String>> {
        Ok(self.properties.read().get(name).map(|p| p.value.clone()))
    }

    pub fn enumerate(&self) -> VBoxResult<Vec<GuestProperty>> {
        Ok(self.properties.read().values().cloned().collect())
    }

    pub fn delete(&self, name: &str) -> VBoxResult<()> {
        self.properties.write().remove(name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guest_properties() {
        let service = GuestPropertyService::new();
        service.set("Test/Value", "hello").unwrap();
        let val = service.get("Test/Value").unwrap();
        assert_eq!(val, Some("hello".to_string()));
    }
}
