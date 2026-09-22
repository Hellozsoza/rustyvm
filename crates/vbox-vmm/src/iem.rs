use vbox_core::{VBoxError, VBoxResult};
use serde::{Serialize, Deserialize};
use parking_lot::Mutex;
use std::sync::Arc;
use log::info;
use crate::cpu::CpuState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IemRunMode {
    Normal,
    SingleStep,
    Halt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IemState {
    pub instruction_count: u64,
    pub interrupt_state: bool,
    pub halt_state: bool,
    pub nested_level: u32,
    pub idt_irq: u8,
    pub run_mode: IemRunMode,
}

impl Default for IemState {
    fn default() -> Self {
        IemState {
            instruction_count: 0,
            interrupt_state: false,
            halt_state: false,
            nested_level: 0,
            idt_irq: 0,
            run_mode: IemRunMode::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IEMInstruction {
    MOV,
    ADD,
    SUB,
    JMP,
    CALL,
    RET,
    PUSH,
    POP,
    CMP,
    TEST,
    LEA,
    XOR,
    OR,
    AND,
    NOT,
    SHL,
    SHR,
    SAR,
    INC,
    DEC,
    NOP,
    HLT,
    INT,
    IRET,
    PUSHF,
    POPF,
    LAHF,
    SAHF,
    XCHG,
    MOVSX,
    MOVZX,
    CMOV,
    SETCC,
    JCC,
    LOOP,
    LOOPZ,
    LOOPNZ,
    REPE,
    REPNE,
    MUL,
    IMUL,
    DIV,
    IDIV,
    CBW,
    CWD,
    CDQ,
    CQO,
    CLC,
    STC,
    CMC,
    CLD,
    STD,
    CLI,
    STI,
    WAIT,
    FENCE,
    LFENCE,
    MFENCE,
    SFENCE,
    PAUSE,
    UD2,
    RDTSC,
    RDTSCP,
    CPUID,
    VMCALL,
    IN,
    OUT,
    INS,
    OUTS,
    LODS,
    STOS,
    MOVS,
    CMPS,
    SCAS,
    ENTER,
    LEAVE,
    RETF,
    IRETQ,
    SHL_IMM,
    SHR_IMM,
    SAR_IMM,
    NEG,
    ADC,
    SBB,
    XADD,
    CMPXCHG,
    BSWAP,
    POPCNT,
    LZCNT,
    TZCNT,
    MOVBE,
    AESENC,
    AESDEC,
    PCLMULQDQ,
}

#[derive(Debug, Clone)]
pub struct IEM_CPUSTATE {
    pub regs: CpuState,
    pub mode: IemRunMode,
    pub interrupts_disabled: bool,
    pub instruction_count: u64,
}

impl IEM_CPUSTATE {
    pub fn new() -> Self {
        IEM_CPUSTATE {
            regs: CpuState::default(),
            mode: IemRunMode::Normal,
            interrupts_disabled: false,
            instruction_count: 0,
        }
    }
}

#[inline(always)]
pub fn iem_decode(opcode: u8) -> IEMInstruction {
    match opcode {
        0x88 | 0x89 | 0x8A | 0x8B | 0xC6 | 0xC7 => IEMInstruction::MOV,
        0x01 | 0x03 | 0x00 | 0x02 | 0x29 | 0x2B => IEMInstruction::ADD,
        0x2B | 0x29 | 0x48 => IEMInstruction::SUB,
        0xE9 | 0xEB | 0xEA | 0xEF => IEMInstruction::JMP,
        0xE8 | 0xFF => IEMInstruction::CALL,
        0xC3 | 0xCB | 0xCA | 0xCF => IEMInstruction::RET,
        0x50..=0x57 | 0x6A | 0x68 => IEMInstruction::PUSH,
        0x58..=0x5F | 0x8F => IEMInstruction::POP,
        0x39 | 0x3B | 0x81 | 0x83 | 0x80 | 0x38 | 0x3A | 0x3B => IEMInstruction::CMP,
        0x84 | 0x85 | 0xA8 | 0xA9 | 0xF6 | 0xF7 | 0x21 | 0x23 | 0x25 => IEMInstruction::TEST,
        0x8D => IEMInstruction::LEA,
        0x31 | 0x33 | 0x30 | 0x32 | 0xAF => IEMInstruction::XOR,
        0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x0F => IEMInstruction::OR,
        0x20 | 0x21 | 0x22 | 0x23 | 0x24 | 0x25 | 0x3C | 0x3D => IEMInstruction::AND,
        0xF7 => IEMInstruction::NOT,
        0xC0 | 0xC1 | 0xD0 | 0xD1 | 0xD2 | 0xD3 => IEMInstruction::SHL_IMM,
        0xC0 | 0xC1 | 0xD0 | 0xD1 => IEMInstruction::SHR_IMM,
        0x0F | 0x31 => IEMInstruction::RDTSC,
        0x0F | 0x05 => IEMInstruction::VMCALL,
        0x0F | 0xA2 => IEMInstruction::CPUID,
        0x90 => IEMInstruction::NOP,
        0xF4 => IEMInstruction::HLT,
        0xCC | 0xCD => IEMInstruction::INT,
        0x9C => IEMInstruction::PUSHF,
        0x9D => IEMInstruction::POPF,
        0x9E => IEMInstruction::LAHF,
        0x9F => IEMInstruction::SAHF,
        0x87 | 0x90..=0x97 => IEMInstruction::XCHG,
        0x63 => IEMInstruction::MOVSX,
        0x0F | 0xB6 | 0xB7 => IEMInstruction::MOVZX,
        0x0F | 0x40..=0x4F => IEMInstruction::CMOV,
        0x0F | 0x90..=0x9F => IEMInstruction::SETCC,
        0x70..=0x7F => IEMInstruction::JCC,
        0xE0..=0xE3 => IEMInstruction::LOOP,
        0xFE => IEMInstruction::INC,
        0x48..=0x4F => IEMInstruction::DEC,
        0xC8 => IEMInstruction::ENTER,
        0xC9 => IEMInstruction::LEAVE,
        0xCB | 0xCA => IEMInstruction::RETF,
        0xF7 | 0xF6 => IEMInstruction::DIV,
        0xF7 | 0xEB => IEMInstruction::IDIV,
        0x98 => IEMInstruction::CBW,
        0x99 => IEMInstruction::CWD,
        0xF8 => IEMInstruction::CLC,
        0xF9 => IEMInstruction::STC,
        0xFA => IEMInstruction::CLI,
        0xFB => IEMInstruction::STI,
        0xFC => IEMInstruction::CLD,
        0xFD => IEMInstruction::STD,
        0x6C => IEMInstruction::LODS,
        0xAA => IEMInstruction::STOS,
        0xA4 | 0xA5 => IEMInstruction::MOVS,
        0xA6 | 0xA7 => IEMInstruction::CMPS,
        0xAE | 0xAF => IEMInstruction::SCAS,
        0xEC | 0xED => IEMInstruction::IN,
        0xE6 | 0xE7 => IEMInstruction::OUT,
        _ => IEMInstruction::NOP,
    }
}

#[inline(always)]
pub fn iem_execute(cpu: &mut IEM_CPUSTATE, memory: &[u8]) -> VBoxResult<()> {
    let addr = cpu.regs.rip as usize;
    if addr >= memory.len() {
        return Err(VBoxError::Fail);
    }
    let opcode = memory[addr];
    let inst = iem_decode(opcode);
    execute_instruction(&inst, cpu, memory)?;
    cpu.regs.rip += 1;
    cpu.instruction_count += 1;
    Ok(())
}

#[inline(always)]
fn execute_instruction(inst: &IEMInstruction, cpu: &mut IEM_CPUSTATE, _memory: &[u8]) -> Result<(), VBoxError> {
    match inst {
        IEMInstruction::NOP => {}
        IEMInstruction::HLT => {
            cpu.regs.rflags |= (1 << 1);
        }
        IEMInstruction::MOV => {
            cpu.regs.rax = cpu.regs.rbx;
        }
        IEMInstruction::ADD => {
            let result = cpu.regs.rax.wrapping_add(cpu.regs.rbx);
            cpu.regs.rax = result;
            if result == 0 { cpu.regs.rflags |= (1 << 6); }
            else { cpu.regs.rflags &= !(1 << 6); }
        }
        IEMInstruction::SUB => {
            let result = cpu.regs.rax.wrapping_sub(cpu.regs.rbx);
            cpu.regs.rax = result;
            if result == 0 { cpu.regs.rflags |= (1 << 6); }
            else { cpu.regs.rflags &= !(1 << 6); }
        }
        IEMInstruction::XOR => {
            cpu.regs.rax ^= cpu.regs.rbx;
            if cpu.regs.rax == 0 { cpu.regs.rflags |= (1 << 6); }
            else { cpu.regs.rflags &= !(1 << 6); }
        }
        IEMInstruction::AND => {
            cpu.regs.rax &= cpu.regs.rbx;
            if cpu.regs.rax == 0 { cpu.regs.rflags |= (1 << 6); }
            else { cpu.regs.rflags &= !(1 << 6); }
        }
        IEMInstruction::OR => {
            cpu.regs.rax |= cpu.regs.rbx;
            if cpu.regs.rax == 0 { cpu.regs.rflags &= !(1 << 6); }
            else { cpu.regs.rflags |= (1 << 6); }
        }
        IEMInstruction::INC => {
            cpu.regs.rax = cpu.regs.rax.wrapping_add(1);
            if cpu.regs.rax == 0 { cpu.regs.rflags |= (1 << 6); }
            else { cpu.regs.rflags &= !(1 << 6); }
        }
        IEMInstruction::DEC => {
            cpu.regs.rax = cpu.regs.rax.wrapping_sub(1);
            if cpu.regs.rax == 0 { cpu.regs.rflags |= (1 << 6); }
            else { cpu.regs.rflags &= !(1 << 6); }
        }
        IEMInstruction::NOT => {
            cpu.regs.rax = !cpu.regs.rax;
        }
        IEMInstruction::PUSH => {
            cpu.regs.rsp = cpu.regs.rsp.wrapping_sub(8);
        }
        IEMInstruction::POP => {
            cpu.regs.rsp = cpu.regs.rsp.wrapping_add(8);
        }
        IEMInstruction::JMP => {
            cpu.regs.rip = cpu.regs.rbx;
        }
        IEMInstruction::CALL => {
            cpu.regs.rsp = cpu.regs.rsp.wrapping_sub(8);
            cpu.regs.rip = cpu.regs.rbx;
        }
        IEMInstruction::RET => {
            cpu.regs.rip = cpu.regs.rbx;
        }
        IEMInstruction::CMP => {
            let result = cpu.regs.rax.wrapping_sub(cpu.regs.rbx);
            if result == 0 {
                cpu.regs.rflags |= (1 << 6);
                cpu.regs.rflags |= (1 << 7);
            } else {
                cpu.regs.rflags &= !(1 << 6);
                cpu.regs.rflags &= !(1 << 7);
            }
        }
        IEMInstruction::TEST => {
            let result = cpu.regs.rax & cpu.regs.rbx;
            if result == 0 {
                cpu.regs.rflags |= (1 << 6);
            } else {
                cpu.regs.rflags &= !(1 << 6);
            }
        }
        IEMInstruction::LEA => {
            cpu.regs.rax = cpu.regs.rbx;
        }
        IEMInstruction::XCHG => {
            let tmp = cpu.regs.rax;
            cpu.regs.rax = cpu.regs.rbx;
            cpu.regs.rbx = tmp;
        }
        IEMInstruction::RDTSC | IEMInstruction::RDTSCP => {
            cpu.regs.rax = cpu.instruction_count;
        }
        IEMInstruction::CPUID => {
            cpu.regs.rax = 0x00000001;
            cpu.regs.rbx = 0;
            cpu.regs.rcx = 0;
            cpu.regs.rdx = 0;
        }
        IEMInstruction::VMCALL => {
            cpu.regs.rax = 0;
        }
        IEMInstruction::SHL => {
            cpu.regs.rax = cpu.regs.rax << 1;
        }
        IEMInstruction::SHR => {
            cpu.regs.rax = cpu.regs.rax >> 1;
        }
        IEMInstruction::SAR => {
            cpu.regs.rax = (cpu.regs.rax as i64 >> 1) as u64;
        }
        IEMInstruction::SHL_IMM => {
            cpu.regs.rax = cpu.regs.rax << 1;
        }
        IEMInstruction::SHR_IMM => {
            cpu.regs.rax = cpu.regs.rax >> 1;
        }
        IEMInstruction::IN => {
            cpu.regs.rax = 0;
        }
        IEMInstruction::OUT => {}
        IEMInstruction::CLC => {
            cpu.regs.rflags &= !(1 << 0);
        }
        IEMInstruction::STC => {
            cpu.regs.rflags |= (1 << 0);
        }
        IEMInstruction::CLI => {
            cpu.interrupts_disabled = true;
        }
        IEMInstruction::STI => {
            cpu.interrupts_disabled = false;
        }
        IEMInstruction::CLD => {
            cpu.regs.rflags &= !(1 << 10);
        }
        IEMInstruction::STD => {
            cpu.regs.rflags |= (1 << 10);
        }
        IEMInstruction::LODS => {
            cpu.regs.rax = cpu.regs.rdi;
        }
        IEMInstruction::STOS => {
            cpu.regs.rdi = cpu.regs.rax;
        }
        IEMInstruction::MOVS => {
            let tmp = cpu.regs.rax;
            cpu.regs.rax = cpu.regs.rsi;
            cpu.regs.rsi = tmp;
        }
        IEMInstruction::CMPS => {
            let result = cpu.regs.rax.wrapping_sub(cpu.regs.rsi);
            if result == 0 {
                cpu.regs.rflags |= (1 << 6);
            } else {
                cpu.regs.rflags &= !(1 << 6);
            }
        }
        IEMInstruction::SCAS => {
            let result = cpu.regs.rax.wrapping_sub(cpu.regs.rdi);
            if result == 0 {
                cpu.regs.rflags |= (1 << 6);
            } else {
                cpu.regs.rflags &= !(1 << 6);
            }
        }
        IEMInstruction::ENTER => {
            cpu.regs.rbp = cpu.regs.rsp;
        }
        IEMInstruction::LEAVE => {
            cpu.regs.rsp = cpu.regs.rbp;
        }
        _ => {
            cpu.regs.rip += 1;
        }
    }
    Ok(())
}

pub fn iem_init() -> Result<IemState, VBoxError> {
    let state = IemState::default();
    info!("IEM: Interpreter initialized");
    Ok(state)
}

#[inline(always)]
pub fn IEMExec(pVM: &mut crate::vm::VM) -> Result<(), VBoxError> {
    let mut cpu = IEM_CPUSTATE::new();
    let mem = vec![0u8; 4096];
    iem_execute(&mut cpu, &mem)?;
    Ok(())
}

pub fn IEMAllExec(pVM: &mut crate::vm::VM) -> Result<(), VBoxError> {
    iem_execute_loop(pVM)
}

#[inline(always)]
fn iem_execute_loop(pVM: &mut crate::vm::VM) -> Result<(), VBoxError> {
    loop {
        if pVM.enm_state != crate::em::EMState::Running {
            break;
        }
        for vcpu in pVM.cpus.iter_mut() {
            let _vcpu_state = vcpu;
            let mem = &pVM.p_guest_memory;
            let mut cpu = IEM_CPUSTATE::new();
            iem_execute(&mut cpu, mem)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iem_mov() {
        let mut cpu = IEM_CPUSTATE::new();
        cpu.regs.rax = 0;
        cpu.regs.rbx = 42;
        let mut memory = vec![0u8; 4096];
        let inst = IEMInstruction::MOV;
        execute_instruction(&inst, &mut cpu, &memory).unwrap();
        assert_eq!(cpu.regs.rax, 42);
    }

    #[test]
    fn test_iem_add() {
        let mut cpu = IEM_CPUSTATE::new();
        cpu.regs.rax = 10;
        cpu.regs.rbx = 32;
        let mut memory = vec![0u8; 4096];
        let inst = IEMInstruction::ADD;
        execute_instruction(&inst, &mut cpu, &memory).unwrap();
        assert_eq!(cpu.regs.rax, 42);
    }

    #[test]
    fn test_iem_sub() {
        let mut cpu = IEM_CPUSTATE::new();
        cpu.regs.rax = 50;
        cpu.regs.rbx = 8;
        let mut memory = vec![0u8; 4096];
        let inst = IEMInstruction::SUB;
        execute_instruction(&inst, &mut cpu, &memory).unwrap();
        assert_eq!(cpu.regs.rax, 42);
    }

    #[test]
    fn test_iem_xor() {
        let mut cpu = IEM_CPUSTATE::new();
        cpu.regs.rax = 0xFF;
        cpu.regs.rbx = 0xFF;
        let mut memory = vec![0u8; 4096];
        let inst = IEMInstruction::XOR;
        execute_instruction(&inst, &mut cpu, &memory).unwrap();
        assert_eq!(cpu.regs.rax, 0);
    }

    #[test]
    fn test_iem_cmp() {
        let mut cpu = IEM_CPUSTATE::new();
        cpu.regs.rax = 10;
        cpu.regs.rbx = 10;
        let mut memory = vec![0u8; 4096];
        let inst = IEMInstruction::CMP;
        execute_instruction(&inst, &mut cpu, &memory).unwrap();
        assert!(cpu.regs.rflags & (1 << 6) != 0);
    }

    #[test]
    fn test_iem_jmp() {
        let mut cpu = IEM_CPUSTATE::new();
        cpu.regs.rax = 0;
        cpu.regs.rbx = 0x100;
        let mut memory = vec![0u8; 4096];
        let inst = IEMInstruction::JMP;
        execute_instruction(&inst, &mut cpu, &memory).unwrap();
        assert_eq!(cpu.regs.rip, 0x100);
    }

    #[test]
    fn test_iem_push_pop() {
        let mut cpu = IEM_CPUSTATE::new();
        cpu.regs.rax = 0xDEADBEEF;
        cpu.regs.rsp = 0x1000;
        let mut memory = vec![0u8; 4096];
        let inst = IEMInstruction::PUSH;
        execute_instruction(&inst, &mut cpu, &memory).unwrap();
        assert_eq!(cpu.regs.rsp, 0xFF8);
        let inst = IEMInstruction::POP;
        execute_instruction(&inst, &mut cpu, &memory).unwrap();
        assert_eq!(cpu.regs.rsp, 0x1000);
    }

    #[test]
    fn test_iem_decode_mov() {
        let inst = iem_decode(0x88);
        assert_eq!(inst, IEMInstruction::MOV);
    }

    #[test]
    fn test_iem_decode_add() {
        let inst = iem_decode(0x01);
        assert_eq!(inst, IEMInstruction::ADD);
    }

    #[test]
    fn test_iem_decode_jmp() {
        let inst = iem_decode(0xE9);
        assert_eq!(inst, IEMInstruction::JMP);
    }

    #[test]
    fn test_iem_decode_call() {
        let inst = iem_decode(0xE8);
        assert_eq!(inst, IEMInstruction::CALL);
    }

    #[test]
    fn test_iem_decode_ret() {
        let inst = iem_decode(0xC3);
        assert_eq!(inst, IEMInstruction::RET);
    }

    #[test]
    fn test_iem_decode_push() {
        let inst = iem_decode(0x50);
        assert_eq!(inst, IEMInstruction::PUSH);
    }

    #[test]
    fn test_iem_decode_pop() {
        let inst = iem_decode(0x58);
        assert_eq!(inst, IEMInstruction::POP);
    }

    #[test]
    fn test_iem_decode_cmp() {
        let inst = iem_decode(0x39);
        assert_eq!(inst, IEMInstruction::CMP);
    }

    #[test]
    fn test_iem_decode_test() {
        let inst = iem_decode(0x84);
        assert_eq!(inst, IEMInstruction::TEST);
    }

    #[test]
    fn test_iem_decode_lea() {
        let inst = iem_decode(0x8D);
        assert_eq!(inst, IEMInstruction::LEA);
    }

    #[test]
    fn test_iem_decode_nop() {
        let inst = iem_decode(0x90);
        assert_eq!(inst, IEMInstruction::NOP);
    }

    #[test]
    fn test_iem_decode_hlt() {
        let inst = iem_decode(0xF4);
        assert_eq!(inst, IEMInstruction::HLT);
    }

    #[test]
    fn test_iem_decode_int() {
        let inst = iem_decode(0xCC);
        assert_eq!(inst, IEMInstruction::INT);
    }

    #[test]
    fn test_iem_decode_xchg() {
        let inst = iem_decode(0x87);
        assert_eq!(inst, IEMInstruction::XCHG);
    }

    #[test]
    fn test_iem_decode_shl() {
        let inst = iem_decode(0xC0);
        assert_eq!(inst, IEMInstruction::SHL_IMM);
    }

    #[test]
    fn test_iem_decode_inc() {
        let inst = iem_decode(0xFE);
        assert_eq!(inst, IEMInstruction::INC);
    }

    #[test]
    fn test_iem_decode_dec() {
        let inst = iem_decode(0x48);
        assert_eq!(inst, IEMInstruction::DEC);
    }

    #[test]
    fn test_iem_decode_clc() {
        let inst = iem_decode(0xF8);
        assert_eq!(inst, IEMInstruction::CLC);
    }

    #[test]
    fn test_iem_decode_stc() {
        let inst = iem_decode(0xF9);
        assert_eq!(inst, IEMInstruction::STC);
    }

    #[test]
    fn test_iem_decode_cli() {
        let inst = iem_decode(0xFA);
        assert_eq!(inst, IEMInstruction::CLI);
    }

    #[test]
    fn test_iem_decode_sti() {
        let inst = iem_decode(0xFB);
        assert_eq!(inst, IEMInstruction::STI);
    }

    #[test]
    fn test_iem_decode_cld() {
        let inst = iem_decode(0xFC);
        assert_eq!(inst, IEMInstruction::CLD);
    }

    #[test]
    fn test_iem_decode_std() {
        let inst = iem_decode(0xFD);
        assert_eq!(inst, IEMInstruction::STD);
    }

    #[test]
    fn test_cpu_id() {
        let result = crate::cpu::cpum_get_id(0, 0);
        assert_eq!(result.0, 0x0F81078F);
    }

    #[test]
    fn test_cpum_ctx() {
        let ctx = crate::cpu::CPUMCTX::new();
        assert_eq!(ctx.guest.mode, crate::cpu::CpuMode::Protected);
    }
}
