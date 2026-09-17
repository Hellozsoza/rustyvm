# Rust + WebAssembly Rewrite Plan for Oracle VirtualBox

> **Branch:** `main` (empty working tree — source code lives on `virtualbox` branch)
> **Goal:** Rewrite VirtualBox to run purely in a browser via WebAssembly, compiled from Rust.
> **Scope:** The entire `src/` tree on the `virtualbox` branch (38,019 C/C++ files, ~17.2M lines).
> **Author:** Auto-generated from source analysis on 2026-09-16.

---

## Table of Contents

- [Current State Assessment](#current-state-assessment)
- [Source Inventory by Subsystem](#source-inventory-by-subsystem)
- [Browser / WebAssembly Constraints](#browser--webassembly-constraints)
- [Architecture: Target Design](#architecture-target-design)
- [Phase 0 — Foundation (Months 1–3)](#phase-0--foundation-months-13)
- [Phase 1 — Core Virtualization Engine (Months 3–8)](#phase-1--core-virtualization-engine-months-38)
- [Phase 2 — Device Emulation (Months 6–14)](#phase-2--device-emulation-months-614)
- [Phase 3 — Storage & Network (Months 10–16)](#phase-3--storage--network-months-1016)
- [Phase 4 — Main / VM Management & Services (Months 13–19)](#phase-4--main--vm-management--services-months-1319)
- [Phase 5 — Frontend / UI & User Experience (Months 16–22)](#phase-5--frontend--ui--user-experience-months-1622)
- [Phase 6 — Integration, Optimization & Hardening (Months 20–24)](#phase-6--integration-optimization--hardening-months-2024)
- [Technical Risk Register](#technical-risk-register)
- [Dependencies & Tooling](#dependencies--tooling)
- [Milestones & Acceptance Criteria](#milestones--acceptance-criteria)

---

## Current State Assessment

### Repository Overview

| Metric | Value |
|--------|-------|
| Branch | `virtualbox` (full), `main` (empty — all C code deleted) |
| Source files | 38,019 C/C++ files |
| Total lines of code | ~17,240,823 (src/ + include/) |
| `src/VBox` directory | 897 MB |
| `src/libs` directory | 326 MB |
| Build system | `Makefile.kmk` / `Config.kmk` / `configure.py` (kBuild) |
| Language | C (Runtime/IPRT), C++ (VMM, Devices, Main, Frontends) |
| License | GPL-3.0-only (base platform) |

### Key Facts

1. The `main` branch contains a single commit `189dec0a5e6` titled "Deleted C code, to not be in the way while writing Rust" which removed **all 66,011 files** from the repository. The branch is now empty and ready for Rust source code.
2. The `virtualbox` branch is the complete Oracle VirtualBox 7.x source tree with upstream commits up to 2026-08-19.
3. IPRT (Runtime) is the cross-platform abstraction layer — 1,248 `.cpp` files, ~565K lines — and is the most portable part of the codebase.
4. VMM (Virtual Machine Monitor) is 65 MB with the instruction emulator (IEM) tables alone consuming 249K lines for ARMv8 and 20K for x86.
5. The Qt-based Frontends (103 MB) are not needed for a browser-only version but the settings/data model (~20 KB of header logic) may be reusable.
6. HostDrivers (5.3 MB) contain kernel-mode network/USB drivers — **completely irrelevant** for a browser build and will not be ported.
7. Additions/3D (327 MB) contains a full Mesa 3D graphics stack — **not ported**; 3D acceleration in the browser will use WebGPU.

---

## Source Inventory by Subsystem

### `src/VBox/VMM/` — Virtual Machine Monitor (65 MB)

| Component | Size | Key Files | Line Counts (Top) |
|-----------|------|-----------|-------------------|
| VMMAll (core tables) | — | `IEMAll*.cpp`, `CPUMAll*.cpp`, `PGMAll*.cpp`, `TMAll.cpp` | 249,663 (ARM64 IEM tables), 100,215 (ARM64 sys regs), 20,979 (x86 IEM), 10,545 (IEM recompiler), 6,590 (CPUM msrs), 5,849 (PGM phys) |
| VMMR3 (ring-3, host user context) | — | `EMR3.cpp`, `VMR3.cpp`, `PDMR3*.cpp`, `PGMR3*.cpp`, `SSMR3.cpp`, `TMR3.cpp`, `CPUMR3*.cpp` | 2,855 (EMR3), 4,480 (VMR3), 4,442 (TMR3), 4,311 (CPUMR3CpuId-x86), 6,400 (PGMR3Phys) |
| VMMR0 (ring-0, host kernel context) | — | `HMR0VMX-x86.cpp`, `HMR0SVM-x86.cpp`, `GMMR0.cpp`, `PDMR0*.cpp` | 9,707 (HMR0SVM), 7,197 (HMR0VMX), 5,750 (GMMR0) |
| Backends | — | `NEMR3Native-win-x86.cpp`, `NEMR3Native-darwin-x86.cpp`, `GIMR3Kvm.cpp`, `GIMR3Hv.cpp`, `GIMR3Minimal.cpp` | 7,266 (NEM-win), 4,635 (NEM-darwin), 1,495 (GIM HV) |
| CPU targets | — | `target-x86/`, `target-armv8/` | 101,07 lines (pdmdev.h), 4,718 (hm_vmx.h) |

**Key observations:**

- **EMR3.cpp** (2,855 lines) contains the main VM execution loop `EMR3ExecuteVM()` and dispatches to three execution backends:
  - **HM** (Hardware-assisted): VT-x on x86, SVM on ARM — calls host CPU virtualization extensions directly
  - **NEM** (Native Execution Mode): Direct host execution with hypervisor intervention — OS-specific (Windows, macOS, Linux/KVM)
  - **IEM** (Interpreter/Emulator): Software fallback; includes a full instruction decoder/executor with massive lookup tables
- **PDM (Pluggable Device Manager)** is the central device abstraction. Devices register via `PCPDMDEVHLPR3` callback tables in `pdmdev.h` (10,107 lines of header). Drivers register via `PCPDMDRVREG` in `pdmdrv.h` (2,502 lines). This is the primary interface to port to Rust traits.
- **Guest RAM (PGM)** uses region-based mapping (`PGMRAMRANGE`, `PGMCHUNKR3MAP`) with shadow paging for nested virtualization. In a browser, guest physical memory becomes a linear `Vec<u8>` with page-table emulation.
- **IEM instruction tables** are auto-generated data (249K lines for ARMv8). These can be regenerated from LLVM/objdump or rewritten as Rust const tables — they are pure data, not logic.
- **IOM** (IO/MEM Manager) handles MMIO and PIO registration — maps device regions to callbacks. In WASM, these become WebAssembly memory accesses.

### `src/VBox/Devices/` — Device Emulation (281 MB)

| Device Category | Files | Top Files by Lines |
|-----------------|-------|--------------------|
| Graphics (VGA/SVGA/3D) | 12 | `DevVGA-SVGA3d-dx-dx11.cpp` (14,767), `DevE1000.cpp` (9,784), `DevVGA-SVGA.cpp` (9,140), `DevVGA-SVGA-cmd.cpp` (9,134), `DevVGA-SVGA3d-ogl.cpp` (7,975), `DevVGA.cpp` (7,945) |
| Network | 10 | `DevE1000.cpp` (9,784), `DevPCNet.cpp` (5,445), `DevDP8390.cpp` (5,535), `SrvIntNetR0.cpp` (6,910), `DevVirtioNet*.cpp` |
| Storage | 8+ | `DevNVMe.cpp` (7,702), `DevATA.cpp` (8,493), `DevAHCI.cpp` (6,184), `DrvVD.cpp` (5,609), `DevLsiLogicSCSI.cpp` (5,528) |
| USB | 3 | `DevXHCI.cpp` (8,378), `DevEHCI.cpp` (5,245), `DevOHCI.cpp` (6,247) |
| Audio | 4+ | `DevHda.cpp` (5,469), `DrvAudio.cpp` (5,052), `DrvHostAudioWasApi.cpp` (3,588), `DrvHostAudioCoreAudio.cpp` (2,934) |
| IOMMU | 2 | `DevIommuAmd.cpp` (7,370), `DevIommuIntel.cpp` |
| VMMDev | 1 | `VMMDev.cpp` (5,467) |
| Bus/Chipset | 3 | `DevIommuAmd.cpp`, `DevPIIX3.cpp`, `DevICH9.cpp` |
| Input | 3 | PS/2 keyboard/mouse, tablet |
| Misc | 10+ | TPM, SMBIOS, ACPI, watchdog, etc. |

**Key observations:**

- **DevVGA-SVGA3d-ogl.cpp** and **DevVGA-SVGA3d-dx-dx11.cpp** use OpenGL and DirectX 11 respectively. These **cannot** be ported directly — must be rewritten to use WebGL/WebGPU via `wasm-bindgen` or `wgpu`.
- **DrvAudio** uses host audio backends (WasApi, CoreAudio, DSound, PulseAudio, Alsa) — all must be replaced with the WebAudio API.
- **Network devices** (E1000, PCNet, DP8390, virtio-net) are pure Ethernet frame processing — good candidates for porting. They currently use ring buffers and descriptor chains that map cleanly to Rust structs.
- **USB controllers** (XHCI, EHCI, OHCI) are complex state machines — significant but well-defined. USB device passthrough is impossible in a browser; virtual USB devices must be emulated entirely.
- **DevVMMDev** provides the VMMDev device (guest-host communication) — this becomes the primary host-guest IPC channel in the browser version.

### `src/VBox/Runtime/` (IPRT) — Cross-Platform Abstraction (44 MB)

| Subdirectory | Purpose | Files |
|-------------|---------|-------|
| `common/` | Core library: string, alloc, file, fs, log, crypto, net, VFS, zip, etc. | ~1,100 `.cpp` files, ~500K lines |
| `r3/` | Ring-3 OS-specific implementations | ~150 `.cpp` files |
| `r0drv/` | Ring-0 driver abstraction | ~20 `.cpp`/`.c` files |
| `gc/` | Garbage collector (Boehm-derived) | — |
| `generic/` | Generic implementations used across OSes | ~20 files |
| `include/` | IPRT public headers | ~30 `.h` files |
| `tools/` | RTSignTool, other utilities | ~10 files |
| `VBox/` | VBox-specific runtime glue | — |

**Key observations:**

- IPRT is the most portable part of the codebase and the **best candidate for first port to Rust**. It provides: `RTThread`, `RTFile`, `RTPath`, `RTSocket`, `RTMem`, `RTStr`, `RTCrypt`, `RTRand`, etc.
- `r0drv/` (ring-0 driver layer) contains OS-specific kernel code (linux, darwin, nt, solaris, freebsd, haiku, os2) — **completely irrelevant for browser/WASM**. The ring-0 path will be replaced with a "virtual ring-0" that runs entirely in ring-3 (WASM has no ring distinction).
- `r3/posix/` and `r3/win/` contain OS-specific implementations — both can be replaced with WASM-specific implementations using `web-sys`/`wasm-bindgen`.
- The `common/string/unidata-*.cpp` files (47K lines) are Unicode data tables — pure data, not logic. Can be regenerated as Rust `const` arrays or loaded from external data files.
- The `common/fs/` subdirectory implements virtual filesystems (VFS) including FAT, NTFS, ISO9660, UDF — these are already pure logic and portable to Rust.

### `src/VBox/Main/` — VM Management / XPCOM COM Layer (20 MB)

| Component | Key Files | Lines |
|-----------|-----------|-------|
| Server-side (VirtualBox object) | `MachineImpl.cpp` (16,751), `VirtualBoxImpl.cpp` (7,000), `MediumImpl.cpp` (11,076) | 30,934 total |
| Client-side (Console) | `ConsoleImpl.cpp` (12,437), `GuestSessionImpl.cpp` (5,999), `ClipboardImpl.cpp` (4,424) | — |
| XML settings | `Settings.cpp` (10,559) | — |
| Web service | `webservice/` | — |

**Key observations:**

- Main is built on XPCOM (Cross-Platform Component Object Model) — a COM-like binary interface standard. In a browser context, this maps naturally to JavaScript `Promise`-based APIs exposed via `wasm-bindgen`.
- The `MachineImpl.cpp` (16,751 lines) is the central VM lifecycle manager — it creates, starts, stops, and configures VMs. This becomes the `VirtualMachine` Rust struct exposed to JavaScript.
- Settings management uses XML files — can be replaced with a Rust-native serialization format (e.g., `serde` + JSON/YAML).

### `src/VBox/Storage/` — Disk Image Formats (2.6 MB)

| Format | File | Lines |
|--------|------|-------|
| VirtualBox Disk | `VD.cpp` (9,684) | 65,892 total |
| VMDK | `VMDK.cpp` (9,202) | — |
| ISCSI | `ISCSI.cpp` (5,567) | — |
| VDI | `VDI.cpp` (3,290) | — |
| VHD | `VHD.cpp` (3,193) | — |
| QCOW | `QCOW.cpp` (2,563) | — |
| VHDX | `VHDX.cpp` (2,399) | — |
| DMG | `DMG.cpp` (2,393) | — |
| QED | `QED.cpp` (2,390) | — |

**Key observations:**

- Storage is **excellent for browser porting** — disk image parsing is pure logic, no hardware dependencies. All formats can be read from `ArrayBuffer`/`SharedArrayBuffer` in WASM.
- The VD (VirtualBox Disk) format is the native format and will be the primary target.
- VFS backends (`fatvfs.cpp`, `ntfsvfs.cpp`, `isovfs.cpp`, `udfhlp.cpp`) can be rewritten as Rust filesystem abstractions over virtual disk images.

### `src/VBox/NetworkServices/` — Network Services (876 KB)

| Service | File | Lines |
|---------|------|-------|
| NAT (SLIRP) | `VBoxNetSlirpNAT.cpp` (3,112) | 19,371 total |
| Internal Network Switch | `VBoxIntNetSwitch.cpp` (1,735) | — |
| DHCP | `Dhcpd/` | — |

**Key observations:**

- NAT networking uses SLIRP (a user-mode network stack) — already pure C logic, portable to Rust.
- Internal network switch is simple frame forwarding — easy to port.
- In a browser context, all networking must go through the browser's network stack (fetch/WebSocket). A virtual NAT can be implemented by proxying guest traffic to the browser's fetch API.

### `src/VBox/HostServices/` — Guest Services (1.4 MB)

| Service | Key Files |
|---------|-----------|
| Shared Folders | `vbsf.cpp` (2,803), `VBoxSharedFoldersSvc.cpp` (1,976) |
| Guest Control | `VBoxGuestControlSvc.cpp` (2,644) |
| Shared Clipboard | `VBoxSharedClipboardSvc*.cpp` |
| Drag & Drop | `VBoxDragAndDropSvc.cpp` (1,277) |
| Guest Properties | `VBoxGuestPropSvc.cpp` (1,900) |

**Key observations:**

- Shared Folders currently use host filesystem access — in the browser, these map to browser `FileSystem Access API` or downloaded/uploaded files.
- Clipboard services map to the browser Clipboard API.
- Guest Properties is a simple key-value store — easy to port.

### Not Ported (Excluded from Scope)

| Directory | Size | Reason |
|-----------|------|--------|
| `src/VBox/HostDrivers/` | 5.3 MB | Kernel-mode drivers (network, USB) — no equivalent in browser |
| `src/VBox/Additions/` | 327 MB | Guest additions including Mesa 3D, Windows/Linux guest drivers — not relevant |
| `src/VBox/Frontends/` (Qt GUI) | 103 MB | Native Qt GUI — replaced by browser UI (see Phase 5) |
| `src/VBox/ImageMounter/` | 140 MB | Host-side image mounting — not applicable |
| `src/VBox/RDP/` | 6.8 MB | Remote Desktop Protocol server — browser is the display, no RDP server needed |
| `src/VBox/ValidationKit/` | 19 MB | Testing framework — replaced by Rust test infrastructure |

---

## Browser / WebAssembly Constraints

### Hard Constraints

1. **No direct hardware access**: VT-x, AMD-V, PCI, USB controllers, etc. are all unavailable. All hardware must be emulated in software.
2. **No native threads in the traditional sense**: WebAssembly threads (`SharedArrayBuffer` + `Atomics`) exist but are limited and may not be available in all browsers. Thread synchronization must use `Atomics.wait`/`Atomics.notify`.
3. **No filesystem directly**: Use the browser's IndexedDB (via `idb-keyval` or similar) or the Emscripten File System API. A virtual filesystem layer maps to browser storage.
4. **No sockets**: Use WebSocket for networking, fetch API for HTTP.
5. **Graphics must use WebGL or WebGPU**: OpenGL and DirectX calls are impossible. The VGA device must render to a canvas via WebGL/WebGPU textures.
6. **Audio must use WebAudio API**: No direct audio device access.
7. **No system calls**: WASM modules cannot make native syscalls. All OS interactions go through the WASM host (browser JavaScript).
8. **Memory is linear**: WASM linear memory is a single `ArrayBuffer`. Guest RAM must fit within this. Current 64-bit address space is unrealistic; a 2GB-4GB guest RAM ceiling is practical.
9. **No JIT compilation of host code**: WASM JIT is handled by the browser. The IEM interpreter/recompiler must be rewritten in Rust and compiled to WASM.
10. **WASM linear memory growth**: Growing memory is expensive in WASM. Pre-allocate guest RAM at VM start.

### Soft Constraints (Design Decisions)

1. **3D acceleration**: Use WebGPU (via `wgpu` crate) instead of OpenGL/DirectX. Feature level will be lower than native.
2. **SMP (multi-CPU)**: Use WebAssembly threads with shared memory if available; otherwise emulate SMP with a single thread and time-slicing.
3. **Snapshot/Save state**: Serialize VM state to `ArrayBuffer` and store in IndexedDB or download as a file.
4. **Clipboard**: Use the browser Clipboard API via `wasm-bindgen`.
5. **Drag and Drop**: Use HTML5 drag-and-drop events.
6. **Shared Folders**: Use the browser FileSystem Access API or upload/download.

---

## Architecture: Target Design

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Browser Environment                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────┐ │
│  │  HTML5   │  │ WebGL2/  │  │ WebAudio │  │  Indexed│ │
│  │  Canvas  │  │ WebGPU   │  │  API     │  │  DB     │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬────┘ │
│       │             │             │              │       │
│  ┌────▼─────────────▼─────────────▼──────────────▼────┐ │
│  │              wasm-bindgen / web-sys                 │ │
│  └────────────────────┬──────────────────────────────┘ │
│                       │                                │
│  ┌────────────────────▼──────────────────────────────┐ │
│  │            WASM Module (Rust)                      │ │
│  │                                                    │ │
│  │  ┌──────────────────────────────────────────────┐  │ │
│  │  │         VMM Layer (Rust)                      │  │ │
│  │  │  ┌─────────┐ ┌─────────┐ ┌────────────────┐  │  │ │
│  │  │  │  EM     │ │  PDM    │ │   IEM          │  │  │ │
│  │  │  │  (Exec  │ │ (Device │ │ (Interpreter/  │  │  │ │
│  │  │  │  Mgr)   │ │  Mgr)   │ │   Recompiler)  │  │  │ │
│  │  │  └────┬────┘ └────┬────┘ └───────┬────────┘  │  │ │
│  │  └───────┼───────────┼──────────────┼───────────┘  │ │
│  │  ┌───────▼───────────▼──────────────▼───────────┐  │ │
│  │  │        Memory Management (PGM)               │  │ │
│  │  │   Linear guest RAM in WASM linear memory     │  │ │
│  │  └──────────────────────────────────────────────┘  │ │
│  │                                                    │ │
│  │  ┌──────────────────────────────────────────────┐  │ │
│  │  │        Device Emulation (Rust)                │ │
│  │  │  VGA/SVGA, Network, Storage, USB, Audio, etc.│ │
│  │  └──────────────────────────────────────────────┘  │ │
│  │                                                    │ │
│  │  ┌──────────────────────────────────────────────┐  │ │
│  │  │        IPRT/Runtime (Rust)                    │ │
│  │  │  Threads, Files, Sockets, Memory, Strings     │ │
│  │  └──────────────────────────────────────────────┘  │ │
│  │                                                    │ │
│  │  ┌──────────────────────────────────────────────┐  │ │
│  │  │        Main / COM Layer (Rust)                │ │
│  │  │  VirtualMachine, Console, Medium, Settings    │ │
│  │  └──────────────────────────────────────────────┘  │ │
│  │                                                    │ │
│  └────────────────────────────────────────────────────┘ │
│                       │                                │
│  ┌────────────────────▼──────────────────────────────┐ │
│  │              Browser JavaScript Glue               │ │
│  │  Event loop, Canvas, Audio context, File access   │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Execution Engine**: Replace HM/NEM/IEM with a single **WASM-native execution loop**. Use `wgpu` for any hardware-assisted acceleration concept. The interpreter (IEM) becomes the primary execution path in browser; a recompiler (JIT) could use `cranelift` or `wasmer` but is unlikely to match native speed.

2. **PDM → Rust Traits**: The `PCPDMDEVHLPR3` callback table becomes a set of Rust traits (`Device`, `Driver`, `Bridge`). Device registration becomes `impl Device for MyDevice {}`.

3. **Memory Model**: Guest physical memory is a `&[u8]` slice into WASM linear memory. Guest virtual → physical translation uses a software page walker (replacing N.P. / shadow paging).

4. **Device Backend Abstraction**: Each device has a `Backend` trait that abstracts host resources:
   - `GraphicsBackend`: Canvas/WebGL/WebGPU
   - `AudioBackend`: WebAudio
   - `NetworkBackend`: WebSocket/fetch
   - `StorageBackend`: IndexedDB/ArrayBuffer
   - `ClipboardBackend`: Clipboard API

5. **Async I/O**: Device I/O becomes `async` Rust with browser-native async (Promises via `wasm-bindgen-futures`).

---

## Phase 0 — Foundation (Months 1–3)

### Objective: Establish the Rust + WASM build infrastructure and port the Runtime (IPRT) layer.

### Deliverables

1. **Build System Setup**
   - Create `Cargo.toml` workspace with the following crates:
     - `vbox-core` — Core types, macros, status codes
     - `vbox-runtime` — IPRT port (threading, memory, files, strings, crypto)
     - `vbox-vmm` — Virtual Machine Monitor core
     - `vbox-devices` — Device emulation
     - `vbox-storage` — Disk image formats
     - `vbox-main` — VM management / COM layer
     - `vbox-web` — Browser glue (wasm-bindgen, web-sys)
     - `vbox-wasi` — Optional WASI compatibility layer
   - Set up `wasm-pack` / `cargo build --target wasm32-unknown-unknown`
   - Create `index.html` + JavaScript glue for initial browser boot

2. **IPRT Port — Memory & Threading** (`vbox-runtime`)
   - Port `RTMem` (allocation, page management) → `std::alloc` / custom allocator
   - Port `RTThread` (thread creation, semaphores, mutexes, condition variables) → `std::thread` + `std::sync` (with `Atomics` fallback for WASM threads)
   - Port `RTSem` (semaphores, mutexes, critical sections) → `std::sync::Mutex`, `std::sync::Semaphore` (or `parking_lot`)
   - Port `RTTimer` → `std::time::Instant` + browser `requestAnimationFrame`
   - Port `RTR0` ring-0 abstraction → No-op "virtual ring-0" (all code runs in ring-3 in WASM)

3. **IPRT Port — Strings & Data Structures** (`vbox-runtime`)
   - Port `RTStr` (string handling) → `String` / `Cow<str>` with custom UTF-16 support
   - Port `RTList`, `RTChain`, `RTPtrStack` → `linked-hash-map`, `smallvec`, or custom `Vec`-based structures
   - Port `RTUuid`, `RTNetAddr`, `RTDate`, `RTTime` → `uuid`, `std::net`, `chrono`
   - Port `RTFile`, `RTPath`, `RTDir` → browser `FileSystem Access API` via `wasm-bindgen`

4. **IPRT Port — Sockets & Networking** (`vbox-runtime`)
   - Port `RTSocket` → `WebSocket` + `fetch` via `wasm-bindgen` futures
   - Port `RTHttp` → `reqwest` (wasm-bindgen version) or native `fetch`
   - Implement virtual socket abstraction that maps guest TCP/UDP to browser WebSocket

5. **IPRT Port — Crypto & Random** (`vbox-runtime`)
   - Port `RTCrypt` → `ring` or `openssl` (wasm-bindgen version)
   - Port `RTRand` → `js-rand` / browser `crypto.getRandomValues()`

6. **IPRT Port — File Systems** (`vbox-runtime`)
   - Port `fatvfs.cpp`, `ntfsvfs.cpp`, `isovfs.cpp`, `udfhlp.cpp` → Rust VFS implementations
   - Map to browser storage (IndexedDB via `idb-keyval` or Emscripten FS)

7. **Core Types & Macros** (`vbox-core`)
   - Port `VBox/types.h` → Rust types (`u32`, `u64`, `bool`, enums)
   - Port `VBox/err.h` → Rust `Error` / `Result` types
   - Port `VBox/com` → XPCOM stub (see Phase 4)
   - Port `VBox/log.h` → `log` crate with browser console fallback
   - Port `VBox/AssertGuest.h` → Debug assertions

8. **Tests**
   - Unit tests for each IPRT module
   - Integration test: allocate memory, create threads, run a simple loop
   - Benchmark: compare IPRT operations to original C++

### Milestone: "Can run a minimal Rust program in the browser using the virtual runtime"

---

## Phase 1 — Core Virtualization Engine (Months 3–8)

### Objective: Port the VMM (Virtual Machine Monitor) including the execution manager, CPU emulation, memory management, and the IEM interpreter.

### Deliverables

1. **Execution Manager (EM) (`vbox-vmm`)**
   - Port `EMR3.cpp` → `em.rs`: The VM execution loop
   - Replace `EMR3ExecuteVM()` with an `async fn execute_vm()` that drives the VM loop
   - Implement the `EMSTATE` state machine as a Rust `enum` with `match` dispatch
   - Port `emR3RawExecute`, `emR3HmExecute`, `emR3RmExecute`, `emR3IemExecute` → Single `execute_once()` function
   - **Key design**: In a browser, HM (hardware-assisted) is unavailable. IEM (interpreter) is the primary path. A recompiler (`IEM recompiler`) can be added later using `cranelift` or `wasmtime` embedded.

2. **CPU Emulation (CPUM) (`vbox-vmm`)**
   - Port `CPUMR3.cpp`, `CPUMR3CpuId*.cpp` → `cpu.rs`, `cpuid.rs`
   - Implement CPUID database as `const` arrays (Rust `&[CpuIdEntry]`)
   - Port x86/AMD64 CPU state (CPUMCTX) → Rust structs with `#[repr(C)]` for WASM compatibility
   - Implement guest register file as `struct CpuRegisters { rax: u64, ..., rflags: u64 }`
   - Port `CPUMR3Db` (CPU database) → Static `&[u8]` or generated Rust module

3. **Interpreter/Emulator (IEM) (`vbox-vmm`)**
   - Port `IEMAll*.cpp` → `iem.rs`, `iem_tables.rs`
   - The IEM instruction tables (249K lines for ARMv8, 20K for x86) are pure data — generate as Rust `const` arrays using a build script (`build.rs`) from existing data files or by rewriting the table generators
   - Port the instruction decoder → `decoder.rs` (x86/AMD64 and ARMv8)
   - Port `IEMAllExec` → `iem::execute()` — the core instruction dispatch loop
   - **Key design**: Use a `match` on opcode or a computed jump table (via array indexing) for fast dispatch
   - For x86: Port `IEMAllAImplC-x86.cpp` (20,979 lines) as the reference implementation
   - For ARMv8: Port `IEMAllIntprA64Tables-armv8.cpp` (249,663 lines) — these are instruction implementations, not tables

4. **Memory Management (PGM) (`vbox-vmm`)**
   - Port `PGMR3Phys.cpp`, `PGMAllPhys.cpp`, `PGMAll*.cpp` → `pgm.rs`
   - Guest physical memory: Map to a `Box<[u8]>` or `Vec<u8>` in WASM linear memory
   - Page table: Implement as a multi-level page table structure (4-level for x86-64, 3-level for ARMv8)
   - Replace shadow paging with a software TLB emulator
   - Port `PGMR3Heap.cpp` → `pgm_heap.rs`
   - Handle MMIO regions: Map device MMIO ranges to `Backend` trait calls

5. **Timer Manager (TM) (`vbox-vmm`)**
   - Port `TMR3.cpp` (4,442 lines), `TMAll.cpp` (2,890 lines) → `tm.rs`
   - Replace host timer APIs with browser `requestAnimationFrame` + `setTimeout`
   - Implement virtual TSC (Time Stamp Counter) using `RDTSC` → `Performance.now()` via `wasm-bindgen`

6. **Debugger (DBGFR3) (`vbox-vmm`)**
   - Port `DBGFR3.cpp` (5,130+ lines), `DBGFR3Mem.cpp`, `DBGFR3Bp.cpp` → `dbg.rs`
   - Browser debugger UI will be built in Phase 5; the engine provides the API

7. **Saved State (SSM) (`vbox-vmm`)**
   - Port `SSMR3.cpp` (10,237 lines), `SSMAll.cpp` → `ssm.rs`
   - Implement serialization/deserialization of VM state using `bincode` or custom binary format
   - Save/load VM state to/from IndexedDB or browser download

8. **Module Loader (VMMR3Ldr)**
   - Port `PDMR3Ldr.cpp` → Module loading from virtual disk images
   - BIOS/UEFI: Load from disk image blob, not from filesystem

### Milestone: "Can interpret and execute a simple x86 guest program in the browser"

---

## Phase 2 — Device Emulation (Months 6–14)

### Objective: Port all emulated devices to Rust, replacing the PDM callback architecture with Rust traits.

### Deliverables

1. **PDM Rewrite (`vbox-devices`)**
   - Define Rust traits: `Device`, `Driver`, `Bridge`, `DeviceHelper`, `DriverHelper`
   - `Device` trait: `fn attach(&mut self, cfg: &Config) -> Result<()>`, `fn detach(&mut self)`, `fn query_interface(&self, iid: &Uuid) -> Option<&dyn Any>`
   - `Driver` trait: `fn init(&mut self, device: &dyn Device) -> Result<()>`, `fn power_off(&mut self)`, `fn reset(&mut self)`
   - Replace `PCPDMDEVHLPR3` callback table with trait methods
   - Implement a `DeviceManager` struct that registers and enumerates devices
   - Replace `PDMR3Queue` with `tokio::sync::mpsc` or `flume` channels

2. **Graphics (`vbox-devices/graphics`)**
   - **DevVGA** (`DevVGA.cpp`, 7,945 lines) → `devices/graphics/vga.rs`
   - **DevVGA-SVGA** (`DevVGA-SVGA.cpp`, 9,140 + `DevVGA-SVGA-cmd.cpp`, 9,134) → `devices/graphics/svga.rs`
   - **DevVGA-SVGA3d** (OpenGL/DirectX versions) → `devices/graphics/svga3d.rs`
     - Replace OpenGL with **WebGPU** via `wgpu` crate
     - Replace DirectX 11 with **WebGPU** via `wgpu` crate
     - Render to an HTML5 `<canvas>` via `web-sys` `HtmlCanvasElement`
     - Implement a `GraphicsBackend` trait with `Canvas` and `WebGPU` implementations
   - **3D Acceleration**: Map 3D commands from guest to WebGPU compute/shader pipelines
   - **Key design**: The VGA device writes to a framebuffer in guest RAM. The browser reads this framebuffer via WASM linear memory and renders it to the canvas each frame.

3. **Network (`vbox-devices/network`)**
   - **DevE1000** (`DevE1000.cpp`, 9,784 lines) → `devices/network/e1000.rs`
   - **DevPCNet** (`DevPCNet.cpp`, 5,445 lines) → `devices/network/pcnet.rs`
   - **DevDP8390** (`DevDP8390.cpp`, 5,535 lines) → `devices/network/dp8390.rs`
   - **virtio-net** variants → `devices/network/virtio_net.rs`
   - **SrvIntNetR0** (`SrvIntNetR0.cpp`, 6,910 lines) → `devices/network/intnet.rs`
   - Implement `NetworkBackend` trait: `fn send_packet(&mut self, packet: &[u8]) -> Result<()>`, `fn recv_packet(&mut self) -> Result<Option<Vec<u8>>>`
   - Map to browser WebSocket (for TAP-like bridging) or `fetch` + WebSocket (for NAT)
   - **Ring buffers**: Replace C ring buffers with `crossbeam::ringbuf` or custom `VecDeque`

4. **Storage (`vbox-devices/storage`)**
   - **DevATA** (`DevATA.cpp`, 8,493 lines) → `devices/storage/ata.rs`
   - **DevAHCI** (`DevAHCI.cpp`, 6,184 lines) → `devices/storage/ahci.rs`
   - **DevNVMe** (`DevNVMe.cpp`, 7,702 lines) → `devices/storage/nvme.rs`
   - **DrvVD** (`DrvVD.cpp`, 5,609 lines) → `devices/storage/vd_driver.rs`
   - Implement `StorageBackend` trait: `fn read_sectors(&mut self, lba: u64, count: u32) -> Result<Vec<u8>>`, `fn write_sectors(&mut self, lba: u64, count: u32, data: &[u8]) -> Result<()>`
   - Map to browser `IndexedDB` or `File` API for persistent storage
   - **Key design**: Guest issues a read/write command → device sends to VD driver → VD driver reads from virtual disk image → disk image is an `ArrayBuffer` from the browser

5. **USB (`vbox-devices/usb`)**
   - **DevXHCI** (`DevXHCI.cpp`, 8,378 lines) → `devices/usb/xhci.rs`
   - **DevEHCI** (`DevEHCI.cpp`, 5,245 lines) → `devices/usb/ehci.rs`
   - **DevOHCI** (`DevOHCI.cpp`, 6,247 lines) → `devices/usb/ohci.rs`
   - **DevVUSBRootHub** (`DrvVUSBRootHub.cpp`, 1,939 lines) → `devices/usb/root_hub.rs`
   - **Key design**: No physical USB devices in browser. Virtual USB devices can be emulated (e.g., a virtual USB flash drive mapped to browser storage).
   - USB descriptors and configuration become static Rust data structures.

6. **Audio (`vbox-devices/audio`)**
   - **DevHda** (`DevHda.cpp`, 5,469 lines) → `devices/audio/hda.rs`
   - **DrvAudio** (`DrvAudio.cpp`, 5,052 lines) → `devices/audio/drv_audio.rs`
   - Replace host audio backends (WasApi, CoreAudio, DSound, PulseAudio, Alsa) with `WebAudio API` via `wasm-bindgen`
   - Implement `AudioBackend` trait: `fn play(&mut self, samples: &[f32])`, `fn capture(&mut self) -> Result<Vec<f32>>`
   - Use `AudioContext` from WebAudio API

7. **VMMDev (`vbox-devices/vmmdev`)**
   - **VMMDev** (`VMMDev.cpp`, 5,467 lines) → `devices/vmmdev.rs`
   - This is the guest-host communication device — becomes the primary IPC channel
   - Map to browser events (custom DOM events or `postMessage`)

8. **IOMMU (`vbox-devices/iommu`)**
   - **DevIommuAmd** (`DevIommuAmd.cpp`, 7,370 lines) → `devices/iommu/amd.rs`
   - **DevIommuIntel** → `devices/iommu/intel.rs`
   - In a browser context, IOMMU is less relevant (no DMA), but implement for completeness.

9. **Chipset/Bus (`vbox-devices/bus`)**
   - **DevPIIX3**, **DevICH9** → `devices/bus/chipset.rs`
   - **DevIommuAmd** (also bus-related) → see above
   - PCI bus configuration space → `devices/bus/pci.rs`

### Milestone: "Can boot a guest OS and display video output in the browser canvas"

---

## Phase 3 — Storage & Network Services (Months 10–16)

### Objective: Port storage image format drivers and network services.

### Deliverables

1. **Storage Image Formats (`vbox-storage`)**
   - Port `VD.cpp` (9,684 lines) → `storage/vd.rs`
   - Port `VMDK.cpp` (9,202 lines) → `storage/vmdk.rs`
   - Port `ISCSI.cpp` (5,567 lines) → `storage/iscsi.rs`
   - Port `VDI.cpp` (3,290 lines) → `storage/vdi.rs`
   - Port `VHD.cpp` (3,193 lines) → `storage/vhd.rs`
   - Port `QCOW.cpp` (2,563 lines) → `storage/qcow.rs`
   - Port `VHDX.cpp` (2,399 lines) → `storage/vhdx.rs`
   - Port `DMG.cpp` (2,393 lines) → `storage/dmg.rs`
   - Port `QED.cpp` (2,390 lines) → `storage/qed.rs`
   - **Key design**: Use `async` disk I/O with `IndexedDB` or `WebFileSystem API`. All reads are `Promise`-based and wrapped in `wasm-bindgen-futures`.
   - Implement `DiskImage` trait with `fn read_sector(&self, lba: u64) -> Result<Vec<u8>>` and `fn write_sector(&mut self, lba: u64, data: &[u8]) -> Result<()>`.

2. **VFS Layer (`vbox-storage/vfs`)**
   - Port `fatvfs.cpp`, `ntfsvfs.cpp`, `isovfs.cpp`, `udfhlp.cpp` → `storage/vfs/fat.rs`, `storage/vfs/ntfs.rs`, `storage/vfs/iso9660.rs`, `storage/vfs/udf.rs`
   - Replace with Rust-native filesystem implementations using `std::fs` or browser storage API
   - Implement `Vfs` trait: `fn open(&self, path: &str) -> Result<VfsFile>`, `fn read(&self, handle: VfsFile) -> Result<Vec<u8>>`

3. **Network Services (`vbox-network`)**
   - Port `VBoxNetSlirpNAT.cpp` (3,112 lines) → `network/slirp.rs`
   - Port `VBoxIntNetSwitch.cpp` (1,735 lines) → `network/intnet.rs`
   - Port `Dhcpd/` → `network/dhcp.rs`
   - Implement `NetworkService` trait: `fn nat_packet(&mut self, packet: &[u8]) -> Result<Vec<u8>>`
   - Map to browser network via `WebSocket` or `fetch`

### Milestone: "Can mount a virtual disk image and access it via the guest's virtual filesystem"

---

## Phase 4 — Main / VM Management & Services (Months 13–19)

### Objective: Port the Main COM layer and host services to Rust.

### Deliverables

1. **VirtualBox Object Model (`vbox-main`)**
   - Port `MachineImpl.cpp` (16,751 lines) → `main/machine.rs`
   - Port `VirtualBoxImpl.cpp` (7,000 lines) → `main/virtualbox.rs`
   - Port `MediumImpl.cpp` (11,076 lines) → `main/medium.rs`
   - Port `ConsoleImpl.cpp` (12,437 lines) → `main/console.rs`
   - **Key design**: Replace XPCOM with a Rust-native object model. Use `Arc<Mutex<VirtualMachine>>` for shared state. Expose to JavaScript via `wasm-bindgen` with `Promise`-based APIs.
   - Implement `VirtualMachine` struct with methods: `new()`, `start()`, `stop()`, `pause()`, `create_snapshot()`, `restore_snapshot()`

2. **Settings Management (`vbox-main/settings`)**
   - Port `Settings.cpp` (10,559 lines) → `main/settings.rs`
   - Replace XML-based settings with `serde`-based Rust structs
   - Support JSON format for browser compatibility
   - Implement `Settings` trait with serialization/deserialization

3. **Host Services (`vbox-hostservices`)**
   - Port Shared Folders → `services/shared_folders.rs`
     - Map to browser `FileSystem Access API` or `IndexedDB`
   - Port Guest Control → `services/guest_control.rs`
     - Map to browser `postMessage` / `WebSocket`
   - Port Shared Clipboard → `services/shared_clipboard.rs`
     - Use browser `Clipboard API` via `wasm-bindgen`
   - Port Drag & Drop → `services/drag_drop.rs`
     - Use HTML5 drag-and-drop events
   - Port Guest Properties → `services/guest_properties.rs`
     - Simple key-value store in WASM memory

4. **COM/XPCOM Layer (`vbox-main/com`)**
   - Port `include/VBox/com/` headers → `main/com.rs`
   - Replace XPCOM with a lightweight Rust trait-object pattern
   - Implement `IUnknown`, `IDispatch` equivalents as Rust traits
   - `wasm-bindgen` generates the JavaScript glue

### Milestone: "Can create, configure, start, and stop a VM from the browser UI"

---

## Phase 5 — Frontend / UI & User Experience (Months 16–22)

### Objective: Build a browser-based UI replacing the Qt frontend.

### Deliverables

1. **Web UI Framework**
   - Use `Yew` (Rust WASM framework, React-like) or `Leptos` (similar, with async/await)
   - Alternative: Build UI in TypeScript/JavaScript and use `wasm-bindgen` to call into the Rust VM engine
   - Recommended: Hybrid approach — Rust VM engine in WASM, UI in TypeScript calling Rust via `wasm-bindgen`

2. **VM Manager Interface**
   - VM list, VM details, snapshots
   - Start/Stop/Pause controls
   - Settings editor (VM configuration)

3. **Console Window**
   - Canvas element for VGA output
   - Keyboard/mouse input events → forwarded to WASM VM
   - Display scale, fullscreen mode
   - Screen rotation, resize handling

4. **Settings Editor**
   - VM creation wizard
   - Storage configuration (disk images, CD/DVD)
   - Network configuration (NAT, Bridged, Internal)
   - Display settings (video memory, 3D acceleration)
   - Audio settings

5. **Library Management**
   - Virtual disk image management (VDI, VMDK, VHD, etc.)
   - ISO image mounting
   - Shared folder management

6. **Performance Monitor**
   - CPU usage, memory usage, network I/O
   - Frame rate, latency

7. **Save/Load State UI**
   - Snapshot management
   - Export/Import VM

### Milestone: "Full browser UI for managing and running VirtualBox VMs"

---

## Phase 6 — Integration, Optimization & Hardening (Months 20–24)

### Objective: Polish, optimize, and harden the browser VirtualBox.

### Deliverables

1. **Performance Optimization**
   - Profile the interpreter loop (`IEM`): Optimize hot paths with `#[inline]`, `#[cold]`, target-specific SIMD
   - Consider a JIT recompiler using `cranelift` or `wasmer` for critical code paths
   - Optimize WASM binary size: Strip debug info, use `lto`, `wasm-opt` (Binaryen)
   - Memory optimization: Pre-allocate guest RAM, use memory pools
   - Cache frequently accessed data (CPUID tables, device configs)

2. **WebAssembly Threads**
   - If `SharedArrayBuffer` is available, use WASM threads for:
     - Device emulation threads (separate threads for USB, audio, network)
     - Async I/O handling
     - Timer management
   - Fallback to single-threaded time-slicing with `Atomics`

3. **WebGPU Acceleration**
   - Upgrade from WebGL2 to WebGPU for better 3D performance
   - Implement compute shaders for video processing
   - Use `wgpu` crate for cross-platform GPU abstraction

4. **Security Hardening**
   - Sandbox WASM module: Limit memory, disable expensive operations
   - Validate all guest inputs (memory accesses, device commands)
   - Use `wasm-bindgen` security best practices
   - Implement Content Security Policy (CSP) headers

5. **Browser Compatibility**
   - Test on Chrome, Firefox, Safari, Edge
   - Feature detection for WebGPU, WebAssembly threads, SIMD
   - Graceful degradation for older browsers

6. **Testing**
   - Unit tests for all modules (using `cargo test`)
   - Integration tests: Boot a guest OS, verify behavior
   - Fuzz testing: Use `cargo-fuzz` on disk image parsers and instruction decoders
   - Browser automated tests: Use `playwright` or `puppeteer` to test the UI

7. **Documentation**
   - Architecture documentation
   - API documentation (`wasm-bindgen` generated)
   - Developer guide for contributing
   - User guide for running VirtualBox in the browser

### Milestone: "Production-ready browser VirtualBox"

---

## Technical Risk Register

| Risk | Severity | Mitigation |
|------|----------|------------|
| IEM interpreter performance in WASM is too slow | **Critical** | Profile and optimize; consider JIT recompiler with `cranelift` or `wasmer`; use SIMD for instruction dispatch |
| WASM memory size limits guest RAM | **High** | Pre-allocate at VM start; implement memory compression; use `SharedArrayBuffer` for >4GB if available |
| WebGPU not available in all browsers | **High** | Fall back to WebGL2; use `wgpu`'s WebGL backend |
| WASM threads not available in Safari | **Medium** | Implement single-threaded fallback with time-slicing; use `Atomics` for synchronization |
| Browser security restrictions (CSP, COOP/COEP) | **Medium** | Use `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers; configure CSP appropriately |
| Large codebase porting time | **Medium** | Phased approach; prioritize core VMM and devices; use automated tools where possible |
| Disk image format compatibility | **Low** | Test against existing VDI/VMDK/VHD files; implement all formats |
| Audio/video synchronization | **Low** | Use browser audio/video APIs which handle synchronization |

---

## Dependencies & Tooling

### Rust Crates Required

| Category | Crates | Purpose |
|----------|--------|---------|
| WASM Glue | `wasm-bindgen`, `web-sys`, `wasm-bindgen-futures`, `js-sys` | Browser JavaScript interop |
| GPU | `wgpu`, `wgpu-types` | WebGPU abstraction (WebGL2/WebGPU) |
| Audio | `web-audio` | WebAudio API |
| Async Runtime | `tokio` (wasm target), `wasm-bindgen-futures`, `futures` | Async I/O |
| Sync | `parking_lot`, `crossbeam`, `flume`, `tokio::sync` | Thread synchronization |
| Data Structures | `smallvec`, `indexmap`, `linked-hash-map`, `hashbrown` | Fast collections |
| Serialization | `serde`, `bincode`, `serde_json`, `ron` | State save/load, settings |
| Crypto | `ring`, `blake3`, `sha2` | Hashing, encryption |
| Compression | `flate2`, `zstd`, `lz4` | Disk image compression |
| Networking | `web-sys` (WebSocket, fetch), `reqwest` (wasm) | Browser network |
| Storage | `idb-keyval`, `web-storage` | IndexedDB, localStorage |
| Logging | `log`, `console_log`, `tracing`, `tracing-wasm` | Browser console logging |
| Testing | `cargo-test`, `wasm-bindgen-test`, `playwright` | Unit/integration/browser tests |
| Build | `wasm-pack`, `binaryen`, `wasm-opt` | WASM optimization |
| Code Generation | `build.rs` (build scripts) | Auto-generate IEM tables, CPUID data |

### Build Pipeline

1. `cargo build --target wasm32-unknown-unknown` for development
2. `wasm-pack build --target web` for production
3. `wasm-opt -O4` for binary optimization (Binaryen)
4. `wasm-bindgen` for JavaScript glue generation
5. Bundler: `webpack` or `rollup` for the final browser bundle

### CI/CD

- GitHub Actions: Build on push, run tests, deploy to GitHub Pages or similar
- Browser testing: `playwright` / `puppeteer` for automated browser tests
- Fuzzing: `cargo fuzz` for disk image parsers and instruction decoders

---

## Milestones & Acceptance Criteria

### M1 — Foundation Complete (Month 3)
- [ ] `cargo build --target wasm32-unknown-unknown` succeeds
- [ ] IPRT memory, threads, strings, files ported
- [ ] Basic browser page loads and calls into Rust WASM
- [ ] Unit tests pass for all IPRT modules

### M2 — VM Core Complete (Month 8)
- [ ] IEM interpreter can execute x86 instructions
- [ ] Guest physical memory mapping works
- [ ] CPU state save/restore works
- [ ] Simple guest program (e.g., a bootloader) executes
- [ ] `EMR3ExecuteVM` equivalent runs in browser

### M3 — Device Emulation MVP (Month 14)
- [ ] VGA device renders to browser canvas
- [ ] Network device can send/receive packets
- [ ] Storage device reads from a virtual disk image
- [ ] Can boot a minimal guest OS
- [ ] Audio output works via WebAudio

### M4 — Full VM Boot (Month 16)
- [ ] Guest OS (e.g., Linux) boots to a usable desktop
- [ ] All major devices functional
- [ ] Save/load state works
- [ ] Shared folders, clipboard work

### M5 — Browser UI Complete (Month 22)
- [ ] Full VM manager UI in browser
- [ ] Console with canvas output and input
- [ ] Settings editor, snapshot management
- [ ] Performance monitoring

### M6 — Production Ready (Month 24)
- [ ] Performance optimized (interpreter speed, WASM binary size)
- [ ] Security hardened
- [ ] Cross-browser compatible
- [ ] Full test suite passing
- [ ] Documentation complete
- [ ] **Public release**

---

## Appendix A: Source File Migration Reference

### Files by Category — Original → Target Rust Module

| Original File (C/C++) | Lines | Target Rust Module |
|----------------------|-------|-------------------|
| `src/VBox/Runtime/common/alloc/*.cpp` | ~50K | `vbox-runtime/src/alloc.rs` |
| `src/VBox/Runtime/common/string/*.cpp` | ~150K | `vbox-runtime/src/string.rs` |
| `src/VBox/Runtime/common/thread/*.cpp` | ~30K | `vbox-runtime/src/thread.rs` |
| `src/VBox/Runtime/common/mem/*.cpp` | ~20K | `vbox-runtime/src/mem.rs` |
| `src/VBox/Runtime/r3/*.cpp` | ~50K | `vbox-runtime/src/r3.rs` |
| `src/VBox/VMM/VMMR3/EMR3.cpp` | 2,855 | `vbox-vmm/src/em.rs` |
| `src/VBox/VMM/VMMR3/VMR3.cpp` | 4,480 | `vbox-vmm/src/vm.rs` |
| `src/VBox/VMM/VMMR3/CPUMR3*.cpp` | ~10K | `vbox-vmm/src/cpu.rs` |
| `src/VBox/VMM/VMMR3/PGMR3*.cpp` | ~12K | `vbox-vmm/src/pgm.rs` |
| `src/VBox/VMM/VMMR3/TMR3.cpp` | 4,442 | `vbox-vmm/src/tm.rs` |
| `src/VBox/VMM/VMMR3/PDMR3*.cpp` | ~15K | `vbox-vmm/src/pdm.rs` |
| `src/VBox/VMM/VMMR3/SSMR3.cpp` | 10,237 | `vbox-vmm/src/ssm.rs` |
| `src/VBox/VMM/VMMAll/IEMAll*.cpp` | ~380K | `vbox-vmm/src/iem.rs`, `vbox-vmm/src/iem_tables.rs` |
| `src/VBox/VMM/VMMAll/CPUMAll*.cpp` | ~20K | `vbox-vmm/src/cpuid.rs` |
| `src/VBox/Devices/Graphics/DevVGA*.cpp` | ~60K | `vbox-devices/src/graphics/vga.rs` |
| `src/VBox/Devices/Network/DevE1000.cpp` | 9,784 | `vbox-devices/src/network/e1000.rs` |
| `src/VBox/Devices/Storage/DevAHCI.cpp` | 6,184 | `vbox-devices/src/storage/ahci.rs` |
| `src/VBox/Devices/Storage/DevNVMe.cpp` | 7,702 | `vbox-devices/src/storage/nvme.rs` |
| `src/VBox/Devices/USB/DevXHCI.cpp` | 8,378 | `vbox-devices/src/usb/xhci.rs` |
| `src/VBox/Devices/Audio/DevHda.cpp` | 5,469 | `vbox-devices/src/audio/hda.rs` |
| `src/VBox/Devices/VMMDev/VMMDev.cpp` | 5,467 | `vbox-devices/src/vmmdev.rs` |
| `src/VBox/Storage/VD.cpp` | 9,684 | `vbox-storage/src/vd.rs` |
| `src/VBox/Main/src-server/MachineImpl.cpp` | 16,751 | `vbox-main/src/machine.rs` |
| `src/VBox/Main/src-client/ConsoleImpl.cpp` | 12,437 | `vbox-main/src/console.rs` |
| `src/VBox/Main/xml/Settings.cpp` | 10,559 | `vbox-main/src/settings.rs` |
| `src/VBox/Runtime/r0drv/**/*.cpp/.c` | ~25K | `vbox-runtime/src/r0.rs` (no-op wrapper) |
| `src/VBox/Runtime/gc/**/*.cpp` | ~5K | `vbox-runtime/src/gc.rs` |

### Files Explicitly Excluded

| Directory | Reason |
|-----------|--------|
| `src/VBox/HostDrivers/` | Kernel-mode drivers — no browser equivalent |
| `src/VBox/Additions/` | Guest additions (Mesa, host drivers) — not needed |
| `src/VBox/Frontends/VirtualBox/src/ui/` | Qt GUI — replaced by browser UI (Phase 5) |
| `src/VBox/Frontends/VBoxManage/` | CLI frontend — not needed for browser |
| `src/VBox/RDP/` | Remote Desktop server — browser is the display |
| `src/VBox/ImageMounter/` | Host-side image mounting — not applicable |
| `src/VBox/ValidationKit/` | Testing framework — replaced by Rust tests |
| `src/VBox/Debugger/GUI/` | GUI debugger — replaced by browser-based debugger |
| `src/VBox/Installer/` | Installer — not needed for browser |
| `src/bldprogs/`, `src/apps/` | Build tools — not needed |
| All `.asm` files (assembly) | WASM JIT handles this; assembly is host-specific |

---

## Appendix B: Key Architectural Patterns

### PDM Device Registration → Rust Trait

**Original C++:**
```cpp
// pdmdev.h
DECLCALLBACK(int) pfnAttach(PPDMDEVINS pDevIns, PCDBGFREGDESC pDesc, void *pvUser);
PCPDMDEVHLPR3 g_pDevHlp = { /* function table */ };
```

**Target Rust:**
```rust
pub trait Device {
    fn attach(&mut self, config: &DeviceConfig) -> Result<()>;
    fn detach(&mut self) -> Result<()>;
    fn reset(&mut self) -> Result<()>;
    fn power_off(&mut self) -> Result<()>;
    fn query_interface(&self, iid: &Uuid) -> Option<&dyn Any>;
}

pub struct DeviceContext {
    pub helper: Box<dyn DeviceHelper>,
    pub mmio_regions: Vec<MmioRegion>,
    pub io_ports: Vec<IoPortRange>,
}
```

### Guest Memory → WASM Linear Memory

**Original C++:**
```cpp
// pgm.h
PGMRAMRANGE *pRange; // Region descriptor
uint8_t *pvShadow;   // Shadow page
```

**Target Rust:**
```rust
pub struct GuestMemory {
    pub ram: Box<[u8]>,           // WASM linear memory slice
    pub page_table: [u64; 512],   // 4-level page table (simplified)
    pub mmio_regions: Vec<MmioRegion>,
}

impl GuestMemory {
    pub fn read(&self, gpa: u64, size: usize) -> Result<Vec<u8>> {
        let pfn = self.translate(gpa)?;
        Ok(&self.ram[pfn..pfn + size])
    }
    pub fn write(&mut self, gpa: u64, data: &[u8]) -> Result<()> {
        let pfn = self.translate(gpa)?;
        self.ram[pfn..pfn + data.len()].copy_from_slice(data);
        Ok(())
    }
}
```

### WASM Module Export

**Generated via `wasm-bindgen`:**
```rust
#[wasm_bindgen]
pub struct VirtualMachine {
    inner: Rc<RefCell<VM>>,
}

#[wasm_bindgen]
impl VirtualMachine {
    pub fn new(config: &VmConfig) -> Result<VirtualMachine> { ... }
    pub fn start(&mut self) -> Result<()> { ... }
    pub fn pause(&mut self) -> Result<()> { ... }
    pub fn stop(&mut self) -> Result<()> { ... }
    pub fn read_frame(&self, buffer: &mut [u8]) -> Result<()> { ... }
}
```

---

## Appendix C: Timeline Summary

```
Month  0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24
       |==|=======|========|=============|======================|===============|
       Phase 0         Phase 1              Phase 2                Phase 3-4
       Foundation      Core VMM             Device Emulation         Services & Mgmt
                        ↓                      ↓                       ↓
                   M1: Build ✓           M3: Boot MVP           M4: Full Boot ✓
                                                      M5: UI Complete ✓
                                                              M6: Production ✓
```

---

*This plan is based on analysis of the VirtualBox source tree on the `virtualbox` branch (38,019 C/C++ files, ~17.2M lines). The `main` branch is empty and ready for the Rust + WebAssembly rewrite. All paths referenced are relative to the `virtualbox` branch.*
