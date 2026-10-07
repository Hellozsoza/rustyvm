# Rusty64 — experimental Rust x86-64 engine

Rusty64 is an original Rust interpreter compiled to `wasm32-unknown-unknown`.
It executes real x86-64 machine code in the browser. The browser integration
is available through **Rust x86-64 engine (experimental)** in the VM manager,
or the deployed site's `/rusty64/` path.

**This is an early CPU interpreter, not yet an emulator that can boot Arch Linux.**
It starts directly in a simplified long-mode environment and loads static,
freestanding ELF64 executables. An ISO, Linux kernel, or ordinary dynamically
linked Linux application will not run. Unsupported instructions and system
calls stop execution with the faulting RIP; they are not silently ignored.
The existing v86 engine still handles the manager's 16/32-bit bootable VMs.

## Implemented

- Sixteen 64-bit integer registers, RIP and arithmetic RFLAGS.
- REX prefixes; 8/16/32/64-bit operands, legacy high byte registers,
  zero-extension on 32-bit register writes, and sign/zero-extension instructions.
- Register, immediate, ModRM/SIB, RIP-relative and displacement addressing.
- MOV, LEA, XCHG register/accumulator, ADD, ADC, SUB, SBB, AND, OR, XOR, CMP,
  TEST, INC, DEC, NEG, NOT, SHL, SHR, SAR, two/three-operand IMUL.
- Conditional branches, CMOVcc, SETcc, direct/indirect near jumps and calls,
  PUSH, POP, RET, LEAVE, NOP, HLT, sign-extension helpers, CLC, STC, CLD.
- A checked ELF64 ET_EXEC loader: validates architecture, file ranges,
  segment bounds, alignment, overlaps, stack reservation, and executable entry.
  Dynamic linking and interpreters are rejected. ELF files are capped at 16 MiB.
- A small host interface using Linux x86-64 syscall numbers: `write` (stdout
  and stderr), `exit`, and `exit_group`. This is not a Linux kernel or a complete
  Linux userspace ABI. The initial stack has zero argc and null argv/envp;
  there is no auxiliary vector, TLS, filesystem, or process model.
- Sparse flat memory using `u64` guest addresses, configurable from 16 MiB
  through 8 GiB. Untouched pages read as zero. Written pages have a separate
  **128 MiB committed-memory budget**. This demonstrates access beyond 4 GiB
  without pretending a wasm32 instance can allocate an 8 GiB linear memory.
- Cooperative instruction budgets and a Web Worker; Run, Pause, Step, Reset,
  terminal output, executed-instruction count and register inspection.

The decoder implements the listed encodings, not the entire instruction set.
Memory is flat: ELF permissions are validated for the entry point but not
enforced on subsequent accesses. There is no MMU, page protection or ring model.
This build has no VM snapshot, image attachment or cookie integration of its
own. Loaded ELF files remain in memory until the page is closed or reloaded.

## Build from source

Required: Rust 1.90.0, its `wasm32-unknown-unknown` target, GNU `as` and `ld`
with x86-64 support, and Node.js 24. The crate has no third-party dependencies.

```sh
cd engine
rustup toolchain install 1.90.0 --profile minimal --component rustfmt,clippy --target wasm32-unknown-unknown
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
node build.mjs
cd ../browser
npm ci
npm run build
npm test
npm run test:browser
```

`node build.mjs` assembles `guests/demo.S` into an actual x86-64 ELF with GNU
binutils, then compiles the Rust engine to WASM. It copies the resulting
`rusty64.wasm` and `demo.elf` into `browser/app/rusty64/` and records artifact
and source SHA-256 checksums in `manifest.json`. `npm run build:engine` from
`browser/` invokes the same script.

The binary artifacts are committed so Vercel's **Other** preset and `browser`
root directory continue to work with Node alone. The normal browser build
verifies artifact checksums, and also verifies source checksums when the
`engine/` directory is available. After changing Rust source, rebuild the
engine before building or committing the frontend. Checksums detect stale or
modified files; they are not a third-party signature or code audit.

## Validation

Native Rust tests cover registers and widths, extended addressing, arithmetic
flags, branches, calls, shifts, memory bounds, malformed ELF files, invalid
instructions, syscall limits, and bounded infinite loops. On x86-64 hosts,
2,304 arithmetic cases compare results and flags against the physical CPU
using Rust inline assembly. The compiled demo tests arithmetic values and
memory addresses above 4 GiB and exits with code 1 if its checks fail.

Chromium tests execute the **compiled WASM engine**, exercise that demo, upload
an ELF, check invalid inputs and exact fault locations, and pause/reset an
infinite guest. Existing v86 browser tests also run to check the integration.

## Work still required for official Arch Linux

1. Complete architectural CPU behavior: reset/real/protected/long-mode
   transitions, control registers and MSRs, CPUID, exceptions, descriptor
   tables, paging and TLB behavior, privilege transitions, interrupt delivery,
   and the instruction extensions needed by the kernel and userspace.
2. Implement a PC platform: firmware/boot protocol, timers, interrupt
   controllers, PCI, boot storage, keyboard and display, and ACPI tables.
3. Validate the platform against architectural test suites, then boot a Linux
   kernel and initramfs, followed by the Arch installation media. Add persistent
   writable disks and device support before claiming installation works.
4. Improve execution speed, likely with a JIT, for usable OS performance.

These are substantial remaining engine projects. The current demo verifies
64-bit instruction execution; it does not verify OS compatibility.

License: GPL-3.0-only; see `../browser/licenses/VirtualBox-COPYING`.
