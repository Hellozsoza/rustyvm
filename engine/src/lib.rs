//! Rusty64: an experimental integer x86-64 interpreter, compiled to WASM.
//! This executes directly in long mode; firmware and OS boot are not implemented.
pub mod elf;
pub mod memory;
#[cfg(target_arch = "wasm32")]
mod wasm;

use memory::Memory;

const CF: u64 = 1;
const PF: u64 = 1 << 2;
const AF: u64 = 1 << 4;
const ZF: u64 = 1 << 6;
const SF: u64 = 1 << 7;
const OF: u64 = 1 << 11;

#[derive(Clone, Copy)]
enum Operand {
    Register(usize),
    HighByte(usize),
    Memory(u64),
    Relative(i64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Running,
    Halted,
    Exited(u32),
    Fault(String),
}

pub struct Cpu {
    pub registers: [u64; 16],
    pub rip: u64,
    pub flags: u64,
    pub memory: Memory,
    pub status: Status,
    pub output: Vec<u8>,
    pub instructions: u64,
    instruction_start: u64,
}

fn mask(bits: u32) -> u64 {
    if bits == 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}
fn signed(value: u64, bits: u32) -> i128 {
    ((value << (64 - bits)) as i64 >> (64 - bits)) as i128
}

impl Cpu {
    pub fn new(memory_size: u64) -> Result<Self, String> {
        Ok(Self {
            registers: [0; 16],
            rip: 0,
            flags: 2,
            memory: Memory::new(memory_size)?,
            status: Status::Running,
            output: Vec::new(),
            instructions: 0,
            instruction_start: 0,
        })
    }

