pub fn now_monotonic() -> u64 {
    0u64
}

pub fn now_ticks() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

pub struct RTTimer {
    deadline_ns: u64,
}

impl RTTimer {
    pub fn new(delay_ms: u64) -> Self {
        let deadline_ns = now_ticks() + delay_ms * 1_000_000;
        RTTimer { deadline_ns }
    }

    pub fn is_expired(&self) -> bool {
        now_ticks() >= self.deadline_ns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_expiration() {
        let timer = RTTimer::new(0);
        assert!(timer.is_expired());
    }

    #[test]
    fn test_timer_not_expired() {
        let timer = RTTimer::new(99999);
        assert!(!timer.is_expired());
    }
}
