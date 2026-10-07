use rusty64::{
    elf,
    memory::{Memory, MAX_GUEST_BYTES},
    Cpu, Status,
};

fn machine(code: &[u8]) -> Cpu {
    let mut cpu = Cpu::new(16 * 1024 * 1024).unwrap();
    cpu.rip = 0x1000;
    cpu.registers[4] = cpu.memory.size - 4096;
    cpu.memory.write(cpu.rip, code).unwrap();
    cpu
}

fn halted(cpu: &mut Cpu) {
    assert_eq!(cpu.run(10000), &Status::Halted);
}

#[test]
fn compiled_elf_guest_executes_above_four_gib() {
    let mut cpu = elf::load(include_bytes!("../guests/demo.elf"), MAX_GUEST_BYTES).unwrap();
    assert_eq!(cpu.run(10000), &Status::Exited(0));
    assert!(String::from_utf8(cpu.output.clone())
        .unwrap()
        .contains("memory above 4 GiB: OK"));
    assert_eq!(cpu.memory.read(0x100002000, 8).unwrap(), 0x300000003);
    assert!(cpu.memory.committed_bytes() < 65536);
    assert!(cpu.instructions > 300);
}

#[test]
fn rex_registers_and_immediate_widths() {
    let mut cpu = machine(&[
        0x49, 0xb8, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x4d, 0x89, 0xc7, 0xf4,
    ]);
    halted(&mut cpu);
    assert_eq!(cpu.registers[8], 0x1122334455667788);
    assert_eq!(cpu.registers[15], cpu.registers[8]);
}

#[test]
fn partial_registers_and_zero_extension() {
    let mut cpu = machine(&[
        0xb4, 0x12, 0x66, 0xb8, 0xcd, 0xab, 0x41, 0xb9, 0x78, 0x56, 0x34, 0x12, 0xf4,
    ]);
    cpu.registers[0] = u64::MAX;
    cpu.registers[9] = u64::MAX;
    cpu.run(1);
    assert_eq!(cpu.registers[0], 0xffffffffffff12ff);
    halted(&mut cpu);
    assert_eq!(cpu.registers[0], 0xffffffffffffabcd);
    assert_eq!(cpu.registers[9], 0x12345678);
}

#[test]
fn rex_byte_register_selects_spl_not_ah() {
    let mut cpu = machine(&[0x40, 0xb4, 0x80, 0xf4]);
    let old = cpu.registers[4];
    halted(&mut cpu);
    assert_eq!(cpu.registers[4], old & !255 | 128);
    assert_eq!(cpu.registers[0], 0);
}

#[test]
fn rip_relative_store_uses_end_of_immediate() {
    // mov qword [rip+0x10], -1; hlt
    let mut cpu = machine(&[
        0x48, 0xc7, 0x05, 0x10, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xf4,
    ]);
    halted(&mut cpu);
    assert_eq!(cpu.memory.read(0x101b, 8).unwrap(), u64::MAX);
}

#[test]
fn sib_extended_index_base_and_negative_displacement() {
    // mov rax, [r12+r9*4-8]
    let mut cpu = machine(&[0x4b, 0x8b, 0x44, 0x8c, 0xf8, 0xf4]);
    cpu.registers[12] = 0x2000;
    cpu.registers[9] = 4;
    cpu.memory
        .write(0x2008, &0xfeedbeef12345678u64.to_le_bytes())
        .unwrap();
    halted(&mut cpu);
    assert_eq!(cpu.registers[0], 0xfeedbeef12345678);
}

#[test]
fn indirect_call_return_and_stack() {
    // call rax; hlt; add rbx, 7; ret
    let mut cpu = machine(&[0xff, 0xd0, 0xf4, 0x48, 0x83, 0xc3, 7, 0xc3]);
    cpu.registers[0] = 0x1003;
    let stack = cpu.registers[4];
    halted(&mut cpu);
    assert_eq!(cpu.registers[3], 7);
    assert_eq!(cpu.registers[4], stack);
}