    fn immediate(&mut self, bytes: usize) -> Result<u64, String> {
        if self
            .rip
            .checked_add(bytes as u64)
            .is_none_or(|end| end - self.instruction_start > 15)
        {
            return Err("Instruction exceeds 15 bytes".into());
        }
        let value = self.memory.read(self.rip, bytes)?;
        self.rip += bytes as u64;
        Ok(value)
    }

    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.immediate(1)? as u8)
    }

    fn register(code: usize, bits: u32, rex: u8) -> Operand {
        if bits == 8 && rex == 0 && (4..8).contains(&code) {
            Operand::HighByte(code - 4)
        } else {
            Operand::Register(code)
        }
    }

    fn modrm(&mut self, bits: u32, rex: u8) -> Result<(Operand, Operand, u8), String> {
        let byte = self.byte()?;
        let mode = byte >> 6;
        let group = (byte >> 3) & 7;
        let reg = Self::register(group as usize + if rex & 4 != 0 { 8 } else { 0 }, bits, rex);
        let rm = (byte & 7) as usize;
        if mode == 3 {
            return Ok((
                reg,
                Self::register(rm + if rex & 1 != 0 { 8 } else { 0 }, bits, rex),
                group,
            ));
        }
        let mut address;
        if rm == 4 {
            let sib = self.byte()?;
            let index = ((sib >> 3) & 7) as usize;
            let base = (sib & 7) as usize;
            address = if index == 4 && rex & 2 == 0 {
                0
            } else {
                self.registers[index + if rex & 2 != 0 { 8 } else { 0 }]
                    .wrapping_shl((sib >> 6) as u32)
            };
            if mode == 0 && base == 5 {
                address = address.wrapping_add(self.immediate(4)? as i32 as i64 as u64);
            } else {
                address =
                    address.wrapping_add(self.registers[base + if rex & 1 != 0 { 8 } else { 0 }]);
            }
        } else if mode == 0 && rm == 5 {
            return Ok((
                reg,
                Operand::Relative(self.immediate(4)? as i32 as i64),
                group,
            ));
        } else {
            address = self.registers[rm + if rex & 1 != 0 { 8 } else { 0 }];
        }
        if mode == 1 {
            address = address.wrapping_add(self.byte()? as i8 as i64 as u64);
        }
        if mode == 2 {
            address = address.wrapping_add(self.immediate(4)? as i32 as i64 as u64);
        }
        Ok((reg, Operand::Memory(address), group))
    }

    fn address(&self, operand: Operand) -> Result<u64, String> {
        match operand {
            Operand::Memory(address) => Ok(address),
            Operand::Relative(offset) => Ok(self.rip.wrapping_add(offset as u64)),
            _ => Err("Instruction requires a memory operand".into()),
        }
    }

    fn read(&self, operand: Operand, bits: u32) -> Result<u64, String> {
        Ok(match operand {
            Operand::Register(reg) => self.registers[reg] & mask(bits),
            Operand::HighByte(reg) => self.registers[reg] >> 8 & 255,
            _ => self
                .memory
                .read(self.address(operand)?, (bits / 8) as usize)?,
        })
    }

    fn write(&mut self, operand: Operand, value: u64, bits: u32) -> Result<(), String> {
        let value = value & mask(bits);
        match operand {
            Operand::Register(reg) => {
                // x86-64 zero-extends 32-bit writes; 8/16-bit writes preserve upper bits.
                self.registers[reg] = if bits >= 32 {
                    value
                } else {
                    self.registers[reg] & !mask(bits) | value
                };
            }
            Operand::HighByte(reg) => {
                self.registers[reg] = self.registers[reg] & !0xff00 | value << 8
            }
            _ => self.memory.write(
                self.address(operand)?,
                &value.to_le_bytes()[..(bits / 8) as usize],
            )?,
        }
        Ok(())
    }

    fn flag(&mut self, bit: u64, enabled: bool) {
        if enabled {
            self.flags |= bit;
        } else {
            self.flags &= !bit;
        }
    }

    fn result_flags(&mut self, value: u64, bits: u32) {
        self.flag(ZF, value & mask(bits) == 0);
        self.flag(SF, value & (1 << (bits - 1)) != 0);
        self.flag(PF, (value as u8).count_ones().is_multiple_of(2));
    }

    fn alu(&mut self, operation: u8, lhs: u64, rhs: u64, bits: u32) -> Result<u64, String> {
        let carry = u64::from(self.flags & CF != 0);
        let (value, arithmetic, subtract, extra) = match operation {
            0 => (lhs.wrapping_add(rhs), true, false, 0),
            1 => (lhs | rhs, false, false, 0),
            2 => (
                lhs.wrapping_add(rhs).wrapping_add(carry),
                true,
                false,
                carry,
            ),
            3 => (lhs.wrapping_sub(rhs).wrapping_sub(carry), true, true, carry),
            4 => (lhs & rhs, false, false, 0),
            5 | 7 => (lhs.wrapping_sub(rhs), true, true, 0),
            6 => (lhs ^ rhs, false, false, 0),
            _ => return Err("Unknown arithmetic operation".into()),
        };
        let result = value & mask(bits);
        self.flags &= !(CF | OF | AF);
        if arithmetic {
            let a = lhs as u128;
            let b = rhs as u128 + extra as u128;
            self.flag(
                CF,
                if subtract {
                    a < b
                } else {
                    a + b > mask(bits) as u128
                },
            );
            let s = if subtract {
                signed(lhs, bits) - signed(rhs, bits) - extra as i128
            } else {
                signed(lhs, bits) + signed(rhs, bits) + extra as i128
            };
            self.flag(OF, s < -(1i128 << (bits - 1)) || s >= (1i128 << (bits - 1)));
            self.flag(AF, (lhs ^ rhs ^ result) & 16 != 0);
        }
        self.result_flags(result, bits);
        Ok(result)
    }

    fn condition(&self, code: u8) -> bool {
        let c = self.flags & CF != 0;
        let z = self.flags & ZF != 0;
        let s = self.flags & SF != 0;
        let o = self.flags & OF != 0;
        match code & 15 {
            0 => o,
            1 => !o,
            2 => c,
            3 => !c,
            4 => z,
            5 => !z,
            6 => c || z,
            7 => !c && !z,
            8 => s,
            9 => !s,
            10 => self.flags & PF != 0,
            11 => self.flags & PF == 0,
            12 => s != o,
            13 => s == o,
            14 => z || s != o,
            15 => !z && s == o,
            _ => unreachable!(),
        }
    }

    fn push(&mut self, value: u64, bits: u32) -> Result<(), String> {
        let address = self.registers[4].wrapping_sub((bits / 8) as u64);
        self.memory
            .write(address, &value.to_le_bytes()[..(bits / 8) as usize])?;
        self.registers[4] = address;
        Ok(())
    }

    fn pop(&mut self, bits: u32) -> Result<u64, String> {
        let value = self.memory.read(self.registers[4], (bits / 8) as usize)?;
        self.registers[4] = self.registers[4].wrapping_add((bits / 8) as u64);
        Ok(value)
    }

    fn multiply(&mut self, dest: Operand, lhs: u64, rhs: u64, bits: u32) -> Result<(), String> {
        let value = signed(lhs, bits) * signed(rhs, bits);
        self.flag(CF, signed(value as u64 & mask(bits), bits) != value);
        self.flag(OF, self.flags & CF != 0);
        self.write(dest, value as u64, bits)
    }

    fn syscall(&mut self) -> Result<(), String> {
        self.registers[1] = self.rip;
        self.registers[11] = self.flags;
        match self.registers[0] {
            1 => {
                let fd = self.registers[7];
                if fd != 1 && fd != 2 {
                    self.registers[0] = (-9i64) as u64;
                    return Ok(());
                }
                let address = self.registers[6];
                let length = self.registers[2];
                if length > 65536 || self.output.len() as u64 + length > 1024 * 1024 {
                    return Err("Guest output exceeded its budget".into());
                }
                self.memory.check(address, length)?;
                for at in address..address + length {
                    self.output.push(self.memory.byte(at)?);
                }
                self.registers[0] = length;
            }
            60 | 231 => self.status = Status::Exited(self.registers[7] as u32),
            call => {
                return Err(format!(
                    "Unsupported syscall {call}; only write, exit and exit_group are implemented"
                ))
            }
        }
        Ok(())
    }

    pub fn run(&mut self, budget: u32) -> &Status {
        for _ in 0..budget.min(100_000) {
            if self.status != Status::Running {
                break;
            }
            self.instruction_start = self.rip;
            let registers = self.registers;
            let flags = self.flags;
            if let Err(error) = self.step() {
                self.rip = self.instruction_start;
                self.registers = registers;
                self.flags = flags;
                self.status = Status::Fault(format!("{error} at RIP {:#018x}", self.rip));
                break;
            }
            self.instructions += 1;
        }
        &self.status
    }

    fn step(&mut self) -> Result<(), String> {
        let mut rex = 0;
        let mut narrow = false;
        let opcode = loop {
            let byte = self.byte()?;
            match byte {
                0x66 => {
                    narrow = true;
                    rex = 0;
                }
                0x40..=0x4f => rex = byte,
                _ => break byte,
            }
        };
        let bits = if rex & 8 != 0 {
            64
        } else if narrow {
            16
        } else {
            32
        };
        let stack_bits = if narrow { 16 } else { 64 };
        let reg_index = (opcode & 7) as usize + if rex & 1 != 0 { 8 } else { 0 };
        match opcode {
            0x00..=0x3d if opcode & 7 <= 5 => {
                let operation = opcode >> 3;
                let form = opcode & 7;
                let width = if form & 1 == 0 { 8 } else { bits };
                let (dest, rhs) = if form <= 3 {
                    let (reg, rm, _) = self.modrm(width, rex)?;
                    let (dest, source) = if form & 2 == 0 { (rm, reg) } else { (reg, rm) };
                    (dest, self.read(source, width)?)
                } else {
                    let value = self.immediate((width.min(32) / 8) as usize)?;
                    (
                        Operand::Register(0),
                        if width == 64 {
                            value as i32 as i64 as u64
                        } else {
                            value
                        },
                    )
                };
                let value = self.alu(operation, self.read(dest, width)?, rhs, width)?;
                if operation != 7 {
                    self.write(dest, value, width)?;
                }
            }
            0x50..=0x57 => self.push(self.registers[reg_index], stack_bits)?,
            0x58..=0x5f => {
                let value = self.pop(stack_bits)?;
                self.write(Operand::Register(reg_index), value, stack_bits)?;
            }
            0x63 => {
                let (reg, rm, _) = self.modrm(32, rex)?;
                let value = self.read(rm, 32)? as i32 as i64 as u64;
                self.write(reg, value, bits)?;
            }
            0x68 | 0x6a => {
                let value = if opcode == 0x6a {
                    self.byte()? as i8 as i64 as u64
                } else if narrow {
                    self.immediate(2)?
                } else {
                    self.immediate(4)? as i32 as i64 as u64
                };
                self.push(value, stack_bits)?;
            }
            0x69 | 0x6b => {
                let (reg, rm, _) = self.modrm(bits, rex)?;
                let immediate = if opcode == 0x6b {
                    self.byte()? as i8 as i64 as u64
                } else if bits == 16 {
                    self.immediate(2)?
                } else {
                    self.immediate(4)? as i32 as i64 as u64
                };
                self.multiply(reg, self.read(rm, bits)?, immediate, bits)?;
            }
            0x70..=0x7f => {
                let offset = self.byte()? as i8 as i64;
                if self.condition(opcode) {
                    self.rip = self.rip.wrapping_add(offset as u64);
                }
            }
            0x80 | 0x81 | 0x83 => {
                let width = if opcode == 0x80 { 8 } else { bits };
                let (_, rm, operation) = self.modrm(width, rex)?;
                let rhs = if opcode == 0x83 {
                    self.byte()? as i8 as i64 as u64 & mask(width)
                } else {
                    let v = self.immediate((width.min(32) / 8) as usize)?;
                    if width == 64 {
                        v as i32 as i64 as u64
                    } else {
                        v
                    }
                };
                let result = self.alu(operation, self.read(rm, width)?, rhs, width)?;
                if operation != 7 {
                    self.write(rm, result, width)?;
                }
            }
            0x84 | 0x85 => {
                let width = if opcode == 0x84 { 8 } else { bits };
                let (reg, rm, _) = self.modrm(width, rex)?;
                self.alu(4, self.read(reg, width)?, self.read(rm, width)?, width)?;
            }
            0x88..=0x8b => {
                let width = if opcode & 1 == 0 { 8 } else { bits };
                let (reg, rm, _) = self.modrm(width, rex)?;
                let (dest, source) = if opcode & 2 == 0 {
                    (rm, reg)
                } else {
                    (reg, rm)
                };
                self.write(dest, self.read(source, width)?, width)?;
            }
            0x8d => {
                let (reg, rm, _) = self.modrm(bits, rex)?;
                self.write(reg, self.address(rm)?, bits)?;
            }
            0x90..=0x97 => {
                if opcode != 0x90 || rex & 1 != 0 {
                    let value = self.read(Operand::Register(reg_index), bits)?;
                    self.write(Operand::Register(reg_index), self.registers[0], bits)?;
                    self.write(Operand::Register(0), value, bits)?;
                }
            }
            0x98 => {
                let source_width = bits / 2;
                self.write(
                    Operand::Register(0),
                    signed(self.registers[0], source_width) as u64,
                    bits,
                )?;
            }
            0x99 => {
                let value = if self.registers[0] & (1 << (bits - 1)) != 0 {
                    u64::MAX
                } else {
                    0
                };
                self.write(Operand::Register(2), value, bits)?;
            }
            0xa8 | 0xa9 => {
                let width = if opcode == 0xa8 { 8 } else { bits };
                let immediate = self.immediate((width.min(32) / 8) as usize)?;
                let immediate = if width == 64 {
                    immediate as i32 as i64 as u64
                } else {
                    immediate
                };
                self.alu(4, self.registers[0] & mask(width), immediate, width)?;
            }
            0xb0..=0xb7 => {
                let value = self.byte()?;
                self.write(Self::register(reg_index, 8, rex), value as u64, 8)?;
            }
            0xb8..=0xbf => {
                let value = self.immediate((bits / 8) as usize)?;
                self.write(Operand::Register(reg_index), value, bits)?;
            }
            0xc0 | 0xc1 | 0xd0..=0xd3 => {
                let width = if opcode & 1 == 0 { 8 } else { bits };
                let (_, rm, group) = self.modrm(width, rex)?;
                if ![4, 5, 7].contains(&group) {
                    return Err("Rotate instructions are not implemented".into());
                }
                let count = match opcode {
                    0xc0 | 0xc1 => self.byte()? as u32,
                    0xd0 | 0xd1 => 1,
                    _ => self.registers[1] as u32,
                } & if width == 64 { 63 } else { 31 };
                if count != 0 {
                    let value = self.read(rm, width)?;
                    let result = match group {
                        4 => value.wrapping_shl(count) & mask(width),
                        5 => value >> count,
                        _ => (signed(value, width) >> count) as u64 & mask(width),
                    };
                    self.flag(
                        CF,
                        if group == 4 {
                            count <= width && value & (1 << (width - count)) != 0
                        } else if group == 7 && count >= width {
                            value & (1 << (width - 1)) != 0
                        } else {
                            value & (1 << (count - 1)) != 0
                        },
                    );
                    if count == 1 {
                        self.flag(
                            OF,
                            match group {
                                4 => (result >> (width - 1) != 0) != (self.flags & CF != 0),
                                5 => value >> (width - 1) != 0,
                                _ => false,
                            },
                        );
                    }
                    self.result_flags(result, width);
                    self.write(rm, result, width)?;
                }
            }
            0xc2 | 0xc3 => {
                let extra = if opcode == 0xc2 {
                    self.immediate(2)?
                } else {
                    0
                };
                self.rip = self.pop(stack_bits)?;
                self.registers[4] = self.registers[4].wrapping_add(extra);
            }
            0xc6 | 0xc7 => {
                let width = if opcode == 0xc6 { 8 } else { bits };
                let (_, rm, group) = self.modrm(width, rex)?;
                if group != 0 {
                    return Err("Unsupported MOV group".into());
                }
                let value = self.immediate((width.min(32) / 8) as usize)?;
                self.write(
                    rm,
                    if width == 64 {
                        value as i32 as i64 as u64
                    } else {
                        value
                    },
                    width,
                )?;
            }
            0xc9 => {
                let value = self
                    .memory
                    .read(self.registers[5], (stack_bits / 8) as usize)?;
                self.registers[4] = self.registers[5].wrapping_add((stack_bits / 8) as u64);
                self.write(Operand::Register(5), value, stack_bits)?;
            }
            0xe8 | 0xe9 | 0xeb => {
                let offset = if opcode == 0xeb {
                    self.byte()? as i8 as i64
                } else {
                    self.immediate(4)? as i32 as i64
                };
                let target = self.rip.wrapping_add(offset as u64);
                if opcode == 0xe8 {
                    self.push(self.rip, 64)?;
                }
                self.rip = target;
            }
            0xf4 => self.status = Status::Halted,
            0xf8 => self.flag(CF, false),
            0xf9 => self.flag(CF, true),
            0xfc => self.flag(1 << 10, false),
            0xf6 | 0xf7 => {
                let width = if opcode == 0xf6 { 8 } else { bits };
                let (_, rm, group) = self.modrm(width, rex)?;
                match group {
                    0 => {
                        let value = self.immediate((width.min(32) / 8) as usize)?;
                        let value = if width == 64 {
                            value as i32 as i64 as u64
                        } else {
                            value
                        };
                        self.alu(4, self.read(rm, width)?, value, width)?;
                    }
                    2 => self.write(rm, !self.read(rm, width)?, width)?,
                    3 => {
                        let value = self.alu(5, 0, self.read(rm, width)?, width)?;
                        self.write(rm, value, width)?;
                    }
                    _ => return Err("One-operand multiply/divide is not implemented".into()),
                }
            }
            0xfe | 0xff => {
                // FF inc/dec use operand width; near calls/jumps/push default to 64 bits.
                let width = if opcode == 0xfe { 8 } else { bits };
                let (_, rm, group) = self.modrm(width, rex)?;
                if group <= 1 {
                    let carry = self.flags & CF;
                    let result = self.alu(
                        if group == 0 { 0 } else { 5 },
                        self.read(rm, width)?,
                        1,
                        width,
                    )?;
                    self.flags = self.flags & !CF | carry;
                    self.write(rm, result, width)?;
                } else if opcode == 0xff && [2, 4, 6].contains(&group) {
                    let value = self.read(rm, if group == 6 { stack_bits } else { 64 })?;
                    if group == 2 {
                        self.push(self.rip, 64)?;
                    }
                    if group == 6 {
                        self.push(value, stack_bits)?;
                    } else {
                        self.rip = value;
                    }
                } else {
                    return Err("Unsupported FF/FE instruction".into());
                }
            }
            0x0f => {
                let second = self.byte()?;
                match second {
                    0x05 => self.syscall()?,
                    0x1f => {
                        let (_, _, group) = self.modrm(bits, rex)?;
                        if group != 0 {
                            return Err("Invalid NOP".into());
                        }
                    }
                    0x40..=0x4f => {
                        let (reg, rm, _) = self.modrm(bits, rex)?;
                        let value = self.read(rm, bits)?;
                        if self.condition(second) {
                            self.write(reg, value, bits)?;
                        }
                    }
                    0x80..=0x8f => {
                        let offset = self.immediate(4)? as i32 as i64;
                        if self.condition(second) {
                            self.rip = self.rip.wrapping_add(offset as u64);
                        }
                    }
                    0x90..=0x9f => {
                        let (_, rm, _) = self.modrm(8, rex)?;
                        self.write(rm, u64::from(self.condition(second)), 8)?;
                    }
                    0xaf => {
                        let (reg, rm, _) = self.modrm(bits, rex)?;
                        self.multiply(reg, self.read(reg, bits)?, self.read(rm, bits)?, bits)?;
                    }
                    0xb6 | 0xb7 | 0xbe | 0xbf => {
                        let source_bits = if second & 1 == 0 { 8 } else { 16 };
                        let (reg, rm, _) = self.modrm(source_bits, rex)?;
                        // ModRM.reg is a full destination even when source is AH/CH/DH/BH.
                        let reg = match reg {
                            Operand::HighByte(i) => Operand::Register(i + 4),
                            other => other,
                        };
                        let value = self.read(rm, source_bits)?;
                        self.write(
                            reg,
                            if second & 8 != 0 {
                                signed(value, source_bits) as u64
                            } else {
                                value
                            },
                            bits,
                        )?;
                    }
                    _ => return Err(format!("Unsupported opcode 0f {second:02x}")),
                }
            }
            _ => return Err(format!("Unsupported opcode {opcode:02x}")),
        }
        Ok(())
    }
}
