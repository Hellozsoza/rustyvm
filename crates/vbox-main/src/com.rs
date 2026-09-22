#![allow(non_camel_case_types)]

use vbox_core::{VBoxResult, VBoxError};
use uuid::Uuid;

pub trait IUnknown {
    fn query_interface(&self, iid: &Uuid) -> Option<Box<dyn IUnknown>>;
    fn add_ref(&self) -> u32;
    fn release(&self) -> u32;
}

pub trait IDispatch: IUnknown {
    fn get_member_name(&self, dispid: i32) -> VBoxResult<String>;
    fn invoke(&self, dispid: i32, args: &[VBoxVariant]) -> VBoxResult<VBoxVariant>;
}

#[derive(Debug, Clone)]
pub enum VBoxVariant {
    Empty,
    Null,
    Bool(bool),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    Blob(Vec<u8>),
    Array(Vec<VBoxVariant>),
    Object(Box<dyn IUnknown>),
}

impl VBoxVariant {
    pub fn as_bool(&self) -> Option<bool> {
        if let VBoxVariant::Bool(v) = self { Some(*v) } else { None }
    }

    pub fn as_i32(&self) -> Option<i32> {
        if let VBoxVariant::I32(v) = self { Some(*v) } else { None }
    }

    pub fn as_u32(&self) -> Option<u32> {
        if let VBoxVariant::U32(v) = self { Some(*v) } else { None }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if let VBoxVariant::I64(v) = self { Some(*v) } else { None }
    }

    pub fn as_u64(&self) -> Option<u64> {
        if let VBoxVariant::U64(v) = self { Some(*v) } else { None }
    }

    pub fn as_f64(&self) -> Option<f64> {
        if let VBoxVariant::F64(v) = self { Some(*v) } else { None }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let VBoxVariant::String(s) = self { Some(s) } else { None }
    }

    pub fn as_blob(&self) -> Option<&[u8]> {
        if let VBoxVariant::Blob(b) = self { Some(b) } else { None }
    }
}

pub struct ComObject {
    pub iid: Uuid,
    pub refs: u32,
}

impl ComObject {
    pub fn new(iid: Uuid) -> Self {
        Self { iid, refs: 1 }
    }

    pub fn query_interface<T: 'static>(&self) -> Option<&T> {
        None
    }
}

pub fn com_initialize() -> VBoxResult<()> {
    Ok(())
}

pub fn com_uninitialize() {
}

pub fn com_create_instance(clsid: &Uuid) -> VBoxResult<Box<dyn IUnknown>> {
    Err(VBoxError::NotImplemented)
}
