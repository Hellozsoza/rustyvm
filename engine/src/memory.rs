//! Sparse flat guest memory. Guest addresses are u64 even on a wasm32 host.
use std::collections::BTreeMap;

pub const PAGE_SIZE: u64 = 4096;
pub const MAX_GUEST_BYTES: u64 = 8 * 1024 * 1024 * 1024;
// Bound actual host use separately from the sparse guest address space.
const MAX_PAGES: usize = 32768; // 128 MiB committed, plus interpreter overhead.

pub struct Memory {
    pub size: u64,
    pages: BTreeMap<u64, Box<[u8; PAGE_SIZE as usize]>>,
}

impl Memory {
    pub fn new(size: u64) -> Result<Self, String> {
        if !(16 * 1024 * 1024..=MAX_GUEST_BYTES).contains(&size) {
            return Err("Guest address space must be between 16 MiB and 8 GiB".into());
        }
        Ok(Self {
            size,
            pages: BTreeMap::new(),
        })
    }

    pub fn check(&self, address: u64, length: u64) -> Result<(), String> {
        if address
            .checked_add(length)
            .is_none_or(|end| end > self.size)
        {
            return Err(format!(
                "Guest memory access out of bounds at {address:#x} ({length} bytes)"
            ));
        }
        Ok(())
    }

    pub fn byte(&self, address: u64) -> Result<u8, String> {
        self.check(address, 1)?;
        Ok(self
            .pages
            .get(&(address / PAGE_SIZE))
            .map_or(0, |page| page[(address % PAGE_SIZE) as usize]))
    }

    pub fn read(&self, address: u64, bytes: usize) -> Result<u64, String> {
        if !(1..=8).contains(&bytes) {
            return Err("Integer memory reads must contain 1–8 bytes".into());
        }
        self.check(address, bytes as u64)?;
        let mut result = 0;
        for offset in 0..bytes {
            result |= u64::from(self.byte(address + offset as u64)?) << (offset * 8);
        }
        Ok(result)
    }

    pub fn write(&mut self, address: u64, bytes: &[u8]) -> Result<(), String> {
        self.check(address, bytes.len() as u64)?;
        if bytes.is_empty() {
            return Ok(());
        }
        let range = address / PAGE_SIZE..=(address + bytes.len() as u64 - 1) / PAGE_SIZE;
        let missing = range
            .clone()
            .filter(|page| !self.pages.contains_key(page))
            .count();
        if self.pages.len() + missing > MAX_PAGES {
            return Err("Guest exceeded the 128 MiB committed-memory budget".into());
        }
        for page in range {
            self.pages
                .entry(page)
                .or_insert_with(|| Box::new([0; PAGE_SIZE as usize]));
        }
        for (offset, value) in bytes.iter().enumerate() {
            let at = address + offset as u64;
            self.pages.get_mut(&(at / PAGE_SIZE)).unwrap()[(at % PAGE_SIZE) as usize] = *value;
        }
        Ok(())
    }

    pub fn committed_bytes(&self) -> u64 {
        self.pages.len() as u64 * PAGE_SIZE
    }
}
