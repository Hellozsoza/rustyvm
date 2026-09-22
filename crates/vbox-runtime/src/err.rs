use vbox_core::VBoxError;

#[derive(Debug)]
pub enum RTERR {
    VInfSuccess,
    VErrFailure,
    VErrInvalidParameter,
    VErrOutOfMemory,
    VErrNotSupported,
    VErrUnknown,
    VErrFileNotFound,
    VErrAccessDenied,
    VErrTimeout,
    VErrInvalidState,
    VErrVersionMismatch,
}

impl std::fmt::Display for RTERR {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RTERR::VInfSuccess => write!(f, "Success"),
            RTERR::VErrFailure => write!(f, "Failure"),
            RTERR::VErrInvalidParameter => write!(f, "Invalid Parameter"),
            RTERR::VErrOutOfMemory => write!(f, "Out of Memory"),
            RTERR::VErrNotSupported => write!(f, "Not Supported"),
            RTERR::VErrUnknown => write!(f, "Unknown"),
            RTERR::VErrFileNotFound => write!(f, "File Not Found"),
            RTERR::VErrAccessDenied => write!(f, "Access Denied"),
            RTERR::VErrTimeout => write!(f, "Timeout"),
            RTERR::VErrInvalidState => write!(f, "Invalid State"),
            RTERR::VErrVersionMismatch => write!(f, "Version Mismatch"),
        }
    }
}

impl From<VBoxError> for RTERR {
    fn from(e: VBoxError) -> Self {
        match e {
            VBoxError::Success => RTERR::VInfSuccess,
            VBoxError::Fail => RTERR::VErrFailure,
            VBoxError::InvalidParameter => RTERR::VErrInvalidParameter,
            VBoxError::OutOfMemory => RTERR::VErrOutOfMemory,
            VBoxError::NotSupported => RTERR::VErrNotSupported,
            VBoxError::Unknown => RTERR::VErrUnknown,
            VBoxError::FileNotFound => RTERR::VErrFileNotFound,
            VBoxError::AccessDenied => RTERR::VErrAccessDenied,
            VBoxError::Timeout => RTERR::VErrTimeout,
            VBoxError::InvalidState => RTERR::VErrInvalidState,
            VBoxError::VersionMismatch => RTERR::VErrVersionMismatch,
            VBoxError::NotImplemented => RTERR::VErrNotSupported,
            VBoxError::InvalidParam(_) => RTERR::VErrInvalidParameter,
        }
    }
}