#[test]
fn signed_branches_and_setcc() {
    // cmp eax, 1; jl +5; mov ebx, 1; setl dl; hlt
    let mut cpu = machine(&[
        0x83, 0xf8, 1, 0x7c, 5, 0xbb, 1, 0, 0, 0, 0x0f, 0x9c, 0xc2, 0xf4,
    ]);
    cpu.registers[0] = 0xffffffff;
    halted(&mut cpu);
    assert_eq!(cpu.registers[3], 0);
    assert_eq!(cpu.registers[2], 1);
}

#[test]
fn shift_mask_sign_extension_and_multiply() {
    // mov rax, -16; sar rax, 68 (masked to 4); imul rax, rax, -7
    let mut cpu = machine(&[
        0x48, 0xc7, 0xc0, 0xf0, 0xff, 0xff, 0xff, 0x48, 0xc1, 0xf8, 68, 0x48, 0x6b, 0xc0, 0xf9,
        0xf4,
    ]);
    halted(&mut cpu);
    assert_eq!(cpu.registers[0], 7);
    assert_eq!(cpu.flags & 0x801, 0);
}

#[test]
fn movzx_high_byte_destination_is_full_register() {
    // movzx esp, ah
    let mut cpu = machine(&[0x0f, 0xb6, 0xe4, 0xf4]);
    cpu.registers[0] = 0xab00;
    halted(&mut cpu);
    assert_eq!(cpu.registers[4], 0xab);
}

#[test]
fn arithmetic_right_shift_beyond_byte_width_keeps_sign_in_carry() {
    let mut cpu = machine(&[0xc0, 0xf8, 9, 0xf4]); // sar al, 9
    cpu.registers[0] = 0x80;
    halted(&mut cpu);
    assert_eq!(cpu.registers[0], 0xff);
    assert_eq!(cpu.flags & 1, 1);
}

#[test]
fn increment_preserves_carry_and_sets_overflow() {
    let mut cpu = machine(&[0xf9, 0x48, 0xff, 0xc0, 0xf4]);
    cpu.registers[0] = i64::MAX as u64;
    halted(&mut cpu);
    assert_eq!(cpu.registers[0], 1 << 63);
    assert_eq!(cpu.flags & 0x801, 0x801);
}

#[test]
fn faults_are_bounded_and_report_original_instruction_pointer() {
    for code in [&[0x0f, 0x0b][..], &[0x67, 0x90][..], &[0x66; 16][..]] {
        let mut cpu = machine(code);
        assert!(matches!(cpu.run(100), Status::Fault(_)));
        assert_eq!(cpu.rip, 0x1000);
        assert_eq!(cpu.instructions, 0);
    }
    let mut cpu = machine(&[0x48, 0x8b, 0x00]);
    cpu.registers[0] = u64::MAX;
    assert!(matches!(cpu.run(1), Status::Fault(message) if message.contains("out of bounds")));
    assert_eq!(cpu.registers[0], u64::MAX);
}

#[test]
fn infinite_guest_yields_at_instruction_budget() {
    let mut cpu = machine(&[0xeb, 0xfe]);
    assert_eq!(cpu.run(1000), &Status::Running);
    assert_eq!(cpu.instructions, 1000);
    assert_eq!(cpu.rip, 0x1000);
}

#[test]
fn sparse_memory_cross_page_access_and_overflow() {
    let mut memory = Memory::new(MAX_GUEST_BYTES).unwrap();
    let at = 0x1_0000_0ffc;
    assert_eq!(memory.read(at, 8).unwrap(), 0);
    memory.write(at, &u64::MAX.to_le_bytes()).unwrap();
    assert_eq!(memory.read(at, 8).unwrap(), u64::MAX);
    assert_eq!(memory.committed_bytes(), 8192);
    assert!(memory.write(MAX_GUEST_BYTES - 1, &[1, 2]).is_err());
    assert_eq!(memory.byte(MAX_GUEST_BYTES - 1).unwrap(), 0);
    assert!(memory.check(u64::MAX, 4).is_err());
}

