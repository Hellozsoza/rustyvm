/* $Id$ */
/** @file @page pg_browser Browser VM manager, storage, and lifecycle UI. */
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
import { validateConfig, readMedia, ENGINE_VERSION, MAX_MEDIA_BYTES } from "./model.mjs";
import { MachineStore } from "./storage.mjs";
import { MachineRuntime } from "./runtime.mjs";

const $ = id => document.getElementById(id);
const store = new MachineStore();
let machines = [], selectedId, editingId, busy = false, activeState = "Powered Off";
const runtime = new MachineRuntime($("screen"), $("serial"), state => { activeState = state; render(); });
const selected = () => machines.find(machine => machine.id === selectedId);
const active = () => machines.find(machine => machine.id === runtime.machineId);
const stateOf = machine => machine.id === runtime.machineId ? activeState : machine.saved ? "Saved" : "Powered Off";

function element(tag, text, className) {
    const node = document.createElement(tag);
    if (text !== undefined)
        node.textContent = text;
    if (className)
        node.className = className;
    return node;
}

function button(text, action, disabled = false) {
    const node = element("button", text);
    node.disabled = disabled || busy;
    node.addEventListener("click", () => perform(action));
    return node;
}

function status(message) { $("status").textContent = message; }

async function perform(action) {
    if (busy)
        return;
    busy = true;
    render();
    try { await action(); }
    catch (error) { status(`Error: ${error.message}`); }
    finally { busy = false; render(); }
}

async function refresh() {
    machines = await store.list();
    if (!selected())
        selectedId = machines[0]?.id;
    render();
}

function tab(name) {
    for (const id of ["details", "console", "snapshots"]) {
        $(id).hidden = id !== name;
        $(`tab-${id}`).setAttribute("aria-selected", String(id === name));
    }
    runtime.focus(false);
}

function section(title, rows) {
    const card = element("section", undefined, "detail-card");
    card.append(element("h2", title));
    const list = element("dl");
    for (const [key, value] of rows)
        list.append(element("dt", key), element("dd", value));
    card.append(list);
    return card;
}

