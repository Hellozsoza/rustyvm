use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use crate::image::{DiskFormat, DiskImage, DiskImageInfo};
use parking_lot::RwLock;
use std::collections::HashMap;

/// iSCSI target connection parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IscsiConnectionParams {
    pub target_name: String,
    pub target_ip: String,
    pub target_port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub mutual_username: Option<String>,
    pub mutual_password: Option<String>,
    pub lun: u32,
    pub chap_algorithm: Option<String>,
}

impl Default for IscsiConnectionParams {
    fn default() -> Self {
        Self {
            target_name: String::new(),
            target_ip: String::new(),
            target_port: 3260,
            username: None,
            password: None,
            mutual_username: None,
            mutual_password: None,
            lun: 0,
            chap_algorithm: None,
        }
    }
}

/// iSCSI target structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IscsiTarget {
    pub params: IscsiConnectionParams,
    pub connected: bool,
    pub sector_count: u64,
    pub sector_size: u32,
    pub block_size: u32,
    pub info: DiskImageInfo,
}

#[derive(Error, Debug)]
pub enum IscsiError {
    #[error("Connection refused to {0}")]
    ConnectionRefused(String),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Target not found: {0}")]
    TargetNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl IscsiTarget {
    /// Open an iSCSI target connection.
    pub fn open(path: &str) -> VBoxResult<IscsiTarget> {
        let params = Self::parse_connection_string(path)?;
        let target = Self {
            params: params.clone(),
            connected: false,
            sector_count: 0,
            sector_size: 512,
            block_size: 512,
            info: DiskImageInfo::new(
                path.to_string(),
                DiskFormat::ISCSI,
                0,
                512,
            ),
        };
        Ok(target)
    }

    /// Parse an iSCSI connection string.
    fn parse_connection_string(path: &str) -> VBoxResult<IscsiConnectionParams> {
        let mut params = IscsiConnectionParams::default();
        if path.starts_with("iscsi://") {
            let rest = &path[8..];
            if let Some(at_pos) = rest.find('@') {
                let credentials = &rest[..at_pos];
                let rest = &rest[at_pos + 1..];
                if let Some(colon_pos) = credentials.find(':') {
                    params.username = Some(credentials[..colon_pos].to_string());
                    params.password = Some(credentials[colon_pos + 1..].to_string());
                }
            } else {
                let target_part = rest;
                if let Some(colon_pos) = target_part.find(':') {
                    params.target_ip = target_part[..colon_pos].to_string();
                    let port_str = &target_part[colon_pos + 1..];
                    if let Some(port) = port_str.find('/') {
                        params.target_port = port_str[..port].parse().unwrap_or(3260);
                        let target_name = &port_str[port + 1..];
                        params.target_name = target_name.to_string();
                    } else {
                        params.target_ip = target_part.to_string();
                    }
                }
            }
        }
        Ok(params)
    }

    /// Connect to the iSCSI target.
    pub fn connect(&mut self) -> VBoxResult<()> {
        if self.params.target_ip.is_empty() {
            return Err(VBoxError::BadParam);
        }
        self.connected = true;
        self.sector_count = 1 << 30; // Simulated 1TB
        Ok(())
    }

    /// Disconnect from the iSCSI target.
    pub fn disconnect(&mut self) -> VBoxResult<()> {
        self.connected = false;
        self.sector_count = 0;
        Ok(())
    }
}

impl DiskImage for IscsiTarget {
    fn read_sector(&self, lba: u64) -> VBoxResult<Vec<u8>> {
        if !self.connected {
            return Err(VBoxError::NotInitialized);
        }
        let sector_size = 512usize;
        Ok(vec![0u8; sector_size])
    }

    fn write_sector(&mut self, lba: u64, data: &[u8]) -> VBoxResult<()> {
        if data.len() != 512 {
            return Err(VBoxError::BadParam);
        }
        Ok(())
    }

    fn get_sector_count(&self) -> u64 {
        self.sector_count
    }

    fn get_sector_size(&self) -> u32 {
        512
    }

    fn get_type(&self) -> DiskFormat {
        DiskFormat::ISCSI
    }

    fn close(self: Box<Self>) -> VBoxResult<()> {
        self.disconnect()?;
        Ok(())
    }

    fn get_info(&self) -> DiskImageInfo {
        self.info.clone()
    }

    fn is_open(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iscsi_string() {
        let result = IscsiTarget::open("iscsi://192.168.1.100:3260/target1");
        assert!(result.is_ok());
        let target = result.unwrap();
        assert_eq!(target.params.target_ip, "192.168.1.100");
    }
}
