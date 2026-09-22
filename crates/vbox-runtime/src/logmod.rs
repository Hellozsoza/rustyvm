#![allow(non_camel_case_types)]

use parking_lot::Mutex;
use ::log::{Level, LevelFilter, Metadata, Record};
use vbox_core::RTLogLevel;

static LOG_LEVEL: Mutex<LevelFilter> = Mutex::new(LevelFilter::Debug);
static LOG_GROUP: Mutex<Option<String>> = Mutex::new(None);

pub const RTLOG_DEFAULT_FLAGS: u32 = 0x00000000;
pub const RTLOG_FLAGS_PREFIX: u32 = 0x00000001;
pub const RTLOG_FLAGS_THREAD: u32 = 0x00000002;
pub const RTLOG_FLAGS_TIMESTAMP: u32 = 0x00000004;
pub const RTLOG_FLAGS_STACK_TRACE: u32 = 0x00000008;

pub const RTLOG_GROUP_MAIN: &str = "MAIN";
pub const RTLOG_GROUP_VMM: &str = "VMM";
pub const RTLOG_GROUP_PDM: &str = "PDM";
pub const RTLOG_GROUP_EM: &str = "EM";
pub const RTLOG_GROUP_IEM: &str = "IEM";
pub const RTLOG_GROUP_PGM: &str = "PGM";
pub const RTLOG_GROUP_TM: &str = "TM";
pub const RTLOG_GROUP_NET: &str = "NET";
pub const RTLOG_GROUP_IO: &str = "IO";
pub const RTLOG_GROUP_STREAM: &str = "STREAM";
pub const RTLOG_GROUP_FILE: &str = "FILE";
pub const RTLOG_GROUP_MEM: &str = "MEM";
pub const RTLOG_GROUP_THREAD: &str = "THREAD";
pub const RTLOG_GROUP_SOCK: &str = "SOCKET";
pub const RTLOG_GROUP_CRYPT: &str = "CRYPT";
pub const RTLOG_GROUP_FS: &str = "FS";
pub const RTLOG_GROUP_DRV: &str = "DRV";
pub const RTLOG_GROUP_HOST: &str = "HOST";
pub const RTLOG_GROUP_GUEST: &str = "GUEST";
pub const RTLOG_GROUP_SERV: &str = "SERV";
pub const RTLOG_GROUP_TEST: &str = "TEST";

pub fn rt_log_init(level: LevelFilter) -> vbox_core::VBoxResult<()> {
    ::log::set_max_level(level);
    *LOG_LEVEL.lock() = level;
    Ok(())
}

pub fn rt_log_print(level: RTLogLevel, group: &str, message: &str) {
    let level_filter = *LOG_LEVEL.lock();
    let should_log = match level {
        RTLogLevel::Debug => level_filter <= LevelFilter::Debug,
        RTLogLevel::Info => level_filter <= LevelFilter::Info,
        RTLogLevel::Warning => level_filter <= LevelFilter::Warn,
        RTLogLevel::Error => level_filter <= LevelFilter::Error,
        RTLogLevel::Fatal => level_filter <= LevelFilter::Error,
    };
    if !should_log {
        return;
    }
    let log_msg = format!("[{}] {}", group, message);
    match level {
        RTLogLevel::Debug => web_sys::console::log_2(&log_msg.into(), &"DEBUG".into()),
        RTLogLevel::Info => web_sys::console::log_2(&log_msg.into(), &"INFO".into()),
        RTLogLevel::Warning => web_sys::console::log_2(&log_msg.into(), &"WARN".into()),
        RTLogLevel::Error => web_sys::console::error_2(&log_msg.into(), &"ERROR".into()),
        RTLogLevel::Fatal => {
            web_sys::console::error_2(&log_msg.into(), &"FATAL".into());
        }
    }
}

pub fn rt_log_set_level(level: LevelFilter) {
    *LOG_LEVEL.lock() = level;
    set_max_level(level);
}

pub fn rt_log_set_group(group: &str) {
    *LOG_GROUP.lock() = Some(group.to_string());
}

pub fn rt_log_get_level() -> LevelFilter {
    *LOG_LEVEL.lock()
}

pub fn rt_log_debug(group: &str, message: &str) {
    rt_log_print(RTLogLevel::Debug, group, message);
}

pub fn rt_log_info(group: &str, message: &str) {
    rt_log_print(RTLogLevel::Info, group, message);
}

pub fn rt_log_warning(group: &str, message: &str) {
    rt_log_print(RTLogLevel::Warning, group, message);
}

pub fn rt_log_error(group: &str, message: &str) {
    rt_log_print(RTLogLevel::Error, group, message);
}

pub fn rt_log_fatal(group: &str, message: &str) {
    rt_log_print(RTLogLevel::Fatal, group, message);
}

pub fn is_log_initialized() -> bool {
    true
}

pub struct RTLog {
    group: String,
    flags: u32,
}

impl RTLog {
    pub fn new(group: &str) -> Self {
        Self {
            group: group.to_string(),
            flags: RTLOG_DEFAULT_FLAGS,
        }
    }

    pub fn init(&self, level: LevelFilter) -> vbox_core::VBoxResult<()> {
        rt_log_init(level)?;
        rt_log_set_group(&self.group);
        Ok(())
    }

    pub fn debug(&self, message: &str) {
        rt_log_debug(&self.group, message);
    }

    pub fn info(&self, message: &str) {
        rt_log_info(&self.group, message);
    }

    pub fn warning(&self, message: &str) {
        rt_log_warning(&self.group, message);
    }

    pub fn error(&self, message: &str) {
        rt_log_error(&self.group, message);
    }

    pub fn fatal(&self, message: &str) {
        rt_log_fatal(&self.group, message);
    }

    pub fn set_level(&self, level: LevelFilter) {
        rt_log_set_level(level);
    }
}

impl Default for RTLog {
    fn default() -> Self {
        Self::new(RTLOG_GROUP_MAIN)
    }
}