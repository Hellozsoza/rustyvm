/* $Id$ */
/** @file WebAssembly guest lifecycle and mutable media management. */
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
import { V86 } from "./vendor/libv86.mjs";
import { ENGINE_VERSION, validateRuntimeMemory } from "./model.mjs";

async function fetchBytes(path) {
    const response = await fetch(path);
    if (!response.ok)
        throw new Error(`Could not load ${path}: HTTP ${response.status}`);
    return response.arrayBuffer();
}

export class MachineRuntime {
    constructor(screen, serial, update) {
        this.screen = screen;
        this.serial = serial;
        this.update = update;
        this.emulator = null;
    }

    async start(machine) {
        validateRuntimeMemory(machine.config.memory);
        if (this.emulator)
            throw new Error("Power off the current VM before starting another.");
        if (machine.saved && machine.saved.version !== ENGINE_VERSION)
            throw new Error("Saved state belongs to a different emulator version. Discard it before booting.");
        const [bios, vga, wasm] = await Promise.all([
            fetchBytes("./vendor/seabios.bin"), fetchBytes("./vendor/vgabios.bin"), fetchBytes("./vendor/v86.wasm")
        ]);
        let module;
        try { module = await WebAssembly.compile(wasm); }
        catch { module = await WebAssembly.compile(await fetchBytes("./vendor/v86-fallback.wasm")); }
        const options = {
            bios: { buffer: bios }, vga_bios: { buffer: vga },
            memory_size: machine.config.memory * 1024 * 1024,
            vga_memory_size: 8 * 1024 * 1024,
            screen_container: this.screen, autostart: false,
            // Supply a rejecting loader so missing or invalid WASM produces an actionable failure.
            wasm_fn: imports => Promise.resolve(new WebAssembly.Instance(module, imports).exports),
            boot_order: { demo: 0x321, floppy: 0x321, cdrom: 0x123, disk: 0x132 }[machine.config.boot]
        };
        this.buffers = {};
        for (const [kind, key] of [["disk", "hda"], ["cdrom", "cdrom"], ["floppy", "fda"]]) {
            if (machine.media[kind]) {
                // v86's SyncBuffer mutates the provided buffer. Keep a separate owned copy
                // so failed persistence cannot silently modify the stored source image.
                const buffer = machine.media[kind].data.slice(0);
                this.buffers[kind] = buffer;
                options[key] = { buffer };
            }
        }
        if (machine.config.boot === "demo")
            options.fda = { buffer: await fetchBytes("./demo.img") };
        else if (!machine.media[machine.config.boot])
            throw new Error("Attach media for the selected boot device in Settings.");
        this.serial.value = "";
        this.machineId = machine.id;
        this.emulator = new V86(options);
        this.focus(false);
        this.emulator.add_listener("serial0-output-byte", byte => {
            this.serial.value = (this.serial.value + String.fromCharCode(byte)).slice(-65536);
            this.serial.scrollTop = this.serial.scrollHeight;
        });
        try {
            await Promise.race([
                new Promise(resolve => this.emulator.add_listener("emulator-ready", resolve)),
                new Promise((resolve, reject) => { this.timer = setTimeout(() => reject(new Error("Emulator startup timed out.")), 30000); })
            ]);
            if (machine.saved)
                await this.emulator.restore_state(machine.saved.state);
            await this.emulator.run();
            this.update("Running");
        } catch (error) {
            await this.destroy();
            throw error;
        } finally {
            clearTimeout(this.timer);
        }
    }

    focus(enabled) {
        this.emulator?.keyboard_set_enabled(enabled);
        this.emulator?.mouse_set_enabled(enabled);
    }

    async pause() {
        if (this.emulator.is_running()) {
            await this.emulator.stop();
            this.focus(false);
            this.update("Paused");
            return;
        }
        await this.emulator.run();
        this.update("Running");
    }

    async capture(machine) {
        const wasRunning = this.emulator.is_running();
        await this.emulator.stop();
        this.focus(false);
        try {
            const state = await this.emulator.save_state();
            const media = this.currentMedia(machine);
            return { version: ENGINE_VERSION, state, media, config: structuredClone(machine.config), created: Date.now() };
        } finally {
            if (wasRunning)
                await this.emulator.run();
        }
    }

    currentMedia(machine) {
        const media = structuredClone(machine.media);
        // Restoring state replaces SyncBuffer.buffer; the original input buffers
        // no longer hold current disk writes. Read the live devices instead.
        // IDE access is specific to the pinned v86 version and covered by tests.
        if (media.disk)
            media.disk.data = this.emulator.v86.cpu.devices.ide.primary.master.buffer.buffer.slice(0);
        if (media.floppy && machine.config.boot !== "demo")
            media.floppy.data = this.emulator.get_disk_fda().slice().buffer;
        return media;
    }

    async destroy() {
        if (this.emulator) {
            this.focus(false);
            await this.emulator.destroy();
        }
        this.emulator = null;
        this.machineId = null;
        this.buffers = {};
        clearTimeout(this.timer);
    }
}
