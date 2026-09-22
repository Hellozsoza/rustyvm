use thiserror::Error;

#[derive(Error, Debug)]
pub enum VBoxError {
    #[error("success")]
    Success,
    #[error("failure")]
    Fail,
    #[error("invalid parameter")]
    InvalidParameter,
    #[error("out of memory")]
    OutOfMemory,
    #[error("not supported")]
    NotSupported,
    #[error("unknown error")]
    Unknown,
    #[error("file not found")]
    FileNotFound,
    #[error("access denied")]
    AccessDenied,
    #[error("timeout")]
    Timeout,
    #[error("invalid state")]
    InvalidState,
    #[error("version mismatch")]
    VersionMismatch,
    #[error("not implemented")]
    NotImplemented,
    #[error("invalid {0}")]
    InvalidParam(String),
}

pub type VBoxResult<T> = Result<T, VBoxError>;

pub type RTGCPHYS = u64;
pub type RTGCPTR = u64;
pub type RTSEL = u16;
pub type RTUINT32 = u32;
pub type RTUINT64 = u64;

pub const RT_OK: i32 = 0;
pub const RT_E_FAIL: i32 = -1;
pub const RT_E_INVALID_POINTER: i32 = -2;
pub type VBOX_STATUS = VBoxError;
pub const PAGE_SIZE: usize = 4096;

pub mod status {
    pub const VINF_SUCCESS: i32 = 0;
    pub const VERR_FAILURE: i32 = -1;
    pub const VERR_INVALID_PARAMETER: i32 = -2;
    pub const VERR_OUT_OF_MEMORY: i32 = -3;
    pub const VERR_NOT_SUPPORTED: i32 = -4;
    pub const VERR_UNKNOWN: i32 = -5;
    pub const VERR_FILE_NOT_FOUND: i32 = -6;
    pub const VERR_ACCESS_DENIED: i32 = -7;
    pub const VERR_TIMEOUT: i32 = -8;
    pub const VERR_INVALID_STATE: i32 = -9;
    pub const VERR_VERSION_MISMATCH: i32 = -10;
}

pub fn rt_status_to_result(status: i32) -> VBoxResult<()> {
    if status == RT_OK {
        Ok(())
    } else {
        Err(VBoxError::Fail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vbox_error_success() {
        assert!(matches!(VBoxError::Success, VBoxError::Success));
    }

    #[test]
    fn test_vbox_result_success() {
        let result: VBoxResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }
}
