/* $Id$ */
/** @file Browser distribution and reproducible real-mode guest builder. */
/*
 * Copyright (C) 2026 RustyVM contributors.
 *
 * This file is part of the experimental VirtualBox browser frontend.
 * This program is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 3.
 * This program is distributed without any warranty; without even the implied
 * warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the repository COPYING file for the full license text.
 * SPDX-License-Identifier: GPL-3.0-only
 */
import { mkdir, readFile, writeFile, copyFile, cp } from "node:fs/promises";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const root = fileURLToPath(new URL("../", import.meta.url));
const dist = `${root}dist`;
await mkdir(`${dist}/vendor`, { recursive: true });
await mkdir(`${dist}/icons`, { recursive: true });
await cp(`${root}app`, dist, { recursive: true });
for (const name of ["libv86.mjs", "v86.wasm", "v86-fallback.wasm"])
    await copyFile(`${root}node_modules/v86/build/${name}`, `${dist}/vendor/${name}`);
await copyFile(`${root}node_modules/v86/LICENSE`, `${dist}/vendor/v86-LICENSE`);
await copyFile(`${root}README.md`, `${dist}/README.md`);
await cp(`${root}licenses`, `${dist}/licenses`, { recursive: true });

const revision = "6db8b157974dbaf1b54d2c2ec12dd71ddc1891e9";
const bios = {
    "seabios.bin": "73e3f359102e3a9982c35fce98eb7cd08f18303ac7f1ba6ebfbe6cdc1c244d98",
    "vgabios.bin": "a4bc0d80cc3ca028c73dafa8fee396b8d054ce87ebd8abfbd31b06b437607880"
};
for (const [name, checksum] of Object.entries(bios)) {
    let bytes;
    try { bytes = await readFile(`${dist}/vendor/${name}`); } catch { /* first build */ }
    if (!bytes || createHash("sha256").update(bytes).digest("hex") !== checksum) {
        // curl respects the cloud HTTPS proxy; Node fetch does not automatically do so.
        const result = await promisify(execFile)("curl", ["--fail", "--silent", "--show-error", "--location",
            "--proto", "=https", "--proto-redir", "=https", "--tlsv1.2", "--connect-timeout", "15", "--max-time", "60",
            `https://raw.githubusercontent.com/copy/v86/${revision}/bios/${name}`], { encoding: "buffer" });
        bytes = result.stdout;
    }
    if (createHash("sha256").update(bytes).digest("hex") !== checksum)
        throw new Error(`BIOS integrity check failed: ${name}`);
    await writeFile(`${dist}/vendor/${name}`, bytes);
}
await cp(`${root}app/icons`, `${dist}/icons`, { recursive: true });

// Assemble a small 16-bit guest without a native assembler. It boots via SeaBIOS,
// writes to VGA and COM1, and echoes actual PS/2 keyboard input on both devices.
const code = [];
const labels = new Map();
const fixups = [];
const emit = (...bytes) => code.push(...bytes);
const label = name => labels.set(name, code.length);
const word = value => emit(value & 255, value >> 8 & 255);
const jump = (opcode, target) => { emit(opcode); fixups.push([code.length, target, "relative"]); word(0); };
emit(0xfa, 0x31, 0xc0, 0x8e, 0xd8, 0x8e, 0xc0, 0x8e, 0xd0, 0xbc, 0x00, 0x7c, 0xfb);
// Save SeaBIOS's boot drive and increment a persistent byte in sector two.
// This exercises real guest disk I/O, not a host-side simulated write.
emit(0x88, 0x16, 0x00, 0x06);
for (const operation of [2, 3]) {
    emit(0xbb, 0x00, 0x08, 0xb8, 0x01, operation, 0xb9, 0x02, 0x00,
        0xb6, 0x00, 0x8a, 0x16, 0x00, 0x06, 0xcd, 0x13);
    if (operation === 2)
        emit(0xfe, 0x06, 0x00, 0x08);
}
// Initialize COM1 (9600 baud, 8N1).
for (const [port, value] of [[0x3f9, 0], [0x3fb, 0x80], [0x3f8, 12], [0x3f9, 0], [0x3fb, 3]]) {
    emit(0xba); word(port); emit(0xb0, value, 0xee);
}
emit(0xbe); fixups.push([code.length, "message", "absolute"]); word(0);
label("print"); emit(0xac, 0x84, 0xc0, 0x74, 6); jump(0xe8, "putchar"); jump(0xe9, "print");
label("keyboard"); emit(0x31, 0xc0, 0xcd, 0x16); jump(0xe8, "putchar"); jump(0xe9, "keyboard");
label("putchar"); emit(0x50, 0x53, 0x52, 0xb4, 0x0e, 0xbb, 0x07, 0x00, 0xcd, 0x10, 0x5a, 0x5b, 0x58,
    0x52, 0xba, 0xf8, 0x03, 0xee, 0x5a, 0xc3);
label("message");
emit(...Buffer.from("VirtualBox Browser: real x86 guest running in WebAssembly\r\nType here to test the virtual keyboard.\r\n> "), 0);
for (const [offset, target, kind] of fixups) {
    const value = kind === "absolute" ? 0x7c00 + labels.get(target) : labels.get(target) - offset - 2;
    code[offset] = value & 255;
    code[offset + 1] = value >> 8 & 255;
}
if (code.length > 510)
    throw new Error("Demo boot sector overflow");
const floppy = Buffer.alloc(1440 * 1024);
floppy.set(code); floppy[510] = 0x55; floppy[511] = 0xaa;
await writeFile(`${dist}/demo.img`, floppy);
console.log("Built browser edition with verified BIOS, pinned WASM engine, and bootable demo.");
