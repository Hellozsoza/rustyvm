# VirtualBox Browser edition (experimental)

This standalone frontend provides a VirtualBox-style manager with real local
x86 guest execution through **v86 WebAssembly**. It reuses the native frontend's
icons and familiar layout. It does **not** compile the VirtualBox hypervisor,
IPRT, XPCOM, or Qt GUI to WASM; the VM execution engine is replaced by v86.
Native VirtualBox's build and public APIs are unchanged.

## Build and run

Use Node.js 24, npm, and curl. From this directory:

```sh
npm ci --cache /workspace/.cache/npm --ignore-scripts
npm run build
npm start
```

The server binds to loopback port 8080 by default. `HOST` and `PORT` override
the bind address and port. Open it in a browser when running on your own machine,
or deploy the **contents of `dist/`** to an HTTPS static host. Cloud onboarding
does not provide a browser preview link. No backend VM service is needed; the
server only serves static files. HTTPS or localhost is required for Web Locks.

Use a recent Chromium or Firefox browser with WebAssembly, IndexedDB, Web Locks,
and sufficient memory. Chromium is the browser exercised by the automated tests;
Firefox, Safari, mobile browsers, fullscreen, audio output, and mouse integration
are not yet independently validated. SharedArrayBuffer and cross-origin
isolation are not required. Keep the entire distribution on one origin.

The build copies pinned `v86@0.5.470` assets, downloads two BIOS binaries from
the matching upstream commit, verifies their SHA-256 checksums, copies icons and
license notices, and generates a small bootable floppy. Subsequent builds reuse
the checksum-verified BIOS files. Initial installation needs
`registry.npmjs.org` and `raw.githubusercontent.com`; running `dist/` needs
no external downloads. Node fetch does not automatically use the cloud proxy,
so the build uses curl with TLS verification enabled.

## Using the manager

1. Start **WebAssembly Demo** to boot actual 16-bit guest code. It writes its
   banner to VGA and COM1, increments a byte in sector two using BIOS disk I/O,
   and echoes PS/2 input. It is a hardware smoke test, not a bundled OS.
2. Use **New** to create a VM. Select a boot device and attach your own compatible
   16/32-bit OS media. You can attach an installer ISO and a blank raw IDE disk
   together. There is one emulated CPU, 16–8192 MB RAM settings (current engine runs up to 2047 MB), and 8 MB VGA memory.
3. Click the guest display to capture keyboard/mouse input. Escape releases it.
   Text injection assumes the guest's US keyboard layout. Use Pause/Resume,
   Reset, Ctrl+Alt+Del, and Full Screen from the console toolbar.
4. **Save State & Close** persists CPU, RAM, and media to IndexedDB. **Start**
   resumes the saved state. Discard it before editing hardware settings.
5. **Take Snapshot** stores CPU, memory, settings, and media. Restore snapshots
   while powered off; this replaces the current disks and settings. Five
   snapshots per VM are allowed. Clone creates independent media copies and
   omits saved execution states and snapshots.
6. Export raw disks and saved v86 states from Details. Import states only into
   a VM with the same engine version, RAM, and media configuration. Native
   VirtualBox `.sav` states are incompatible. Keep exports before clearing
   browser site data. State files can include all guest memory and disk data.

Only one VM may run, and only one manager tab may write storage on an origin.
Media is read locally and never uploaded. Disk writes persist on explicit
Power Off, Save State, and snapshots. Tab closure, crashes, or clearing site data
can lose unsaved writes; save before leaving. Browser storage quotas may prevent
saves. Failed writes leave the running guest available for retry rather than
closing it. VM list and settings are stored in a persistent SameSite=Strict cookie (one-year lifetime); media, saved execution states, and snapshots remain in IndexedDB. Existing IndexedDB VM definitions migrate automatically. Cookies contain no image or saved-state bytes. The catalog is limited to 3800 encoded bytes; saves that exceed this capacity fail visibly without deleting existing VMs. Cookie metadata accompanies same-origin HTTP requests. Cookie and IndexedDB writes cannot form a single atomic transaction; failed IndexedDB writes restore the previous catalog.

RAM settings support up to 8 GB. This pinned v86 engine uses signed 32-bit memory arithmetic and cannot provide 8 GB of guest RAM. Starting a VM configured above 2047 MB fails with an explicit message before any allocation; a different engine is needed to execute that configuration. Browser allocation limits may also prevent smaller VMs from starting.