function download(data, name) {
    const url = URL.createObjectURL(new Blob([data], { type: "application/octet-stream" }));
    const link = element("a");
    link.href = url;
    link.download = name.replace(/[\\/:*?"<>|]/g, "_");
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function render() {
    const machine = selected();
    const locked = !machine || !!runtime.emulator || busy;
    for (const id of ["settings", "menu-settings", "clone", "remove"])
        $(id).disabled = locked;
    $("start").disabled = locked;
    $("new").disabled = $("menu-new").disabled = busy;
    $("snapshot").disabled = busy || !runtime.emulator || !active() || selectedId !== runtime.machineId;
    for (const id of ["pause", "reset", "save", "poweroff", "cad", "guest-text"])
        $(id).disabled = busy || !runtime.emulator || activeState === "Starting";
    $("send-keys").querySelector("button").disabled = $("guest-text").disabled;
    $("pause").textContent = activeState === "Paused" ? "Resume" : "Pause";
    $("console-title").textContent = active() ? `${active().config.name} — ${activeState}` : "Guest display";
    $("machines").replaceChildren();
    for (const entry of machines) {
        const row = element("button", undefined, entry.id === selectedId ? "machine selected" : "machine");
        row.setAttribute("role", "option");
        row.setAttribute("aria-selected", String(entry.id === selectedId));
        const icon = element("img");
        icon.src = `./icons/${entry.config.os === "Linux" ? "os_linux" : entry.config.os === "Windows" ? "os_win2k" : "os_other"}.png`;
        icon.alt = "";
        const text = element("span");
        text.append(element("strong", entry.config.name), element("small", stateOf(entry)));
        row.append(icon, text);
        row.disabled = busy;
        row.onclick = () => { selectedId = entry.id; render(); };
        $("machines").append(row);
    }
    $("details").replaceChildren();
    $("snapshots").replaceChildren();
    if (!machine) {
        $("details").append(element("h1", "Welcome to VirtualBox Browser"), element("p", "Create a machine to boot a local image or try the built-in x86 demo."));
        return;
    }
    $("details").append(
        section("General", [["Name", machine.config.name], ["Operating System", `${machine.config.os} (16/32-bit x86)`], ["State", stateOf(machine)]]),
        section("System", [["Base Memory", `${machine.config.memory} MB`], ["Processor", "1 × emulated x86 (v86 WebAssembly)"], ["Boot Device", machine.config.boot]]),
        section("Display", [["Graphics Controller", "VGA / Bochs VBE"], ["Video Memory", "8 MB"]]),
        section("Storage", ["floppy", "cdrom", "disk"].map(kind => [kind, machine.media[kind] ? `${machine.media[kind].name} (${(machine.media[kind].data.byteLength / 1048576).toFixed(1)} MB)` : kind === "floppy" && machine.config.boot === "demo" ? "Built-in bootable demo" : "Not attached"])),
        section("Network", [["Adapter", "Disconnected — browser networking is not configured"]])
    );
    const actions = element("div", undefined, "detail-actions");
    if (machine.media.disk)
        actions.append(button("Export hard disk (raw IMG)", async () => {
            const current = machine.id === runtime.machineId ? runtime.currentMedia(machine) : machine.media;
            download(current.disk.data, `${machine.config.name}.img`);
        }, !!runtime.emulator));
    if (machine.saved) {
        actions.append(button("Export saved state", () => download(machine.saved.state, `${machine.config.name}.v86state`)));
        actions.append(button("Discard saved state", async () => {
            if (!confirm("Discard this saved CPU and memory state? The hard disk is kept."))
                return;
            const changed = { ...machine, saved: null };
            await store.put(changed); await refresh(); status("Saved state discarded.");
        }, !!runtime.emulator));
    }
    const importLabel = element("label", "Import v86 state (same engine, memory and disks required): ", "import-state");
    const importFile = element("input");
    importFile.type = "file"; importFile.accept = ".v86state"; importFile.disabled = !!runtime.emulator || busy;
    importFile.onchange = () => perform(async () => {
        const file = importFile.files[0];
        if (!file)
            return;
        if (!file.size || file.size > 768 * 1024 * 1024)
            throw new Error("State file must contain data and be at most 768 MB.");
        const state = await file.arrayBuffer();
        if (new DataView(state).getUint32(0, true) !== 0x86768676)
            throw new Error("Not a v86 saved state; native VirtualBox states cannot be imported.");
        await store.put({ ...machine, saved: { version: ENGINE_VERSION, state, created: Date.now() } });
        await refresh(); status("State imported. Start restores it with this VM’s current media and memory settings.");
    });
    importLabel.append(importFile); actions.append(importLabel);
    $("details").append(actions);
    $("snapshots").append(element("h2", `${machine.config.name} — Snapshots`),
        element("p", "Snapshots include CPU, memory, configuration, and disk contents. Up to five snapshots per VM; restoring replaces current disk contents."));
    for (const snapshot of machine.snapshots || []) {
        const row = element("div", undefined, "snapshot-row");
        row.append(element("strong", snapshot.name), element("span", new Date(snapshot.created).toLocaleString()));
        row.append(button("Restore", async () => {
            if (!confirm("Restore this snapshot and replace the current VM configuration and disks?"))
                return;
            if (snapshot.version !== ENGINE_VERSION)
                throw new Error("Snapshot engine version differs from this build.");
            await store.put({ ...machine, config: snapshot.config, media: snapshot.media,
                saved: { version: snapshot.version, state: snapshot.state, created: snapshot.created } });
            await refresh(); status("Snapshot restored. Start to resume it.");
        }, !!runtime.emulator));
        row.append(button("Delete", async () => {
            await store.put({ ...machine, snapshots: machine.snapshots.filter(entry => entry.id !== snapshot.id) });
            await refresh(); status("Snapshot deleted.");
        }, !!runtime.emulator));
        $("snapshots").append(row);
    }
}

function editor(isNew) {
    const machine = isNew ? null : selected();
    if (!isNew && (!machine || runtime.emulator))
        return;
    if (machine?.saved) {
        status("Discard the saved state before changing the VM’s hardware configuration.");
        return;
    }
    editingId = machine?.id;
    $("config-form").reset();
    const config = machine?.config || { name: "New machine", os: "Other", memory: 64, boot: "demo" };
    for (const [key, value] of Object.entries(config))
        $("config-form").elements[key].value = value;
    $("editor-title").textContent = isNew ? "Create Virtual Machine" : "Virtual Machine Settings";
    $("form-error").textContent = "";
    runtime.focus(false);
    $("editor").showModal();
}

$("config-form").onsubmit = event => {
    event.preventDefault();
    perform(async () => {
        const form = event.target;
        form.querySelector('button[type="submit"]').disabled = true;
        try {
            const config = validateConfig({ name: form.elements.name.value, os: form.elements.os.value,
                memory: Number(form.elements.memory.value), boot: form.elements.boot.value });
            const original = editingId ? await store.get(editingId) : null;
            const media = structuredClone(original?.media || {});
            for (const kind of ["floppy", "cdrom", "disk"])
                if (form.elements[kind].files[0])
                    media[kind] = await readMedia(form.elements[kind].files[0], kind);
            const blank = Number(form.elements.blank.value);
            if (!Number.isInteger(blank) || blank < 0 || blank * 1048576 > MAX_MEDIA_BYTES)
                throw new Error("Blank disk size must be an integer from 0 to 2048 MB.");
            if (blank && form.elements.disk.files[0])
                throw new Error("Choose either a disk image or a new blank disk.");
            if (blank)
                media.disk = { name: "Blank disk.img", data: new ArrayBuffer(blank * 1048576) };
            if (config.boot !== "demo" && !media[config.boot])
                throw new Error("Attach an image for the selected boot device.");
            const id = original?.id || crypto.randomUUID();
            await store.put({ id, config, media, saved: null, snapshots: original?.snapshots || [] });
            selectedId = id;
            $("editor").close();
            await refresh(); status("Machine saved locally.");
        } catch (error) { $("form-error").textContent = error.message; }
        finally { form.querySelector('button[type="submit"]').disabled = false; }
    });
};

for (const id of ["new", "menu-new"])
    $(id).onclick = () => editor(true);
for (const id of ["settings", "menu-settings"])
    $(id).onclick = () => editor(false);
$("cancel").onclick = () => { if (!busy) $("editor").close(); };
$("editor").addEventListener("cancel", event => { if (busy) event.preventDefault(); });
$("about").onclick = () => { runtime.focus(false); $("compatibility").showModal(); };
$("close-about").onclick = () => $("compatibility").close();
for (const name of ["details", "console", "snapshots"])
    $(`tab-${name}`).onclick = () => tab(name);
$("clone").onclick = () => perform(async () => {
    const machine = structuredClone(selected());
    machine.id = crypto.randomUUID(); machine.config.name = `${machine.config.name.slice(0, 110)} Clone`;
    machine.saved = null; machine.snapshots = [];
    await store.put(machine); selectedId = machine.id; await refresh(); status("Independent machine and disk copy created.");
});
$("remove").onclick = () => perform(async () => {
    const machine = selected();
    if (!confirm(`Permanently remove “${machine.config.name}” and its local disks and snapshots?`))
        return;
    await store.remove(machine.id); await refresh(); status("Machine removed.");
});
$("start").onclick = () => perform(async () => {
    const machine = selected();
    activeState = "Starting"; tab("console"); status("Loading WebAssembly and booting guest…");
    await runtime.start(machine); render(); status("Guest running locally in WebAssembly.");
});
$("pause").onclick = () => perform(() => runtime.pause());
$("reset").onclick = () => perform(async () => {
    if (confirm("Reset the guest? Unsaved guest work can be lost."))
        runtime.emulator.restart();
});
$("poweroff").onclick = () => perform(async () => {
    if (!confirm("Power off the guest? Disk writes are kept, but unsaved memory is lost."))
        return;
    const machine = active();
    const wasRunning = runtime.emulator.is_running();
    await runtime.emulator.stop(); runtime.focus(false);
    try {
        await store.put({ ...machine, media: runtime.currentMedia(machine), saved: null });
    } catch (error) {
        if (wasRunning) await runtime.emulator.run();
        throw error;
    }
    await runtime.destroy(); await refresh(); status("Powered off. Disk writes saved locally.");
});
$("save").onclick = () => perform(async () => {
    const machine = active();
    const saved = await runtime.capture(machine);
    await store.put({ ...machine, media: saved.media, saved: { version: saved.version, state: saved.state, created: saved.created } });
    await runtime.destroy(); await refresh(); status("CPU, memory, and disk state saved. Start to resume.");
});
$("snapshot").onclick = () => perform(async () => {
    const machine = active();
    if (machine.snapshots.length >= 5)
        throw new Error("Delete a snapshot first; the limit is five per VM.");
    const name = prompt("Snapshot name", `Snapshot ${machine.snapshots.length + 1}`);
    if (!name)
        return;
    const snapshot = { ...await runtime.capture(machine), id: crypto.randomUUID(), name: name.slice(0, 120) };
    await store.put({ ...machine, snapshots: [...machine.snapshots, snapshot] });
    await refresh(); status("Snapshot saved locally.");
});
$("fullscreen").onclick = () => perform(async () => {
    await $("screen").requestFullscreen(); $("screen").focus(); runtime.focus(true);
});
$("cad").onclick = () => perform(async () => {
    await runtime.emulator.keyboard_send_scancodes([0x1d, 0x38, 0xe0, 0x53, 0xe0, 0xd3, 0xb8, 0x9d]);
});
$("screen").onfocus = () => runtime.focus(activeState === "Running");
$("screen").onblur = () => runtime.focus(false);
$("screen").onpointerdown = () => $("screen").focus();
document.addEventListener("keydown", event => {
    if (event.key === "Escape") { runtime.focus(false); $("screen").blur(); }
}, true);
$("send-keys").onsubmit = event => {
    event.preventDefault();
    perform(async () => {
        if (activeState !== "Running")
            throw new Error("Resume the guest before sending keys.");
        runtime.emulator.keyboard_set_enabled(true);
        try { await runtime.emulator.keyboard_send_text($("guest-text").value, 20); }
        finally { runtime.focus(false); }
        $("guest-text").value = "";
    });
};
window.addEventListener("beforeunload", event => {
    if (runtime.emulator) { event.preventDefault(); event.returnValue = ""; }
});

async function initialize() {
    try {
        await store.open();
        machines = await store.list();
        if (!machines.length) {
            await store.put({ id: crypto.randomUUID(), config: { name: "WebAssembly Demo", os: "Other", memory: 32, boot: "demo" }, media: {}, saved: null, snapshots: [] });
        }
        await refresh(); status("Ready. Start the demo or create a VM with local boot media.");
    } catch (error) { status(`Storage unavailable: ${error.message}`); document.querySelectorAll("button").forEach(node => node.disabled = true); }
}

// Hold an origin-wide lock: two manager tabs must not overwrite the same disks.
if (navigator.locks) {
    navigator.locks.request("virtualbox-browser-manager", { ifAvailable: true }, async lock => {
        if (!lock) {
            status("Another manager tab is open. Close it and reload this tab.");
            document.querySelectorAll("button").forEach(node => node.disabled = true);
            return;
        }
        await initialize();
        await new Promise(() => {});
    });
} else {
    status("This browser needs Web Locks support on a secure origin (HTTPS or localhost).");
    document.querySelectorAll("button").forEach(node => node.disabled = true);
}
