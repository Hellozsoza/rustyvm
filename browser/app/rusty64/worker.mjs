// SPDX-License-Identifier: GPL-3.0-only
// The interpreter runs in a worker so untrusted guest loops cannot block the UI.
let engine;
let running = false;
let scheduled = false;
let ready = false;
const decoder = new TextDecoder();

function errorText() {
    return decoder.decode(new Uint8Array(engine.memory.buffer, engine.engine_error_ptr(), engine.engine_error_len()));
}

function snapshot(state) {
    const names = ["RAX", "RCX", "RDX", "RBX", "RSP", "RBP", "RSI", "RDI", "R8", "R9", "R10", "R11", "R12", "R13", "R14", "R15", "RIP", "RFLAGS"];
    const value = index => BigInt(engine.engine_register(index, 0) >>> 0) | BigInt(engine.engine_register(index, 1) >>> 0) << 32n;
    const length = engine.engine_output_len();
    const output = decoder.decode(new Uint8Array(engine.memory.buffer, engine.engine_output_ptr(), length), { stream: true });
    engine.engine_clear_output();
    postMessage({ type: "state", state, output,
        registers: Object.fromEntries(names.map((name, index) => [name, value(index).toString(16).padStart(16, "0")])),
        instructions: value(18).toString(), committed: Number(value(19)),
        exitCode: engine.engine_exit_code(), error: state === "Fault" ? errorText() : "" });
}

function execute(budget) {
    const code = engine.engine_run(budget);
    if (code !== 0) { running = false; ready = false; }
    snapshot([running ? "Running" : "Paused", "Halted", "Exited", "Fault"][code]);
}

function schedule() {
    if (scheduled || !running) return;
    scheduled = true;
    setTimeout(() => {
        scheduled = false;
        if (!running) return;
        try { execute(10000); schedule(); }
        catch (error) { running = false; ready = false; postMessage({ type: "error", error: `Engine stopped: ${error.message}` }); }
    }, 16);
}

const initialized = (async () => {
    const response = await fetch("./rusty64.wasm");
    if (!response.ok) throw new Error(`Could not load Rust engine: HTTP ${response.status}`);
    const result = await WebAssembly.instantiate(await response.arrayBuffer(), {});
    engine = result.instance.exports;
})();

self.onmessage = async ({ data }) => {
    try {
        await initialized;
        switch (data.type) {
            case "load": {
                running = false;
                ready = false;
                if (!Number.isInteger(data.memory) || data.memory < 16 || data.memory > 8192)
                    throw new Error("Address space must be between 16 and 8192 MiB.");
                if (data.bytes) {
                    const pointer = engine.engine_input(data.bytes.byteLength);
                    if (!pointer) throw new Error(errorText());
                    new Uint8Array(engine.memory.buffer, pointer, data.bytes.byteLength).set(new Uint8Array(data.bytes));
                }
                if (engine.engine_load(data.memory, data.bytes ? 0 : 1)) throw new Error(errorText());
                ready = true;
                decoder.decode();
                postMessage({ type: "loaded" });
                snapshot("Ready");
                break;
            }
            case "run":
                if (!ready) throw new Error("Load or reset an executable before running.");
                running = true; schedule(); break;
            case "pause": running = false; snapshot("Paused"); break;
            case "step":
                if (!ready) throw new Error("Load or reset an executable before stepping.");
                running = false; execute(1); break;
        }
    } catch (error) {
        running = false;
        postMessage({ type: "error", error: error.message });
    }
};
