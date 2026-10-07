/* $Id$ | Real guest disk I/O and local boot media tests | SPDX-License-Identifier: GPL-3.0-only */
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
import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";

async function create(page, kind, name, buffer) {
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    await page.locator("#new").click();
    await page.locator('[name="name"]').fill(`${kind} guest`);
    await page.locator('[name="boot"]').selectOption(kind);
    await page.locator(`[name="${kind}"]`).setInputFiles({ name, mimeType: "application/octet-stream", buffer });
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.locator("#editor")).not.toBeVisible();
    await page.locator("#start").click();
    await expect(page.locator("#serial")).toHaveValue(/real x86 guest/, { timeout: 30000 });
}

async function storedCounter(page, kind) {
    return page.evaluate(async kind => {
        const { MachineStore } = await import("./storage.mjs");
        const store = new MachineStore(); await store.open();
        const machines = await store.list(); store.db.close();
        const machine = machines.find(entry => entry.config.name === `${kind} guest`);
        return new Uint8Array(machine.media[kind].data)[512];
    }, kind);
}

function fixedVdi(raw) {
    const blocks = Math.ceil(raw.length / 1048576);
    const data = Buffer.alloc(1024 + blocks * 1048576);
    for (const [offset, value] of [[64, 0xbeda107f], [68, 0x10001], [72, 400], [76, 2],
        [340, 512], [344, 1024], [376, 1048576], [384, blocks], [388, blocks]])
        data.writeUInt32LE(value, offset);
    data.writeBigUInt64LE(BigInt(raw.length), 368);
    for (let i = 0; i < blocks; i++) data.writeUInt32LE(i, 512 + i * 4);
    raw.copy(data, 1024);
    return data;
}

for (const format of ["floppy", "raw", "vdi"]) {
    test(`${format} guest writes persist through save/restore and power off`, async ({ page }) => {
        page.on("dialog", dialog => dialog.accept());
        const raw = await readFile(new URL("../dist/demo.img", import.meta.url));
        const kind = format === "floppy" ? "floppy" : "disk";
        await create(page, kind, `guest.${format === "vdi" ? "vdi" : "img"}`, format === "vdi" ? fixedVdi(raw) : raw);
        await page.locator("#save").click();
        await expect(page.locator("#status")).toContainText("state saved");
        expect(await storedCounter(page, kind)).toBe(1);
        await page.locator("#start").click();
        await expect(page.locator("#console-title")).toContainText("Running");
        // A reset executes the guest's real INT 13 disk read/increment/write again.
        await page.locator("#reset").click();
        await expect(page.locator("#serial")).toHaveValue(/real x86 guest/, { timeout: 10000 });
        await page.locator("#poweroff").click();
        await expect(page.locator("#status")).toContainText("Disk writes saved");
        expect(await storedCounter(page, kind)).toBe(2);
        await page.reload();
        expect(await storedCounter(page, kind)).toBe(2);
    });
}

test("local El Torito ISO boots in the WASM guest", async ({ page }) => {
    const floppy = await readFile(new URL("../dist/demo.img", import.meta.url));
    const iso = Buffer.alloc(32 * 2048 + floppy.length);
    // Minimal ISO9660 descriptors and El Torito floppy-emulation boot catalog.
    for (const [sector, type] of [[16, 1], [17, 0], [18, 255]]) {
        iso[sector * 2048] = type; iso.write("CD001", sector * 2048 + 1); iso[sector * 2048 + 6] = 1;
    }
    iso.write("EL TORITO SPECIFICATION", 17 * 2048 + 7);
    iso.writeUInt32LE(20, 17 * 2048 + 71);
    const catalog = 20 * 2048;
    iso[catalog] = 1; iso[catalog + 30] = 0x55; iso[catalog + 31] = 0xaa;
    let sum = 0; for (let i = 0; i < 32; i += 2) sum += iso.readUInt16LE(catalog + i);
    iso.writeUInt16LE(-sum & 0xffff, catalog + 28);
    iso[catalog + 32] = 0x88; iso[catalog + 33] = 2;
    iso.writeUInt16LE(1, catalog + 38); iso.writeUInt32LE(32, catalog + 40);
    floppy.copy(iso, 32 * 2048);
    await create(page, "cdrom", "guest.iso", iso);
    await expect(page.locator("#screen")).toContainText("real x86 guest");
});

test("guest VGA graphics mode renders colored framebuffer pixels", async ({ page }) => {
    const floppy = Buffer.alloc(1440 * 1024);
    // Set mode 13h and fill A000:0000 with palette color 42, then halt.
    floppy.set([0xfa, 0x31, 0xc0, 0x8e, 0xd8, 0xb8, 0x13, 0x00, 0xcd, 0x10,
        0xb8, 0x00, 0xa0, 0x8e, 0xc0, 0x31, 0xff, 0xb9, 0xff, 0xff, 0xb0, 42,
        0xfc, 0xf3, 0xaa, 0xf4, 0xeb, 0xfd]);
    floppy[510] = 0x55; floppy[511] = 0xaa;
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    await page.locator("#new").click();
    await page.locator('[name="boot"]').selectOption("floppy");
    await page.locator('[name="floppy"]').setInputFiles({ name: "vga.img", mimeType: "application/octet-stream", buffer: floppy });
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.locator("#editor")).not.toBeVisible();
    await page.locator("#start").click();
    await expect(page.locator("#screen canvas")).toBeVisible();
    await expect.poll(() => page.locator("#screen canvas").evaluate(canvas => {
        const pixel = canvas.getContext("2d").getImageData(160, 100, 1, 1).data;
        return canvas.width === 320 && canvas.height === 200 && pixel[0] + pixel[1] + pixel[2] > 0;
    })).toBe(true);
});
