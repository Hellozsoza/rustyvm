use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::{RwLock, Mutex};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// COM interface identifier.
pub type IID = [u32; 4];

/// COM interface identifiers.
pub const IID_IUNKNOWN: IID = [0x00000000, 0x00000000, 0x00000000, 0x00000000];
pub const IID_IDISPATCH: IID = [0x00020400, 0x00000000, 0x00000000, 0x00000000];
pub const IID_IVIRTUALBOX: IID = [0x541080C0, 0x4F1E4ABB, 0x96AF1858, 0x00000000];
pub const IID_IMACHINE: IID = [0x337A3F50, 0x1E534CD8, 0xB4E51E95, 0x00000000];
pub const IID_ICONSOLERE: IID = [0x73A484D0, 0x64B9437C, 0xB7A7C941, 0x00000000];
pub const IID_IMEDEV: IID = [0x337A3F50, 0x1E534CD8, 0xB4E51E95, 0x00000001];
pub const IID_IHOST: IID = [0x7493A9A0, 0x64B9437C, 0xB7A7C941, 0x00000000];
pub const IID_IUSBCONTROLLER: IID = [0x7B8B0A20, 0x64B9437C, 0xB7A7C941, 0x00000000];
pub const IID_ISTORAGECONTROLLER: IID = [0x2B7F3A00, 0x64B9437C, 0xB7A7C941, 0x00000000];
pub const IID_IBOOL: IID = [0x00000001, 0x00000000, 0x00000000, 0x00000000];
pub const IID_IVIRTUALBOXEVENTS: IID = [0x541080C1, 0x4F1E4ABB, 0x96AF1858, 0x00000000];

/// COM interface trait.
pub trait ComInterface {
    fn query_interface(&self, iid: &IID) -> Option<ComPtr>;
    fn add_ref(&self) -> u32;
    fn release(&self) -> u32;
}

/// IUnknown trait.
pub trait IUnknown: ComInterface {
    fn query_interface(&self, iid: &IID) -> Option<ComPtr>;
    fn add_ref(&self) -> u32;
    fn release(&self) -> u32;
}

/// IDispatch trait.
pub trait IDispatch: ComInterface {
    fn get_disp_id(&self, name: &str) -> VBoxResult<i32>;
    fn invoke(&self, disp_id: i32, flags: u32) -> VBoxResult<()>;
    fn get_type_info(&self, index: u32) -> VBoxResult<()>;
    fn get_type_info_count(&self) -> u32;
}

/// COM object with reference counting.
pub struct ComObject {
    ref_count: Mutex<u32>,
    _private: (),
}

impl ComObject {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            ref_count: Mutex::new(1),
            _private: (),
        })
    }

    pub fn add_ref(&self) -> u32 {
        let mut count = self.ref_count.lock();
        *count += 1;
        *count
    }

    pub fn release(&self) -> u32 {
        let mut count = self.ref_count.lock();
        *count -= 1;
        *count
    }

    pub fn ref_count(&self) -> u32 {
        *self.ref_count.lock()
    }
}

/// COM smart pointer.
pub struct ComPtr {
    obj: Option<Arc<ComObject>>,
}

impl ComPtr {
    pub fn new(obj: Arc<ComObject>) -> Self {
        obj.add_ref();
        Self { obj: Some(obj) }
    }

    pub fn query_interface(&self, iid: &IID) -> VBoxResult<ComPtr> {
        if iid == &IID_IUNKNOWN || iid == &IID_IDISPATCH {
            if let Some(obj) = &self.obj {
                Ok(ComPtr::new(obj.clone()))
            } else {
                Err(VBoxError::NotInitialized)
            }
        } else {
            Err(VBoxError::NotImplemented)
        }
    }

    pub fn add_ref(&self) -> u32 {
        self.obj.as_ref().map(|o| o.add_ref()).unwrap_or(0)
    }

    pub fn release(&self) -> u32 {
        self.obj.as_ref().map(|o| o.release()).unwrap_or(0)
    }

    pub fn as_ptr(&self) -> Option<Arc<ComObject>> {
        self.obj.clone()
    }
}

impl Clone for ComPtr {
    fn clone(&self) -> Self {
        if let Some(obj) = &self.obj {
            Self::new(obj.clone())
        } else {
            Self { obj: None }
        }
    }
}

impl std::ops::Deref for ComPtr {
    type Target = Option<Arc<ComObject>>;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_com_ptr() {
        let obj = ComObject::new();
        let ptr = ComPtr::new(obj);
        assert!(ptr.obj.is_some());
    }
}
