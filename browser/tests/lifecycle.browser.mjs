/* $Id$ | Actual WASM boot, input, storage, snapshot and lifecycle tests | SPDX-License-Identifier: GPL-3.0-only */
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

async function boot(page) {
    await page.getByRole("button", { name: "Start", exact: true }).click();
    await expect(page.locator("#serial")).toHaveValue(/real x86 guest running in WebAssembly/, { timeout: 30000 });
    await expect(page.locator("#console-title")).toContainText("Running");
    await expect(page.locator("#screen")).toContainText("real x86 guest");
}

async function type(page, text) {
    await page.getByRole("textbox", { name: "Text to type in guest" }).fill(text);
    await page.getByRole("button", { name: "Send to guest" }).click();
    await expect(page.locator("#serial")).toHaveValue(new RegExp(text), { timeout: 10000 });
}

test("actual WASM boot, keyboard, pause, snapshot, save/reload/resume and restore", async ({ page }) => {
    const errors = [];
    page.on("pageerror", error => errors.push(error.message));
    page.on("dialog", dialog => dialog.type() === "prompt" ? dialog.accept("Working snapshot") : dialog.accept());
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    await boot(page);
    await page.locator("#screen").click();
    await page.keyboard.type("physical", { delay: 20 });
    await expect(page.locator("#serial")).toHaveValue(/physical/);
    await type(page, "hello");
    await page.getByRole("button", { name: "Pause", exact: true }).click();
    await expect(page.locator("#console-title")).toContainText("Paused");
    await page.getByRole("button", { name: "Resume", exact: true }).click();
    await expect(page.locator("#console-title")).toContainText("Running");
    await page.getByRole("button", { name: "Take Snapshot", exact: true }).click();
    await expect(page.locator("#status")).toHaveText("Snapshot saved locally.");
    await type(page, "after");
    await page.getByRole("button", { name: "Save State & Close" }).click();
    await expect(page.locator("#machines")).toContainText("Saved");
    await page.reload();
    await expect(page.locator("#machines")).toContainText("Saved");
    await page.getByRole("button", { name: "Start", exact: true }).click();
    await expect(page.locator("#console-title")).toContainText("Running");
    await expect(page.locator("#screen")).toContainText("helloafter");
    await type(page, "resume");
    await page.getByRole("button", { name: "Power Off", exact: true }).click();
    await expect(page.locator("#machines")).toContainText("Powered Off");
    await page.getByRole("tab", { name: "Snapshots", exact: true }).click();
    await page.getByRole("button", { name: "Restore", exact: true }).click();
    await expect(page.locator("#status")).toContainText("Snapshot restored");
    await page.getByRole("button", { name: "Start", exact: true }).click();
    await expect(page.locator("#console-title")).toContainText("Running");
    await expect(page.locator("#screen")).toContainText("hello");
    await expect(page.locator("#screen")).not.toContainText("helloafter");
    await type(page, "restored");
    await page.getByRole("button", { name: "Power Off", exact: true }).click();
    expect(errors).toEqual([]);
});

test("create, settings, clone and delete persist; guest names cannot inject markup", async ({ page }) => {
    page.on("dialog", dialog => dialog.accept());
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    await page.getByRole("button", { name: "New", exact: true }).click();
    await page.locator('[name="name"]').fill('<img src=x onerror=alert(1)>');
    await page.locator('[name="memory"]').fill("48");
    await page.locator('[name="blank"]').fill("2");
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.locator("#machines")).toContainText('<img src=x onerror=alert(1)>');
    expect(await page.locator("#machines img").count()).toBe(2);
    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await page.locator('[name="name"]').fill("Local guest");
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await page.getByRole("button", { name: "Clone", exact: true }).click();
    await expect(page.locator("#machines")).toContainText("Local guest Clone");
    await page.reload();
    await expect(page.locator("#machines")).toContainText("Local guest Clone");
    await page.getByRole("option", { name: /Local guest Clone/ }).click();
    await page.getByRole("button", { name: "Remove", exact: true }).click();
    await expect(page.locator("#machines")).not.toContainText("Local guest Clone");
});

test("missing runtime files fail visibly and leave the manager usable", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    await page.route("**/vendor/v86.wasm", route => route.fulfill({ status: 404, body: "missing" }));
    await page.getByRole("button", { name: "Start", exact: true }).click();
    await expect(page.locator("#status")).toContainText("HTTP 404");
    await expect(page.getByRole("button", { name: "Start", exact: true })).toBeEnabled();
    await page.unroute("**/vendor/v86.wasm");
    await boot(page);
});

test("second tab cannot overwrite a running manager’s storage", async ({ page, context }) => {
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    const second = await context.newPage();
    await second.goto("/");
    await expect(second.locator("#status")).toContainText("Another manager tab is open");
    await expect(second.getByRole("button", { name: "Start", exact: true })).toBeDisabled();
});
