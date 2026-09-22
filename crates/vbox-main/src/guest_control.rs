use serde::{Deserialize, Serialize};
use thiserror::Error;
use vbox_core::{VBoxError, VBoxResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Guest control service result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestControlResult {
    pub session_id: u32,
    pub object_id: u32,
    pub context_id: u32,
    pub rc: i32,
    pub data: Vec<u8>,
}

/// Guest process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestProcess {
    pub pid: u32,
    pub session_id: u32,
    pub command: String,
    pub arguments: Vec<String>,
    pub environment: Vec<String>,
    pub cwd: String,
    pub timeout: u64,
}

/// Guest control service.
pub struct GuestControlService {
    sessions: RwLock<HashMap<u32, GuestProcess>>,
    next_session_id: RwLock<u32>,
}

impl GuestControlService {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            next_session_id: RwLock::new(1),
        }
    }

    pub fn execute(&self, command: &str, arguments: &[String]) -> VBoxResult<GuestControlResult> {
        let session_id = *self.next_session_id.write();
        *self.next_session_id.write() = session_id.wrapping_add(1);

        let process = GuestProcess {
            pid: session_id,
            session_id,
            command: command.to_string(),
            arguments: arguments.to_vec(),
            environment: Vec::new(),
            cwd: String::from("/"),
            timeout: 0,
        };

        self.sessions.write().insert(session_id, process);

        let result = GuestControlResult {
            session_id,
            object_id: session_id,
            context_id: session_id,
            rc: 0,
            data: vec![],
        };
        Ok(result)
    }

    pub fn send_signal(&self, session_id: u32, signal: &str) -> VBoxResult<()> {
        let mut sessions = self.sessions.write();
        if let Some(process) = sessions.get_mut(&session_id) {
            process.arguments.push(signal.to_string());
        }
        Ok(())
    }

    pub fn get_session(&self, session_id: u32) -> Option<GuestProcess> {
        self.sessions.read().get(&session_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guest_control_execute() {
        let service = GuestControlService::new();
        let result = service.execute("echo", &["hello".to_string()]);
        assert!(result.is_ok());
    }
}
