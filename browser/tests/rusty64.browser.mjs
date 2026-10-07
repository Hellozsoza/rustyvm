import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";

function executable(code) {
    // Minimal executable ELF64 segment, mapped at 0x1000, entry 0x1100.
    const data = Buffer.alloc(256 + code.length);
    data.set([0x7f, 0x45, 0x4c, 0x46, 2, 1, 1]);
    data.writeUInt16LE(2, 16); data.writeUInt16LE(62, 18); data.writeUInt32LE(1, 20);
    data.writeBigUInt64LE(0x1100n, 24); data.writeBigUInt64LE(64n, 32);
    data.writeUInt16LE(64, 52); data.writeUInt16LE(56, 54); data.writeUInt16LE(1, 56);
    data.writeUInt32LE(1, 64); data.writeUInt32LE(5, 68);
    data.writeBigUInt64LE(0x1000n, 80);
    data.writeBigUInt64LE(BigInt(data.length), 96); data.writeBigUInt64LE(BigInt(data.length), 104);
    data.writeBigUInt64LE(4096n, 112);
    data.set(code, 256);
    return data;
}

test("Rust WASM executes compiled ELF64 demo and addresses memory above 4 GiB", async ({ page }) => {
    const errors = [];
    page.on("pageerror", error => errors.push(error.message));
    await page.goto("/");
    await page.getByRole("link", { name: "Rust x86-64 engine (experimental)", exact: true }).click();
    await expect(page.locator("#status")).toContainText("Ready");
    await page.getByRole("button", { name: "Step", exact: true }).click();
    await expect(page.locator("#metrics")).toContainText("1 instructions");
    await expect(page.locator("#registers")).toContainText("0x0000000100000001");
    await page.getByRole("button", { name: "Run", exact: true }).click();
    await expect(page.locator("#status")).toContainText("Exited (code 0)");
    await expect(page.locator("#output")).toContainText("64-bit arithmetic and memory above 4 GiB: OK");
    await expect(page.locator("#registers")).toContainText("0x0000000300000003");
    await page.getByRole("button", { name: "Reset", exact: true }).click();
    await expect(page.locator("#status")).toContainText("Ready");
    await expect(page.locator("#output")).toBeEmpty();
    expect(errors).toEqual([]);
});

test("loads user ELF64 locally and reports malformed or unsupported executables", async ({ page }) => {
    await page.goto("/rusty64/");
    await expect(page.locator("#status")).toContainText("Ready");
    const binary = await readFile(new URL("../../engine/guests/demo.elf", import.meta.url));
    await page.locator("#file").setInputFiles({ name: "guest.elf", mimeType: "application/octet-stream", buffer: binary });
    await expect(page.locator("#status")).toContainText("guest.elf: Ready");
    await page.getByRole("button", { name: "Run", exact: true }).click();
    await expect(page.locator("#status")).toContainText("Exited (code 0)");
    await page.locator("#file").setInputFiles({ name: "bad.elf", mimeType: "application/octet-stream", buffer: Buffer.from("not an ELF") });
    await expect(page.locator("#status")).toContainText("Expected a little-endian ELF64");
    await expect(page.locator("#run")).toBeDisabled();
    await page.locator("#file").setInputFiles({ name: "unsupported.elf", mimeType: "application/octet-stream", buffer: executable([0x0f, 0x0b]) });
    await expect(page.locator("#status")).toContainText("unsupported.elf: Ready");
    await page.getByRole("button", { name: "Run", exact: true }).click();
    await expect(page.locator("#status")).toContainText("Unsupported opcode 0f 0b at RIP 0x0000000000001100");
});

test("infinite guest yields, pauses, resets and does not block the page", async ({ page }) => {
    await page.goto("/rusty64/");
    await expect(page.locator("#status")).toContainText("Ready");
    await page.locator("#file").setInputFiles({ name: "loop.elf", mimeType: "application/octet-stream", buffer: executable([0xeb, 0xfe]) });
    await expect(page.locator("#status")).toContainText("loop.elf: Ready");
    await page.getByRole("button", { name: "Run", exact: true }).click();
    await expect(page.locator("#status")).toContainText("Running");
    await page.getByRole("button", { name: "Pause", exact: true }).click();
    await expect(page.locator("#status")).toContainText("Paused");
    const metrics = await page.locator("#metrics").textContent();
    await page.getByRole("button", { name: "Step", exact: true }).click();
    await expect(page.locator("#metrics")).not.toHaveText(metrics);
    await page.getByRole("button", { name: "Reset", exact: true }).click();
    await expect(page.locator("#metrics")).toContainText("0 instructions");
});

test("demo fails explicitly when its high guest address is outside selected memory", async ({ page }) => {
    await page.goto("/rusty64/");
    await expect(page.locator("#status")).toContainText("Ready");
    await page.locator("#memory").fill("16");
    await page.getByRole("button", { name: "Load 64-bit demo", exact: true }).click();
    await expect(page.locator("#metrics")).toContainText("16 MiB address space");
    await page.getByRole("button", { name: "Run", exact: true }).click();
    await expect(page.locator("#status")).toContainText("Guest memory access out of bounds at 0x100002000");
});
