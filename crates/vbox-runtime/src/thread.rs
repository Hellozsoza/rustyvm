use std::marker::PhantomData;

pub struct RTThread<T: Send + 'static> {
    handle: Option<std::thread::JoinHandle<T>>,
    _phantom: PhantomData<T>,
}

impl<T: Send + 'static> RTThread<T> {
    pub fn spawn<F: FnOnce() -> T + Send + 'static>(f: F) -> RTThread<T> {
        RTThread {
            handle: Some(std::thread::spawn(f)),
            _phantom: PhantomData,
        }
    }

    pub fn join(mut self) -> Result<T, ()> {
        self.handle.take().ok_or(()).map_err(|_| ()).and_then(|h| {
            h.join().map_err(|_| ())
        })
    }

    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().map(|h| h.is_finished()).unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_spawn() {
        let t = RTThread::spawn(|| 42);
        let result = t.join().unwrap();
        assert_eq!(result, 42);
    }
}
