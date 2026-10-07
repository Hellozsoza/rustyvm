// SPDX-License-Identifier: GPL-3.0-only
const $ = id => document.getElementById(id);
const worker = new Worker(new URL("./worker.mjs", import.meta.url), { type: "module" });
let currentBytes = null;
let currentName = "Built-in demo";
let loadedMemory = 8192;
let pending = false;

function controls(ready, running = false) {
    $("run").disabled = !ready || running;
    $("step").disabled = !ready || running;
    $("pause").disabled = !running;
    $("reset").disabled = pending;
    $("demo").disabled = pending;
    $("file").disabled = pending;
}

function load(bytes, name, memory = Number($("memory").value)) {
    if (!Number.isInteger(memory) || memory < 16 || memory > 8192) {
        $("status").textContent = "Address space must be an integer between 16 and 8192 MiB.";
        return;
    }
    currentBytes = bytes;
    currentName = name;
    loadedMemory = memory;
    pending = true;
    controls(false);
    $("status").textContent = `Loading ${name}…`;
    $("output").textContent = "";
    $("registers").replaceChildren();
    $("metrics").textContent = "";
    worker.postMessage({ type: "load", memory, bytes });
}

worker.onmessage = ({ data }) => {
    if (data.type === "loaded") { pending = false; return; }
    if (data.type === "error") {
        pending = false;
        controls(false);
        $("status").textContent = data.error;
        return;
    }
    if (data.type !== "state") return;
    if (data.output) $("output").textContent = ($("output").textContent + data.output).slice(-1024 * 1024);
    controls(["Ready", "Running", "Paused"].includes(data.state), data.state === "Running");
    $("status").textContent = data.state === "Fault" ? data.error
        : `${currentName}: ${data.state}${data.state === "Exited" ? ` (code ${data.exitCode})` : ""}`;
    $("metrics").textContent = `${data.instructions} instructions · ${(data.committed / 1024).toFixed(0)} KiB committed · ${loadedMemory} MiB address space`;
    $("registers").replaceChildren(...Object.entries(data.registers).flatMap(([name, value]) => {
        const label = document.createElement("dt"), content = document.createElement("dd");
        label.textContent = name;
        content.textContent = `0x${value}`;
        return [label, content];
    }));
};
worker.onerror = () => {
    pending = false;
    controls(false);
    $("status").textContent = "Rust engine worker failed. Reload the page to restart it.";
};
$("demo").onclick = () => load(null, "Built-in demo");
$("reset").onclick = () => load(currentBytes, currentName, loadedMemory);
$("file").onchange = async () => {
    const file = $("file").files[0];
    if (!file) return;
    if (file.size === 0 || file.size > 16 * 1024 * 1024) {
        $("status").textContent = "Choose an ELF64 executable between 1 byte and 16 MiB.";
        return;
    }
    try { load(await file.arrayBuffer(), file.name); }
    catch (error) { $("status").textContent = `Could not read executable: ${error.message}`; }
    $("file").value = "";
};
for (const action of ["run", "pause", "step"]) $(action).onclick = () => worker.postMessage({ type: action });
load(null, "Built-in demo");
