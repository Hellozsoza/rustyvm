import { test, expect } from "@playwright/test";

test("cookie catalog preserves settings without binary data and survives IndexedDB removal", async ({ page, context }) => {
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await page.locator('[name="memory"]').fill("8192");
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await expect(page.locator("#details")).toContainText("8192 MB");
    const cookies = await context.cookies();
    const catalog = JSON.parse(decodeURIComponent(cookies.find(cookie => cookie.name === "rustyvm_machines_v1").value));
    expect(catalog).toHaveLength(1);
    expect(catalog[0].config.memory).toBe(8192);
    expect(Object.keys(catalog[0]).sort()).toEqual(["config", "id"]);
    await page.reload();
    await expect(page.locator("#details")).toContainText("8192 MB");
    await page.getByRole("button", { name: "Start", exact: true }).click();
    await expect(page.locator("#status")).toContainText("at most 2047 MB");
    await expect(page.getByRole("button", { name: "Start", exact: true })).toBeEnabled();
    await page.evaluate(async () => {
        const { MachineStore } = await import("/storage.mjs");
        const store = new MachineStore();
        await store.open();
        await store.transaction("readwrite", records => records.clear());
        store.db.close();
    });
    await page.reload();
    await expect(page.locator("#details")).toContainText("8192 MB");
});

test("cookie capacity failures leave previous records intact", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    const result = await page.evaluate(async () => {
        const { MachineStore } = await import("/storage.mjs");
        const store = new MachineStore();
        await store.open();
        const original = (await store.list())[0];
        let failed;
        for (let i = 0; i < 100; i++) {
            try {
                await store.put({ ...original, id: `capacity-${i}`, config: { ...original.config, name: "長".repeat(120) } });
            } catch (error) { failed = { id: `capacity-${i}`, message: error.message }; break; }
        }
        const payload = await store.transaction("readonly", records => records.get(failed.id));
        const intact = await store.get(original.id);
        const rows = await store.list();
        store.db.close();
        return { failed, payloadExists: !!payload, intact: intact.config.name === original.config.name, containsFailed: rows.some(row => row.id === failed.id) };
    });
    expect(result.failed.message).toContain("cookie capacity");
    expect(result.payloadExists).toBe(false);
    expect(result.containsFailed).toBe(false);
    expect(result.intact).toBe(true);
});

test("legacy IndexedDB catalog migrates with media preserved", async ({ page, context }) => {
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    await context.clearCookies();
    await page.evaluate(async () => {
        const { MachineStore } = await import("/storage.mjs");
        const store = new MachineStore();
        await store.open();
        await store.transaction("readwrite", records => records.clear());
        await store.transaction("readwrite", records => records.put({ id: "legacy", config: { name: "Legacy VM", memory: 64, os: "Other", boot: "demo" }, media: { disk: { name: "disk.img", data: new Uint8Array([1, 2, 3]).buffer } }, saved: null, snapshots: [] }));
        store.db.close();
    });
    await page.reload();
    await expect(page.locator("#machines")).toContainText("Legacy VM");
    const result = await page.evaluate(async () => {
        const { MachineStore } = await import("/storage.mjs");
        const store = new MachineStore();
        await store.open();
        const vm = await store.get("legacy");
        const payload = await store.transaction("readonly", records => records.get("legacy"));
        store.db.close();
        return { bytes: [...new Uint8Array(vm.media.disk.data)], hasConfig: "config" in payload };
    });
    expect(result.bytes).toEqual([1, 2, 3]);
    expect(result.hasConfig).toBe(false);
});

test("failed binary persistence rolls back settings and removal in cookies", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#status")).toContainText("Ready.");
    const result = await page.evaluate(async () => {
        const { MachineStore } = await import("/storage.mjs");
        const store = new MachineStore();
        await store.open();
        const original = (await store.list())[0];
        store.transaction = async () => { throw new Error("Storage quota exceeded"); };
        let writeError, removeError;
        try { await store.put({ ...original, config: { ...original.config, name: "Changed" } }); }
        catch (error) { writeError = error.message; }
        const afterWrite = store.catalog();
        try { await store.remove(original.id); }
        catch (error) { removeError = error.message; }
        const afterRemove = store.catalog();
        store.db.close();
        return { writeError, removeError, afterWrite, afterRemove, original: { id: original.id, config: original.config } };
    });
    expect(result.writeError).toBe("Storage quota exceeded");
    expect(result.removeError).toBe("Storage quota exceeded");
    expect(result.afterWrite).toEqual([result.original]);
    expect(result.afterRemove).toEqual([result.original]);
});
