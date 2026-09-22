#[cfg(target_arch = "wasm32")]
use console_log;

pub fn init(level: log::LevelFilter) -> Result<(), log::SetLoggerError> {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(level)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = level;
        Ok(())
    }
}

pub fn info(msg: &str) {
    log::info!("{}", msg);
}

pub fn warn(msg: &str) {
    log::warn!("{}", msg);
}

pub fn error(msg: &str) {
    log::error!("{}", msg);
}
