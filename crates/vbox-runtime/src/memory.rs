#![allow(non_camel_case_types)]

use std::alloc::{Layout, System};
use std::ptr::{self, NonNull};
use parking_lot::Mutex;
use vbox_core::{VBoxResult, VBoxError, PAGE_SIZE};

pub const RTMEMHEAP_MAGIC: u32 = 0x52544D48;

#[repr(C)]
pub struct RTMEMHEAP {
    pub u32Magic: u32,
    pub cbHeap: usize,
    pub cbUsed: usize,
    pub cbReserved: usize,
}

pub struct RTMemHeapInstance {
    base: usize,
    size: usize,
    used: usize,
    blocks: Mutex<Vec<HeapEntry>>,
}

struct HeapEntry {
    addr: usize,
    size: usize,
    zeroed: bool,
}

impl RTMemHeapInstance {
    pub fn create(size: usize) -> VBoxResult<NonNull<RTMEMHEAP>> {
        let aligned = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let layout = Layout::from_size_align(aligned, 1)
            .map_err(|_| VBoxError::InvalidParam)?;
        let ptr = unsafe { System.alloc(layout) };
        if ptr.is_null() {
            return Err(VBoxError::NoMemory);
        }
        let heap = Box::new(RTMEMHEAP {
            u32Magic: RTMEMHEAP_MAGIC,
            cbHeap: aligned,
            cbUsed: 0,
            cbReserved: 0,
        });
        let heap_ptr = Box::into_raw(heap);
        Ok(NonNull::new(heap_ptr as *mut _).unwrap())
    }

    pub fn destroy(heap: NonNull<RTMEMHEAP>) -> VBoxResult<()> {
        unsafe {
            let layout = Layout::from_size_align((*heap.as_ptr()).cbHeap, 1)
                .map_err(|_| VBoxError::InvalidParam)?;
            System.dealloc(heap.as_ptr() as *mut u8, layout);
        }
        Ok(())
    }
}

pub struct RTMemHeapHandle {
    heap: NonNull<RTMEMHEAP>,
    id: u32,
}

impl RTMemHeapHandle {
    pub fn new(size: usize) -> VBoxResult<Self> {
        let heap = RTMemHeapInstance::create(size)?;
        Ok(Self {
            heap,
            id: 0,
        })
    }

    pub fn alloc(&mut self, size: usize) -> VBoxResult<*mut u8> {
        let aligned = (size + 8 - 1) & !(8 - 1);
        let layout = Layout::from_size_align(aligned, 1)
            .map_err(|_| VBoxError::InvalidParam)?;
        let ptr = unsafe { System.alloc(layout) };
        if ptr.is_null() {
            return Err(VBoxError::NoMemory);
        }
        unsafe { ptr::write_bytes(ptr, 0, aligned); }
        Ok(ptr)
    }

    pub fn alloc_z(&mut self, size: usize) -> VBoxResult<*mut u8> {
        let aligned = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let layout = Layout::from_size_align(aligned, PAGE_SIZE)
            .map_err(|_| VBoxError::InvalidParam)?;
        let ptr = unsafe { System.alloc(layout) };
        if ptr.is_null() {
            return Err(VBoxError::NoMemory);
        }
        unsafe { ptr::write_bytes(ptr, 0, aligned); }
        Ok(ptr)
    }

    pub fn free(&mut self, ptr: *mut u8, size: usize) {
        let aligned = (size + 8 - 1) & !(8 - 1);
        let layout = Layout::from_size_align(aligned, 1)
            .unwrap_or(Layout::from_size_align(aligned, 1).unwrap());
        unsafe { System.dealloc(ptr, layout); }
    }

    pub fn size(&self) -> usize {
        unsafe { (*self.heap.as_ptr()).cbHeap }
    }

    pub fn used(&self) -> usize {
        unsafe { (*self.heap.as_ptr()).cbUsed }
    }
}

impl Drop for RTMemHeapHandle {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align(self.size(), 1)
                .unwrap_or(Layout::from_size_align(self.size(), 1).unwrap());
            System.dealloc(self.heap.as_ptr() as *mut u8, layout);
        }
    }
}

pub fn rt_mem_heap_create(size: usize) -> VBoxResult<RTMemHeapHandle> {
    RTMemHeapHandle::new(size)
}

