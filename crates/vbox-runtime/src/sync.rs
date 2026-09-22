use parking_lot::Mutex;

pub struct CritSect {
    lock: Mutex<bool>,
}

impl CritSect {
    pub fn new() -> Self {
        CritSect { lock: Mutex::new(false) }
    }

    pub fn lock(&self) -> parking_lot::MutexGuard<bool> {
        self.lock.lock()
    }
}

pub struct Semaphore {
    count: parking_lot::Condvar,
    value: Mutex<usize>,
}

impl Semaphore {
    pub fn new(count: usize) -> Self {
        Semaphore {
            count: parking_lot::Condvar::new(),
            value: Mutex::new(count),
        }
    }

    pub fn wait(&self) {
        let mut val = self.value.lock();
        while *val == 0 {
            self.count.wait(&mut val);
        }
        *val -= 1;
    }

    pub fn signal(&self) {
        let mut val = self.value.lock();
        *val += 1;
        self.count.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crit_sect() {
        let cs = CritSect::new();
        let _guard = cs.lock();
    }

    #[test]
    fn test_semaphore() {
        let sem = Semaphore::new(1);
        sem.wait();
        sem.signal();
    }
}