#[test]
fn loader_rejects_invalid_and_dynamic_images() {
    let original = include_bytes!("../guests/demo.elf");
    for length in [0, 6, 16, 63, 80] {
        assert!(elf::load(&original[..length], MAX_GUEST_BYTES).is_err());
    }
    for (offset, value) in [(4, 1), (5, 2), (16, 3), (18, 3), (54, 55), (56, 0)] {
        let mut bad = original.to_vec();
        bad[offset] = value;
        assert!(elf::load(&bad, MAX_GUEST_BYTES).is_err(), "offset {offset}");
    }
    let mut bad = original.to_vec();
    let phoff = u64::from_le_bytes(bad[32..40].try_into().unwrap()) as usize;
    bad[phoff..phoff + 4].copy_from_slice(&3u32.to_le_bytes());
    assert!(elf::load(&bad, MAX_GUEST_BYTES)
        .err()
        .unwrap()
        .contains("Dynamic"));
    bad = original.to_vec();
    bad[phoff + 32..phoff + 40].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(elf::load(&bad, MAX_GUEST_BYTES).is_err());
}

#[test]
fn missing_syscalls_and_output_overruns_fault() {
    let mut cpu = machine(&[0x0f, 5]);
    cpu.registers[0] = 9; // mmap is not implemented
    assert!(
        matches!(cpu.run(1), Status::Fault(message) if message.contains("Unsupported syscall 9"))
    );
    let mut cpu = machine(&[0x0f, 5]);
    cpu.registers[0] = 1;
    cpu.registers[7] = 1;
    cpu.registers[2] = 65537;
    assert!(matches!(cpu.run(1), Status::Fault(message) if message.contains("output exceeded")));
}

// Compare the interpreter with actual host CPU results, including arithmetic flags.
#[cfg(target_arch = "x86_64")]
#[test]
fn arithmetic_matches_native_x86_64() {
    use std::arch::asm;
    let values = [
        0,
        1,
        15,
        16,
        127,
        255,
        0x80000000,
        0xffffffff,
        i64::MAX as u64,
        1 << 63,
        u64::MAX - 1,
        u64::MAX,
    ];
    for operation in 0..8u8 {
        for lhs in values {
            for rhs in values {
                for carry in [0u64, 1] {
                    let mut result = lhs;
                    let flags: u64;
                    macro_rules! native {
                        ($op:literal) => { asm!("bt {carry}, 0", concat!($op, " {a}, {b}"), "pushfq", "pop {flags}",
                            a = inout(reg) result, b = in(reg) rhs, carry = in(reg) carry, flags = lateout(reg) flags) };
                    }
                    unsafe {
                        match operation {
                            0 => native!("add"),
                            1 => native!("or"),
                            2 => native!("adc"),
                            3 => native!("sbb"),
                            4 => native!("and"),
                            5 => native!("sub"),
                            6 => native!("xor"),
                            7 => native!("cmp"),
                            _ => unreachable!(),
                        }
                    }
                    let mut cpu = machine(&[0x48, operation * 8 + 1, 0xd8, 0xf4]);
                    cpu.registers[0] = lhs;
                    cpu.registers[3] = rhs;
                    cpu.flags = 2 | carry;
                    halted(&mut cpu);
                    let tested_flags = if [1, 4, 6].contains(&operation) {
                        0x8c5
                    } else {
                        0x8d5
                    };
                    assert_eq!(
                        cpu.registers[0], result,
                        "op={operation} lhs={lhs:x} rhs={rhs:x}"
                    );
                    assert_eq!(
                        cpu.flags & tested_flags,
                        flags & tested_flags,
                        "op={operation} lhs={lhs:x} rhs={rhs:x} carry={carry}"
                    );
                }
            }
        }
    }
}