pub fn rt_mem_heap_destroy(heap: RTMemHeapHandle) -> VBoxResult<()> {
    drop(heap);
    Ok(())
}

pub fn rt_mem_heap_alloc(heap: &mut RTMemHeapHandle, size: usize) -> VBoxResult<*mut u8> {
    heap.alloc(size)
}

pub fn rt_mem_heap_free(heap: &mut RTMemHeapHandle, ptr: *mut u8, size: usize) {
    heap.free(ptr, size);
}

pub fn rt_mem_heap_alloc_z(heap: &mut RTMemHeapHandle, size: usize) -> VBoxResult<*mut u8> {
    heap.alloc_z(size)
}

pub fn rt_mem_heap_size(heap: &RTMemHeapHandle) -> usize {
    heap.size()
}

pub fn rt_mem_copy(dst: *mut u8, src: *const u8, cb: usize) {
    unsafe { ptr::copy_nonoverlapping(src, dst, cb); }
}

pub fn rt_mem_fill(ptr: *mut u8, value: u8, cb: usize) {
    unsafe { ptr::write_bytes(ptr, value, cb); }
}

pub fn rt_mem_zero(ptr: *mut u8, cb: usize) {
    unsafe { ptr::write_bytes(ptr, 0, cb); }
}

pub fn rt_mem_cmp(a: *const u8, b: *const u8, cb: usize) -> i32 {
    unsafe {
        let sa = core::slice::from_raw_parts(a, cb);
        let sb = core::slice::from_raw_parts(b, cb);
        sa.cmp(sb) as i32
    }
}

pub fn rt_mem_move(dst: *mut u8, src: *const u8, cb: usize) {
    unsafe { ptr::copy(src, dst, cb); }
}

pub fn rt_mem_alloc_page(count: usize) -> VBoxResult<*mut u8> {
    let size = count * PAGE_SIZE;
    let layout = Layout::from_size_align(size, PAGE_SIZE)
        .map_err(|_| VBoxError::InvalidParam)?;
    let ptr = unsafe { System.alloc(layout) };
    if ptr.is_null() {
        return Err(VBoxError::NoMemory);
    }
    unsafe { ptr::write_bytes(ptr, 0, size); }
    Ok(ptr)
}

pub unsafe fn rt_mem_free_page(ptr: *mut u8, count: usize) {
    let size = count * PAGE_SIZE;
    let layout = Layout::from_size_align(size, PAGE_SIZE)
        .unwrap_or(Layout::from_size_align(size, PAGE_SIZE).unwrap());
    System.dealloc(ptr, layout);
}

pub fn rt_mem_alloc32(size: usize) -> VBoxResult<*mut u8> {
    let aligned = (size + 4 - 1) & !(4 - 1);
    let layout = Layout::from_size_align(aligned, 1)
        .map_err(|_| VBoxError::InvalidParam)?;
    let ptr = unsafe { System.alloc(layout) };
    if ptr.is_null() {
        return Err(VBoxError::NoMemory);
    }
    Ok(ptr)
}

pub fn rt_mem_alloc64(size: usize) -> VBoxResult<*mut u8> {
    let aligned = (size + 8 - 1) & !(8 - 1);
    let layout = Layout::from_size_align(aligned, 1)
        .map_err(|_| VBoxError::InvalidParam)?;
    let ptr = unsafe { System.alloc(layout) };
    if ptr.is_null() {
        return Err(VBoxError::NoMemory);
    }
    Ok(ptr)
}

pub struct RTMemTracker {
    allocations: Mutex<Vec<MemTrack>>,
}

struct MemTrack {
    ptr: *mut u8,
    size: usize,
    line: u32,
    file: &'static str,
}

impl RTMemTracker {
    pub fn new() -> Self {
        Self {
            allocations: Mutex::new(Vec::new()),
        }
    }

    pub fn track(&self, ptr: *mut u8, size: usize, file: &'static str, line: u32) {
        self.allocations.lock().push(MemTrack { ptr, size, file, line });
    }

    pub fn untrack(&self, ptr: *mut u8) {
        self.allocations.lock().retain(|t| t.ptr != ptr);
    }

    pub fn leak_count(&self) -> usize {
        self.allocations.lock().len()
    }
}

impl Default for RTMemTracker {
    fn default() -> Self {
        Self::new()
    }
}
