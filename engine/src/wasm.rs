//! Small C ABI for the browser. No JS bindings or third-party Rust dependencies.
use crate::{elf, Cpu, Status};
use std::cell::RefCell;

#[derive(Default)]
struct Engine {
    input: Vec<u8>,
    cpu: Option<Cpu>,
    error: String,
}

thread_local! { static ENGINE: RefCell<Engine> = RefCell::new(Engine::default()); }

#[no_mangle]
pub extern "C" fn engine_input(length: u32) -> u32 {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        if length as usize > elf::MAX_ELF_BYTES {
            engine.error = "ELF file exceeds 16 MiB".into();
            return 0;
        }
        engine.input.resize(length as usize, 0);
        engine.input.as_mut_ptr() as u32
    })
}

#[no_mangle]
pub extern "C" fn engine_load(memory_mib: u32, demo: u32) -> u32 {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        let input = if demo != 0 {
            include_bytes!("../guests/demo.elf").as_slice()
        } else {
            engine.input.as_slice()
        };
        match elf::load(input, u64::from(memory_mib) * 1024 * 1024) {
            Ok(cpu) => {
                engine.cpu = Some(cpu);
                engine.error.clear();
                0
            }
            Err(error) => {
                engine.error = error;
                3
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn engine_run(budget: u32) -> u32 {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        let Some(cpu) = &mut engine.cpu else {
            engine.error = "Load a guest first".into();
            return 3;
        };
        match cpu.run(budget).clone() {
            Status::Running => 0,
            Status::Halted => 1,
            Status::Exited(_) => 2,
            Status::Fault(error) => {
                engine.error = error;
                3
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn engine_register(index: u32, high: u32) -> u32 {
    ENGINE.with(|engine| {
        let engine = engine.borrow();
        let Some(cpu) = &engine.cpu else {
            return 0;
        };
        let value = match index {
            0..=15 => cpu.registers[index as usize],
            16 => cpu.rip,
            17 => cpu.flags,
            18 => cpu.instructions,
            19 => cpu.memory.committed_bytes(),
            _ => 0,
        };
        (if high == 0 { value } else { value >> 32 }) as u32
    })
}

#[no_mangle]
pub extern "C" fn engine_exit_code() -> u32 {
    ENGINE.with(
        |engine| match engine.borrow().cpu.as_ref().map(|cpu| &cpu.status) {
            Some(Status::Exited(code)) => *code,
            _ => 0,
        },
    )
}

#[no_mangle]
pub extern "C" fn engine_output_ptr() -> u32 {
    ENGINE.with(|engine| {
        engine
            .borrow()
            .cpu
            .as_ref()
            .map_or(0, |cpu| cpu.output.as_ptr() as u32)
    })
}

#[no_mangle]
pub extern "C" fn engine_output_len() -> u32 {
    ENGINE.with(|engine| {
        engine
            .borrow()
            .cpu
            .as_ref()
            .map_or(0, |cpu| cpu.output.len() as u32)
    })
}

#[no_mangle]
pub extern "C" fn engine_clear_output() {
    ENGINE.with(|engine| {
        if let Some(cpu) = &mut engine.borrow_mut().cpu {
            cpu.output.clear();
        }
    });
}

#[no_mangle]
pub extern "C" fn engine_error_ptr() -> u32 {
    ENGINE.with(|engine| engine.borrow().error.as_ptr() as u32)
}

#[no_mangle]
pub extern "C" fn engine_error_len() -> u32 {
    ENGINE.with(|engine| engine.borrow().error.len() as u32)
}
