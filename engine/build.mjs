// SPDX-License-Identifier: GPL-3.0-only
// Builds both the guest machine code and the Rust WebAssembly engine from source.
import { execFileSync } from "node:child_process";
import { mkdir, mkdtemp, readFile, writeFile, copyFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";

const root = fileURLToPath(new URL("./", import.meta.url));
const destination = fileURLToPath(new URL("../browser/app/rusty64/", import.meta.url));
const temporary = await mkdtemp(join(tmpdir(), "rusty64-"));
try {
    execFileSync("as", ["--64", "guests/demo.S", "-o", `${temporary}/demo.o`], { cwd: root, stdio: "inherit" });
    execFileSync("ld", ["-m", "elf_x86_64", "--build-id=none", "-z", "max-page-size=4096", "-o", "guests/demo.elf", `${temporary}/demo.o`], { cwd: root, stdio: "inherit" });
    execFileSync("cargo", ["build", "--locked", "--offline", "--release", "--target", "wasm32-unknown-unknown"], { cwd: root, stdio: "inherit" });
    await mkdir(destination, { recursive: true });
    await copyFile(`${root}target/wasm32-unknown-unknown/release/rusty64.wasm`, `${destination}rusty64.wasm`);
    await copyFile(`${root}guests/demo.elf`, `${destination}demo.elf`);
    const sha256 = data => createHash("sha256").update(data).digest("hex");
    const sources = {};
    for (const file of ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src/lib.rs", "src/memory.rs", "src/elf.rs", "src/wasm.rs", "guests/demo.S", "build.mjs"])
        sources[file] = sha256(await readFile(`${root}${file}`));
    const artifacts = {};
    for (const file of ["rusty64.wasm", "demo.elf"])
        artifacts[file] = sha256(await readFile(`${destination}${file}`));
    await writeFile(`${destination}manifest.json`, JSON.stringify({ engine: "rusty64", version: "0.1.0", rust: "1.90.0", artifacts, sources }, null, 2) + "\n");
    console.log("Compiled x86-64 demo and Rust WASM engine; updated browser artifacts and source checksums.");
} finally { await rm(temporary, { recursive: true, force: true }); }
