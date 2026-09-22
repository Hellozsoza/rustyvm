use std::fmt;
use std::string::ToString;

pub type VBoxResult<T> = Result<T, VBoxError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VBoxError {
    NoMemory,
    InvalidPointer,
    NotImplemented,
    Timeout,
    AlreadyExists,
    NotFound,
    AccessDenied,
    UnrecognizedMedia,
    HandleEof,
    BadVersion,
    BadPath,
    BadFile,
    BadString,
    BadParam,
    BadState,
    BadFlags,
    BadPointer,
    BufferOverflow,
    InsufficientBuffer,
    NotSupported,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NetworkUnreachable,
    HostUnreachable,
    TimedOut,
    WouldBlock,
    InProgress,
    AlreadyInitialized,
    NotInitialized,
    NoDevice,
    NoDriver,
    NoService,
    PermissionDenied,
    QuotaExceeded,
    DiskFull,
    InvalidHandle,
    InvalidVersion,
    InvalidFormat,
    InvalidLength,
    InvalidOffset,
    InvalidSize,
    InvalidType,
    InvalidValue,
    EndOfFile,
    InvalidRequest,
    Unknown(String),
}

impl fmt::Display for VBoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VBoxError::NoMemory => write!(f, "Out of memory"),
            VBoxError::InvalidPointer => write!(f, "Invalid pointer"),
            VBoxError::NotImplemented => write!(f, "Not implemented"),
            VBoxError::Timeout => write!(f, "Timeout"),
            VBoxError::AlreadyExists => write!(f, "Already exists"),
            VBoxError::NotFound => write!(f, "Not found"),
            VBoxError::AccessDenied => write!(f, "Access denied"),
            VBoxError::UnrecognizedMedia => write!(f, "Unrecognized media"),
            VBoxError::HandleEof => write!(f, "Handle end of file"),
            VBoxError::BadVersion => write!(f, "Bad version"),
            VBoxError::BadPath => write!(f, "Bad path"),
            VBoxError::BadFile => write!(f, "Bad file"),
            VBoxError::BadString => write!(f, "Bad string"),
            VBoxError::BadParam => write!(f, "Bad parameter"),
            VBoxError::BadState => write!(f, "Bad state"),
            VBoxError::BadFlags => write!(f, "Bad flags"),
            VBoxError::BadPointer => write!(f, "Bad pointer"),
            VBoxError::BufferOverflow => write!(f, "Buffer overflow"),
            VBoxError::InsufficientBuffer => write!(f, "Insufficient buffer"),
            VBoxError::NotSupported => write!(f, "Not supported"),
            VBoxError::ConnectionRefused => write!(f, "Connection refused"),
            VBoxError::ConnectionReset => write!(f, "Connection reset"),
            VBoxError::ConnectionAborted => write!(f, "Connection aborted"),
            VBoxError::NetworkUnreachable => write!(f, "Network unreachable"),
            VBoxError::HostUnreachable => write!(f, "Host unreachable"),
            VBoxError::TimedOut => write!(f, "Timed out"),
            VBoxError::WouldBlock => write!(f, "Would block"),
            VBoxError::InProgress => write!(f, "In progress"),
            VBoxError::AlreadyInitialized => write!(f, "Already initialized"),
            VBoxError::NotInitialized => write!(f, "Not initialized"),
            VBoxError::NoDevice => write!(f, "No device"),
            VBoxError::NoDriver => write!(f, "No driver"),
            VBoxError::NoService => write!(f, "No service"),
            VBoxError::PermissionDenied => write!(f, "Permission denied"),
            VBoxError::QuotaExceeded => write!(f, "Quota exceeded"),
            VBoxError::DiskFull => write!(f, "Disk full"),
            VBoxError::InvalidHandle => write!(f, "Invalid handle"),
            VBoxError::InvalidVersion => write!(f, "Invalid version"),
            VBoxError::InvalidFormat => write!(f, "Invalid format"),
            VBoxError::InvalidLength => write!(f, "Invalid length"),
            VBoxError::InvalidOffset => write!(f, "Invalid offset"),
            VBoxError::InvalidSize => write!(f, "Invalid size"),
            VBoxError::InvalidType => write!(f, "Invalid type"),
            VBoxError::InvalidValue => write!(f, "Invalid value"),
            VBoxError::EndOfFile => write!(f, "End of file"),
            VBoxError::InvalidRequest => write!(f, "Invalid request"),
            VBoxError::Unknown(s) => write!(f, "Unknown error: {}", s),
        }
    }
}

impl core::error::Error for VBoxError {}

impl VBoxError {
    pub fn from_status(status: i32) -> Self {
        match status {
            0 => VBoxError::NotImplemented,
            -1 => VBoxError::InvalidPointer,
            -2 => VBoxError::NoMemory,
            -3 => VBoxError::NotImplemented,
            -4 => VBoxError::Timeout,
            -5 => VBoxError::AlreadyExists,
            -6 => VBoxError::NotFound,
            -7 => VBoxError::AccessDenied,
            -8 => VBoxError::UnrecognizedMedia,
            -9 => VBoxError::HandleEof,
            -10 => VBoxError::AccessDenied,
            _ => VBoxError::Unknown(format!("status code {}", status)),
        }
    }

    pub fn to_status(&self) -> i32 {
        match self {
            VBoxError::NoMemory => -2,
            VBoxError::InvalidPointer => -1,
            VBoxError::NotImplemented => -3,
            VBoxError::Timeout => -4,
            VBoxError::AlreadyExists => -5,
            VBoxError::NotFound => -6,
            VBoxError::AccessDenied => -7,
            VBoxError::UnrecognizedMedia => -8,
            VBoxError::HandleEof => -9,
            _ => -1,
        }
    }
}

impl From<core::ffi::FromBytesWithNulError> for VBoxError {
    fn from(_: core::ffi::FromBytesWithNulError) -> Self {
        VBoxError::BadString
    }
}

impl From<core::str::Utf8Error> for VBoxError {
    fn from(_: core::str::Utf8Error) -> Self {
        VBoxError::BadString
    }
}

impl From<String> for VBoxError {
    fn from(_: String) -> Self {
        VBoxError::Unknown("string conversion error".to_string())
    }
}
