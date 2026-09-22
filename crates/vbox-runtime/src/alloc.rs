#[derive(Debug)]
pub struct HeapInfo {
    pub total_allocated: usize,
    pub peak_allocation: usize,
}

static mut HEAP_INFO: HeapInfo = HeapInfo {
    total_allocated: 0,
    peak_allocation: 0,
};

pub fn alloc(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 1).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) };
    if !ptr.is_null() {
        unsafe {
            HEAP_INFO.total_allocated += size;
            if HEAP_INFO.total_allocated > HEAP_INFO.peak_allocation {
                HEAP_INFO.peak_allocation = HEAP_INFO.total_allocated;
            }
        }
    }
    ptr
}

pub fn free(ptr: *mut u8, size: usize) {
    if !ptr.is_null() {
        let layout = std::alloc::Layout::from_size_align(size, 1).unwrap();
        unsafe { std::alloc::dealloc(ptr, layout) };
        unsafe {
            HEAP_INFO.total_allocated = HEAP_INFO.total_allocated.saturating_sub(size);
        }
    }
}

pub fn alloc_zeroed(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 1).unwrap();
    let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
    if !ptr.is_null() {
        unsafe {
            HEAP_INFO.total_allocated += size;
            if HEAP_INFO.total_allocated > HEAP_INFO.peak_allocation {
                HEAP_INFO.peak_allocation = HEAP_INFO.total_allocated;
            }
        }
    }
    ptr
}

pub fn page_size() -> usize {
    4096
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc() {
        let ptr = alloc(1024);
        assert!(!ptr.is_null());
        unsafe { std::ptr::write_bytes(ptr, 0xAB, 1024); }
        free(ptr, 1024);
    }

    #[test]
    fn test_alloc_zeroed() {
        let ptr = alloc_zeroed(256);
        assert!(!ptr.is_null());
        unsafe {
            let bytes = std::slice::from_raw_parts(ptr, 256);
            assert_eq!(bytes, &[0u8; 256]);
        }
        free(ptr, 256);
    }

    #[test]
    fn test_page_size() {
        assert_eq!(page_size(), 4096);
    }
}
