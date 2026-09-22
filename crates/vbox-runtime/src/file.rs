use vbox_core::VBoxError;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub struct RTFile {
    path: String,
    handle: Option<std::fs::File>,
}

impl RTFile {
    pub fn open(path: &str) -> Result<Self, VBoxError> {
        match std::fs::File::open(path) {
            Ok(file) => Ok(RTFile { path: path.to_string(), handle: Some(file) }),
            Err(_) => Err(VBoxError::FileNotFound),
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, VBoxError> {
        match &mut self.handle {
            Some(f) => Ok(f.read(buf).map_err(|_| VBoxError::Fail)?),
            None => Err(VBoxError::InvalidState),
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<usize, VBoxError> {
        match &mut self.handle {
            Some(f) => Ok(f.write(buf).map_err(|_| VBoxError::Fail)?),
            None => Err(VBoxError::InvalidState),
        }
    }

    pub fn close(self) -> Result<(), VBoxError> {
        drop(self.handle);
        Ok(())
    }
}
