/* $Id$ */
/** @file Browser VM configuration validation and media conversion. */
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
export const ENGINE_VERSION = "0.5.470";
export const MAX_RAM_MB = 8192;
// v86 clamps signed 32-bit RAM sizes; larger multiples can wrap to zero.
export const ENGINE_MAX_RAM_MB = 2047;
export function validateRuntimeMemory(memory) {
    if (memory > ENGINE_MAX_RAM_MB)
        throw new Error(`This v86 engine supports at most ${ENGINE_MAX_RAM_MB} MB guest RAM. The ${memory} MB setting is saved, but requires a different engine to run.`);
}

export const MAX_MEDIA_BYTES = 2048 * 1024 * 1024;

/** Validate untrusted configuration before allocating guest resources. */
export function validateConfig(input) {
    if (!input || typeof input.name !== "string" || !input.name.trim() || input.name.length > 120)
        throw new Error("Name must contain 1–120 characters.");
    if (!Number.isInteger(input.memory) || input.memory < 16 || input.memory > MAX_RAM_MB)
        throw new Error("Memory must be an integer between 16 and 8192 MB.");
    if (!["Other", "Linux", "Windows"].includes(input.os))
        throw new Error("Unsupported operating system category.");
    if (!["demo", "floppy", "cdrom", "disk"].includes(input.boot))
        throw new Error("Unsupported boot device.");
    return { name: input.name.trim(), memory: input.memory, os: input.os, boot: input.boot };
}

/** Convert standalone fixed/dynamic VDI 1.1 images to raw IDE media. */
export function vdiToRaw(buffer) {
    if (!(buffer instanceof ArrayBuffer) || buffer.byteLength < 472)
        throw new Error("Truncated VDI header.");
    const view = new DataView(buffer);
    const u32 = offset => view.getUint32(offset, true);
    if (u32(64) !== 0xbeda107f || u32(68) !== 0x00010001 || u32(72) < 400)
        throw new Error("Only VDI version 1.1 is supported.");
    const type = u32(76);
    if (type !== 1 && type !== 2)
        throw new Error("Differencing and undo VDI images require their parents and are not supported.");
    // Parent UUIDs must be zero even when an invalid image claims a standalone type.
    if (new Uint8Array(buffer, 424, 32).some(byte => byte !== 0))
        throw new Error("VDI images with parent linkage are not supported.");
    const sizeBig = view.getBigUint64(368, true);
    if (sizeBig === 0n || sizeBig > BigInt(MAX_MEDIA_BYTES) || sizeBig % 512n)
        throw new Error("VDI virtual disk must be sector-aligned and at most 2 GB.");
    const size = Number(sizeBig);
    const blockSize = u32(376), extra = u32(380), count = u32(384), allocated = u32(388);
    const map = u32(340), data = u32(344);
    if (!blockSize || blockSize % 512 || count !== Math.ceil(size / blockSize)
        || allocated > count || map < 72 + u32(72) || map + count * 4 > data || data > buffer.byteLength
        || data + allocated * (blockSize + extra) > buffer.byteLength)
        throw new Error("Invalid or truncated VDI block layout.");
    const raw = new Uint8Array(size);
    const used = new Set();
    for (let index = 0; index < count; index++) {
        const physical = u32(map + index * 4);
        if (physical === 0xffffffff || physical === 0xfffffffe)
            continue;
        if (physical >= allocated || used.has(physical))
            throw new Error("Invalid VDI block reference.");
        used.add(physical);
        const offset = data + physical * (blockSize + extra) + extra;
        const length = Math.min(blockSize, size - index * blockSize);
        raw.set(new Uint8Array(buffer, offset, length), index * blockSize);
    }
    return raw.buffer;
}

/** Read bounded local media; never upload it to a server. */
export async function readMedia(file, kind) {
    if (file.size === 0 || file.size > MAX_MEDIA_BYTES)
        throw new Error("Each media image must contain data and be at most 2 GB.");
    let data;
    try { data = await file.arrayBuffer(); }
    catch { throw new Error("The browser could not load this image into memory. Try a smaller image or free memory and retry."); }
    if (file.name.toLowerCase().endsWith(".vdi")) {
        if (kind !== "disk")
            throw new Error("VDI images can only be attached as hard disks.");
        data = vdiToRaw(data);
    } else if (/\.(vmdk|vhd|vhdx|qcow2|ova|ovf|vbox)$/i.test(file.name)) {
        throw new Error("Convert this format to raw IMG first; it cannot be attached directly.");
    }
    if (kind === "disk" && data.byteLength % 512)
        throw new Error("Raw hard disk images must be aligned to 512-byte sectors.");
    if (kind === "floppy" && ![360, 720, 1200, 1440, 2880].some(kb => data.byteLength === kb * 1024))
        throw new Error("Use a standard 360 KB, 720 KB, 1.2 MB, 1.44 MB, or 2.88 MB floppy image.");
    return { name: file.name, data };
}
