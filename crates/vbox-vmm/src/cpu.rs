use vbox_core::{VBoxError};
use serde::{Serialize, Deserialize};
use parking_lot::Mutex;
use std::sync::OnceLock;
use log::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuMode {
    Real,
    Protected,
    LongMode,
    Legacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuState {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
    pub cs: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
    pub ss: u64,
    pub efer: u64,
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub dr0: u64,
    pub dr1: u64,
    pub dr2: u64,
    pub dr3: u64,
    pub dr6: u64,
    pub dr7: u64,
    pub mode: CpuMode,
}

impl Default for CpuState {
    fn default() -> Self {
        CpuState {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, rflags: 0,
            cs: 0, ds: 0, es: 0, fs: 0, gs: 0, ss: 0,
            efer: 0, cr0: 0, cr2: 0, cr3: 0, cr4: 0,
            dr0: 0, dr1: 0, dr2: 0, dr3: 0, dr6: 0, dr7: 0,
            mode: CpuMode::Protected,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CpuIdEntry {
    pub vendor: &'static str,
    pub brand: &'static str,
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub ext_family: u8,
    pub ext_model: u8,
    pub cpu_count: u8,
    pub features_edx: u32,
    pub features_ecx: u32,
    pub ext_features_edx: u32,
    pub ext_features_ecx: u32,
}

pub static CPUID_DATABASE: &[CpuIdEntry] = &[
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Core i7-4790", family: 6, model: 60, stepping: 3, ext_family: 0, ext_model: 3, cpu_count: 4, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Core i7-6700K", family: 6, model: 94, stepping: 3, ext_family: 0, ext_model: 3, cpu_count: 4, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Core i9-9900K", family: 6, model: 158, stepping: 10, ext_family: 0, ext_model: 9, cpu_count: 8, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Core i5-12400", family: 6, model: 151, stepping: 2, ext_family: 0, ext_model: 0, cpu_count: 6, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Xeon E5-2690", family: 6, model: 62, stepping: 4, ext_family: 0, ext_model: 4, cpu_count: 8, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD Ryzen 7 5800X", family: 25, model: 80, stepping: 0, ext_family: 0, ext_model: 0, cpu_count: 8, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD Ryzen 9 5900X", family: 25, model: 101, stepping: 0, ext_family: 0, ext_model: 0, cpu_count: 12, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD Ryzen 5 3600", family: 23, model: 96, stepping: 0, ext_family: 0, ext_model: 0, cpu_count: 6, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD Ryzen 7 2700X", family: 23, model: 48, stepping: 1, ext_family: 0, ext_model: 0, cpu_count: 8, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD Ryzen 9 7950X", family: 25, model: 176, stepping: 1, ext_family: 0, ext_model: 0, cpu_count: 16, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Core i3-10100", family: 6, model: 122, stepping: 2, ext_family: 0, ext_model: 0, cpu_count: 4, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Core i7-3770", family: 6, model: 58, stepping: 9, ext_family: 0, ext_model: 5, cpu_count: 4, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Core i5-4690", family: 6, model: 61, stepping: 2, ext_family: 0, ext_model: 2, cpu_count: 4, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Xeon E3-1270", family: 6, model: 60, stepping: 3, ext_family: 0, ext_model: 3, cpu_count: 4, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Pentium G4560", family: 6, model: 94, stepping: 3, ext_family: 0, ext_model: 3, cpu_count: 2, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD FX-8350", family: 21, model: 1, stepping: 1, ext_family: 0, ext_model: 0, cpu_count: 8, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD Ryzen 7 1700", family: 23, model: 31, stepping: 1, ext_family: 0, ext_model: 0, cpu_count: 8, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD Ryzen 3 3100", family: 23, model: 96, stepping: 0, ext_family: 0, ext_model: 0, cpu_count: 4, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Core i9-13900K", family: 6, model: 183, stepping: 1, ext_family: 0, ext_model: 13, cpu_count: 24, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Core Ultra 7 265K", family: 6, model: 202, stepping: 2, ext_family: 0, ext_model: 2, cpu_count: 20, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD Ryzen 7 7700X", family: 25, model: 96, stepping: 1, ext_family: 0, ext_model: 0, cpu_count: 8, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD Ryzen 9 7900X", family: 25, model: 171, stepping: 1, ext_family: 0, ext_model: 0, cpu_count: 12, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Celeron G5905", family: 6, model: 151, stepping: 2, ext_family: 0, ext_model: 0, cpu_count: 2, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "GenuineIntel", brand: "Intel Atom N5100", family: 6, model: 138, stepping: 2, ext_family: 0, ext_model: 0, cpu_count: 4, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
    CpuIdEntry { vendor: "AuthenticAMD", brand: "AMD EPYC 7763", family: 25, model: 1, stepping: 1, ext_family: 0, ext_model: 0, cpu_count: 64, features_edx: 0x0F81078F, features_ecx: 0x00000007, ext_features_edx: 0x00000001, ext_features_ecx: 0x00000001 },
];

static CPU_STATES: OnceLock<Mutex<Vec<CpuState>>> = OnceLock::new();

pub fn cpum_get_id(cpu_id: u32, leaf: u32) -> (u32, u32, u32, u32) {
    let idx = (cpu_id % CPUID_DATABASE.len() as u32) as usize;
    let entry = &CPUID_DATABASE[idx];
    match leaf {
        0 => (entry.features_edx, entry.features_ecx, 0, 0),
        1 => (entry.ext_features_edx, entry.ext_features_ecx, 0, 0),
        _ => (0, 0, 0, 0),
    }
}

pub fn cpum_get_guest_context(cpu_index: usize) -> *mut CpuState {
    let states = CPU_STATES.get_or_init(|| Mutex::new(vec![CpuState::default(); 256]));
    let mut states = states.lock();
    if cpu_index >= states.len() {
        states.resize(cpu_index + 1, CpuState::default());
    }
    &mut states[cpu_index] as *mut CpuState
}

pub fn cpum_set_guest_context(cpu_index: usize, state: &CpuState) {
    let states = CPU_STATES.get_or_init(|| Mutex::new(vec![CpuState::default(); 256]));
    let mut states = states.lock();
    if cpu_index >= states.len() {
        states.resize(cpu_index + 1, CpuState::default());
    }
    states[cpu_index] = state.clone();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUMCTX {
    pub guest: CpuState,
}

impl CPUMCTX {
    pub fn new() -> Self {
        CPUMCTX { guest: CpuState::default() }
    }
}

pub fn CPUMR3Init() -> Result<(), VBoxError> {
    let _ctx = CPUMCTX::new();
    info!("CPUMR3Init: CPU state initialized");
    Ok(())
}
