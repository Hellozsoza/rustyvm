/* $Id$ | Hostile media validation tests | SPDX-License-Identifier: GPL-3.0-only */
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
import test from "node:test";
import assert from "node:assert/strict";
import { validateConfig, vdiToRaw, readMedia } from "../app/model.mjs";

function vdi(type = 1) {
    const data = new ArrayBuffer(2048);
    const view = new DataView(data);
    for (const [offset, value] of [[64, 0xbeda107f], [68, 0x10001], [72, 400], [76, type],
        [340, 512], [344, 1024], [376, 512], [384, 2], [388, 2], [512, 1], [516, 0]])
        view.setUint32(offset, value, true);
    view.setBigUint64(368, 1024n, true);
    new Uint8Array(data, 1024, 512).fill(0x41);
    new Uint8Array(data, 1536, 512).fill(0x42);
    return data;
}

test("validates memory bounds, boot devices and untrusted config", () => {
    const config = { name: "  Linux  ", os: "Linux", memory: 64, boot: "disk" };
    assert.equal(validateConfig(config).name, "Linux");
    for (const memory of [0, 513, NaN, Infinity, 32.5, "64"])
        assert.throws(() => validateConfig({ ...config, memory }));
    for (const name of ["", " ", "x".repeat(121), null])
        assert.throws(() => validateConfig({ ...config, name }));
    assert.throws(() => validateConfig({ ...config, boot: "usb" }));
});

test("converts fixed and dynamic VDI block maps rather than using file order", () => {
    for (const type of [1, 2]) {
        const output = new Uint8Array(vdiToRaw(vdi(type)));
        assert.equal(output.length, 1024);
        assert.ok(output.subarray(0, 512).every(byte => byte === 0x42));
        assert.ok(output.subarray(512).every(byte => byte === 0x41));
    }
});

test("unallocated and explicit-zero blocks are zero filled", () => {
    const input = vdi();
    const view = new DataView(input);
    view.setUint32(512, 0xffffffff, true);
    view.setUint32(516, 0xfffffffe, true);
    assert.ok(new Uint8Array(vdiToRaw(input)).every(byte => byte === 0));
});

test("rejects truncated, oversized, linked and corrupt VDI input before allocation", () => {
    assert.throws(() => vdiToRaw(new ArrayBuffer(10)), /Truncated/);
    for (const [offset, value] of [[76, 4], [424, 1], [340, 0], [344, 9999], [376, 0], [384, 900], [512, 99], [516, 1]]) {
        const input = vdi(); new DataView(input).setUint32(offset, value, true);
        assert.throws(() => vdiToRaw(input));
    }
    const huge = vdi(); new DataView(huge).setBigUint64(368, 1n << 60n, true);
    assert.throws(() => vdiToRaw(huge), /at most 512 MB/);
});

test("VDI blocks with metadata prefixes and partial final blocks", () => {
    const input = new ArrayBuffer(2576);
    new Uint8Array(input).set(new Uint8Array(vdi()).subarray(0, 1024));
    const view = new DataView(input);
    view.setBigUint64(368, 1536n, true); view.setUint32(376, 1024, true); view.setUint32(380, 8, true);
    new Uint8Array(input, 1032, 1024).fill(0x44);
    // Allocate a full second physical block (including padding) as required by VDI.
    const full = new ArrayBuffer(3088); new Uint8Array(full).set(new Uint8Array(input));
    new Uint8Array(full, 2064, 1024).fill(0x55);
    const result = new Uint8Array(vdiToRaw(full));
    assert.ok(result.subarray(0, 1024).every(byte => byte === 0x55));
    assert.ok(result.subarray(1024).every(byte => byte === 0x44));
});

test("rejects unsupported formats, empty media and invalid floppy/sector sizes", async () => {
    await assert.rejects(readMedia(new File([], "empty.img"), "disk"));
    await assert.rejects(readMedia(new File([new Uint8Array(512)], "disk.vmdk"), "disk"), /Convert/);
    await assert.rejects(readMedia(new File([new Uint8Array(513)], "disk.img"), "disk"), /aligned/);
    await assert.rejects(readMedia(new File([new Uint8Array(512)], "floppy.img"), "floppy"), /standard/);
    const media = await readMedia(new File([vdi()], "disk.vdi"), "disk");
    assert.equal(media.data.byteLength, 1024);
});
