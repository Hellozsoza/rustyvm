//! Minimal, checked loader for freestanding static x86-64 ELF executables.
use crate::Cpu;

pub const MAX_ELF_BYTES: usize = 16 * 1024 * 1024;

fn field(data: &[u8], offset: usize, bytes: usize) -> Result<u64, String> {
    let slice = data
        .get(offset..offset.checked_add(bytes).ok_or("ELF offset overflow")?)
        .ok_or("Truncated ELF header")?;
    Ok(slice
        .iter()
        .enumerate()
        .fold(0, |value, (i, byte)| value | (u64::from(*byte) << (i * 8))))
}

pub fn load(data: &[u8], memory_size: u64) -> Result<Cpu, String> {
    if data.len() > MAX_ELF_BYTES {
        return Err("ELF file exceeds 16 MiB".into());
    }
    if data.get(..7) != Some(b"\x7fELF\x02\x01\x01") {
        return Err(
            "Expected a little-endian ELF64 executable; ISO boot is not implemented".into(),
        );
    }
    if field(data, 16, 2)? != 2 || field(data, 18, 2)? != 62 || field(data, 20, 4)? != 1 {
        return Err("Expected a static ET_EXEC x86-64 ELF (no PIE or other architectures)".into());
    }
    if field(data, 52, 2)? != 64 || field(data, 54, 2)? != 56 {
        return Err("Invalid ELF header sizes".into());
    }
    let entry = field(data, 24, 8)?;
    let phoff = usize::try_from(field(data, 32, 8)?).map_err(|_| "ELF header offset overflow")?;
    let count = field(data, 56, 2)? as usize;
    if count == 0 || count > 128 {
        return Err("Invalid ELF segment count".into());
    }
    let end = phoff.checked_add(count * 56).ok_or("ELF header overflow")?;
    if end > data.len() {
        return Err("Truncated ELF program headers".into());
    }
    let mut cpu = Cpu::new(memory_size)?;
    let mut ranges = Vec::new();
    let mut executable_entry = false;
    for index in 0..count {
        let at = phoff + index * 56;
        let kind = field(data, at, 4)?;
        if kind == 2 || kind == 3 {
            return Err("Dynamic linking and ELF interpreters are not supported".into());
        }
        if kind != 1 {
            continue;
        }
        let flags = field(data, at + 4, 4)?;
        let offset = field(data, at + 8, 8)?;
        let address = field(data, at + 16, 8)?;
        let filesz = field(data, at + 32, 8)?;
        let memsz = field(data, at + 40, 8)?;
        let align = field(data, at + 48, 8)?;
        if align > 1 && (!align.is_power_of_two() || address % align != offset % align) {
            return Err("Invalid ELF segment alignment".into());
        }
        if filesz > memsz
            || offset
                .checked_add(filesz)
                .is_none_or(|end| end > data.len() as u64)
        {
            return Err("Invalid ELF segment file range".into());
        }
        cpu.memory.check(address, memsz)?;
        let segment_end = address + memsz;
        if memsz == 0 {
            continue;
        }
        // Reserve the top 64 KiB for the initial stack.
        if segment_end > memory_size - 65536 {
            return Err("ELF segment overlaps the reserved stack".into());
        }
        if ranges
            .iter()
            .any(|&(start, end)| address < end && segment_end > start)
        {
            return Err("Overlapping ELF load segments".into());
        }
        ranges.push((address, segment_end));
        executable_entry |= flags & 1 != 0 && entry >= address && entry < segment_end;
        cpu.memory
            .write(address, &data[offset as usize..(offset + filesz) as usize])?;
    }
    if !executable_entry {
        return Err("ELF entry point is outside executable load segments".into());
    }
    cpu.rip = entry;
    cpu.registers[4] = memory_size - 32;
    cpu.memory.write(cpu.registers[4], &[0; 32])?; // argc=0, argv=NULL, envp=NULL
    Ok(cpu)
}
