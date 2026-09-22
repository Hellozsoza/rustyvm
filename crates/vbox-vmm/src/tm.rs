use vbox_core::VBoxError;
use vbox_core::VBoxResult;
// serde not needed for non-serializable types
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Instant, Duration};
use log::info;
use crate::cpu::CpuState;

#[derive(Debug, Clone)]
pub struct TMTimer {
    pub deadline: Instant,
    pub period: Duration,
    pub callback: fn(),
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct TMState {
    pub timers: Vec<TMTimer>,
    pub tsc_value: u64,
    pub start_time: Instant,
}

impl TMState {
    pub fn new() -> Self {
        TMState {
            timers: Vec::new(),
            tsc_value: 0,
            start_time: Instant::now(),
        }
    }
}

pub fn TMR3Init() -> Result<TMState, VBoxError> {
    let state = TMState::new();
    info!("TM: Timer Manager initialized");
    Ok(state)
}

pub fn TMR3TimerCreate(state: &mut TMState) -> Result<TMTimer, VBoxError> {
    let timer = TMTimer {
        deadline: Instant::now(),
        period: Duration::from_millis(1),
        callback: || {},
        active: false,
    };
    state.timers.push(timer);
    info!("TM: Timer created");
    Ok(state.timers.last().unwrap().clone())
}

pub fn TMR3TimerSet(timer: &mut TMTimer, deadline: Instant) {
    timer.deadline = deadline;
    timer.active = true;
}

pub fn TMR3TimerDoUpdate(state: &mut TMState) -> Result<Duration, VBoxError> {
    let now = Instant::now();
    let elapsed = now.duration_since(state.start_time);
    for timer in state.timers.iter_mut() {
        if timer.active && now >= timer.deadline {
            (timer.callback)();
            timer.deadline = now + timer.period;
        }
    }
    state.tsc_value = elapsed.as_nanos() as u64;
    Ok(elapsed)
}

pub fn tm_get_tsc(state: &TMState) -> u64 {
    state.tsc_value
}

pub fn tm_get_tsc_latest() -> u64 {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    now.as_nanos() as u64
}
