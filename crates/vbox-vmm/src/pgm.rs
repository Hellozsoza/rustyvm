use vbox_core::{PAGE_SIZE, VBoxError, VBoxResult};
use serde::{Serialize, Deserialize};
use parking_lot::Mutex;
use std::sync::Arc;
use log::info;

pub const GUEST_RAM_SIZE: usize = 2 * 1024 * 1024 * 1024;
pub const PAGE_TABLE_LEVELS: usize = 4;
pub const PAGE_TABLE_ENTRIES: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PGMPHYSPAGE {
    pub base: u64,
    pub size: u64,
    pub flags: u32,
    pub pData: Box<[u8]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PGMRAMRANGE {
    pub Base: u64,
    pub cb: u64,
    pub pPages: Vec<PGMPHYSPAGE>,
}

#[derive(Debug, Clone)]
pub struct MmioRegion {
    pub base: u64,
    pub size: u64,
    pub handler: fn(u64, u32) -> u32,
}

#[derive(Debug)]
pub struct GuestMemory {
    pub ram: Box<[u8]>,
    pub page_table: Vec<u64>,
    pub mmio_regions: Vec<MmioRegion>,
}

impl GuestMemory {
    pub fn new() -> Result<Self, VBoxError> {
        info!("PGM: Allocating {} bytes guest RAM", GUEST_RAM_SIZE);
        let ram = vec![0u8; GUEST_RAM_SIZE].into_boxed_slice();
        let page_table = vec![0u64; PAGE_TABLE_ENTRIES * PAGE_TABLE_LEVELS];
        Ok(GuestMemory {
            ram,
            page_table,
            mmio_regions: Vec::new(),
        })
    }

    pub fn read(&self, gpa: u64, size: usize) -> Result<Vec<u8>, VBoxError> {
        if gpa as usize + size > GUEST_RAM_SIZE {
            return Err(VBoxError::Fail);
        }
        let mut result = Vec::with_capacity(size);
        for i in 0..size {
            result.push(self.ram[(gpa as usize + i)]);
        }
        Ok(result)
    }

    pub fn write(&mut self, gpa: u64, data: &[u8]) -> Result<(), VBoxError> {
        if gpa as usize + data.len() > GUEST_RAM_SIZE {
            return Err(VBoxError::Fail);
        }
        for (i, byte) in data.iter().enumerate() {
            self.ram[(gpa as usize + i)] = *byte;
        }
        Ok(())
    }

    pub fn pgm_translate(&self, gva: u64) -> Result<u64, VBoxError> {
        let pml4_index = ((gva >> 39) & 0x1FF) as usize;
        let pdpt_index = ((gva >> 30) & 0x1FF) as usize;
        let pd_index = ((gva >> 21) & 0x1FF) as usize;
        let pt_index = ((gva >> 12) & 0x1FF) as usize;
        let offset = gva & 0xFFF;

        let pml4_entry = self.page_table[pml4_index];
        if pml4_entry & 1 == 0 {
            return Err(VBoxError::Fail);
        }
        let pdpt_base = (pml4_entry & !0xFFF) as usize;
        let pdpt_entry = self.page_table[pdpt_base + pdpt_index];
        if pdpt_entry & 1 == 0 {
            return Err(VBoxError::Fail);
        }
        let pd_base = (pdpt_entry & !0xFFF) as usize;
        let pd_entry = self.page_table[pd_base + pd_index];
        if pd_entry & 1 == 0 {
            return Err(VBoxError::Fail);
        }
        let pt_base = (pd_entry & !0xFFF) as usize;
        let pt_entry = self.page_table[pt_base + pt_index];
        if pt_entry & 1 == 0 {
            return Err(VBoxError::Fail);
        }
        let frame_base = pt_entry & !0xFFF;
        Ok(frame_base | offset)
    }

    pub fn pgm_map_mmio(&mut self, region: MmioRegion) {
        self.mmio_regions.push(region);
    }

    pub fn pgm_set_pf_handler(&mut self) {
        info!("PGM: Page fault handler registered");
    }
}

#[derive(Clone, Debug)]
pub struct PgmState {
    pub guest_memory: Arc<Mutex<GuestMemory>>,
}

pub fn pgm_init() -> Result<PgmState, VBoxError> {
    let mem = GuestMemory::new()?;
    let state = PgmState {
        guest_memory: Arc::new(Mutex::new(mem)),
    };
    info!("PGM: Physical memory manager initialized");
    Ok(state)
}

pub fn PGMR3PhysInit() -> Result<PgmState, VBoxError> {
    pgm_init()
}

pub fn PGMR3HeapCreate() -> Result<(), VBoxError> {
    info!("PGM: Heap created for dynamic allocations");
    Ok(())
}

pub trait PdmDeviceTrait: Send {
    fn attach(&mut self, config: &str) -> Result<(), VBoxError>;
    fn detach(&mut self) -> Result<(), VBoxError>;
    fn reset(&mut self) -> Result<(), VBoxError>;
    fn power_off(&mut self) -> Result<(), VBoxError>;
    fn query_interface(&self, iid: &str) -> Option<String>;
}
