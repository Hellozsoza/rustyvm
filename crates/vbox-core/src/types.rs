#![allow(non_camel_case_types)]

pub type uint8 = u8;
pub type uint16 = u16;
pub type uint32 = u32;
pub type uint64 = u64;
pub type int8 = i8;
pub type int16 = i16;
pub type int32 = i32;
pub type int64 = i64;
pub type byte = u8;
pub type boolean = bool;
pub type char8 = char;
pub type usize_t = usize;
pub type ssize_t = isize;

#[repr(C)]
pub struct RTPtr<T = ()> {
    pub ptr: *mut T,
}

#[repr(C)]
pub struct RTSize(pub usize);

#[repr(C)]
pub struct RTCapacity(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTThreadState {
    Running,
    Waiting,
    Terminated,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTThreadType {
    Main,
    Worker,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTLogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTSemType {
    Mutex,
    Semaphore,
    CriticalSection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTFileAccess {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTFileOpenAction {
    Open,
    Create,
    OpenOrCreate,
    CreateNew,
    Replace,
}