## Compatibility and limits

| Capability | Browser edition |
| --- | --- |
| CPU | v86 software emulation, compatible 16/32-bit x86 guests |
| Display | VGA and Bochs VBE; text and graphical canvas |
| Storage | Local ISO, standard floppy, sector-aligned raw disks |
| VDI | Standalone fixed/dynamic VDI 1.1 converted to raw; block mapping validated |
| Persistence | VM list/settings in cookies; IndexedDB for RAM/CPU state, media, snapshots, raw disk export |
| RAM/media | 16–8192 MB RAM settings (current engine runs up to 2047 MB); maximum 2 GB per image; full images held in memory |
| Guest networking | Disconnected; no NAT, relay, or bridged interface configured |
| Unsupported | 64-bit, multicore, VT-x/AMD-V, USB/PCI passthrough, 3D acceleration |
| Native integration | No VirtualBox Guest Additions, shared folders, host clipboard, XPCOM API |
| Native formats | No `.vbox`, `.sav`, OVA/OVF, VMDK, VHD/VHDX, QCOW2, or differencing VDI |

Convert unsupported disks to raw using a trusted native tool before attaching
them. Changing an extension does not convert an image. VDI files with parents,
oversized virtual capacity, corrupt maps, or truncated blocks are rejected.
Guest compatibility and speed are those of [v86](https://github.com/copy/v86),
not those of native VirtualBox. No commercial guest OS image is bundled.

A true port of VirtualBox's execution engine would additionally require a
WASM-compatible IPRT, an interpreter-only VMM replacing ring-0 services and native
assembly/JIT dependencies, browser implementations of timers/threading/storage,
device integration, and a port of the Qt/Main frontend. That work is not
implemented by this browser edition. Hardware passthrough and native
virtualization remain inaccessible to ordinary browser pages.

## Validation

```sh
npm test
npm run test:browser
```

Browser tests use `/usr/bin/chromium`; set `CHROMIUM_PATH` for another installed
Chromium executable. Playwright starts the server when needed. No browser download
is necessary in the prepared cloud machine. Tests execute real WASM guests and
verify VGA/serial output, physical and injected keyboard input, pause/resume,
snapshot rollback, saved state across reload, local floppy/raw/VDI/ISO boot,
guest disk writes before and after restore, manager operations, markup injection
resistance, missing-asset failures, and competing-tab protection. Unit tests
exercise hostile VDI layouts and configuration/media bounds.

## Sources and licenses

The frontend and reused VirtualBox icons are GPL-3.0-only; see
`licenses/VirtualBox-COPYING`. v86 is BSD-2-Clause; its notice is copied to
`dist/vendor/v86-LICENSE`.
Its embedded SoftFloat, Zstandard, and floppy code notices are included under
`licenses/` and copied to the distribution. SeaBIOS and its VGA BIOS are LGPLv3;
`licenses/SeaBIOS-LICENSE` and the GPL text are included.

BIOS binaries come from v86 commit
`6db8b157974dbaf1b54d2c2ec12dd71ddc1891e9`. Its
[BIOS build script](https://github.com/copy/v86/blob/6db8b157974dbaf1b54d2c2ec12dd71ddc1891e9/bios/fetch-and-build-seabios.sh)
builds [SeaBIOS rel-1.16.2](https://github.com/coreboot/seabios/tree/rel-1.16.2)
using the accompanying `seabios.config`. These are the corresponding source,
configuration, and build instructions for the unchanged binaries. Distributors
must comply with each dependency's source and notice requirements. The browser
edition is experimental and is not an Oracle-supported VirtualBox release.

## Deploy to Vercel

Connect this Git repository in Vercel and set the project root to
`browser/`. `vercel.json` installs locked dependencies, builds
the static frontend and deploys `dist/` with a restrictive content security
policy. The build downloads firmware from its pinned upstream revision and
verifies both SHA-256 checksums. No environment variables or secrets are needed.

From an authenticated Vercel CLI session, run `vercel deploy --prod` in this
directory. Vercel authentication stays within its supported CLI.

Official Arch Linux installation ISOs require x86-64 and cannot boot in this engine. Arch Linux 32 provides separate 32-bit media; compatibility still depends on v86. Images up to 2 GB may be attached, but loading, cloning, saving, and snapshots require additional browser memory and storage quota.
