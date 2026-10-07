/* $Id$ */
/** @file Cookie VM catalog with IndexedDB media and execution-state persistence. */
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
import { validateConfig } from "./model.mjs";

const CATALOG_COOKIE = "rustyvm_machines_v1";
// Stay below browsers' 4 KB per-cookie limit, including attributes.
const MAX_COOKIE_BYTES = 3800;

export class MachineStore {
    catalog() {
        const entry = document.cookie.split(";").map(part => part.trim())
            .find(part => part.startsWith(`${CATALOG_COOKIE}=`));
        if (!entry) return [];
        const rows = JSON.parse(decodeURIComponent(entry.slice(CATALOG_COOKIE.length + 1)));
        if (!Array.isArray(rows)) throw new Error("Invalid VM catalog cookie.");
        const ids = new Set();
        return rows.map(row => {
            if (typeof row.id !== "string" || !/^[a-zA-Z0-9-]{1,64}$/.test(row.id) || ids.has(row.id))
                throw new Error("Invalid VM identifier in catalog cookie.");
            ids.add(row.id);
            return { id: row.id, config: validateConfig(row.config) };
        });
    }

    writeCatalog(rows) {
        const value = encodeURIComponent(JSON.stringify(rows));
        if (CATALOG_COOKIE.length + value.length > MAX_COOKIE_BYTES)
            throw new Error("VM list exceeds cookie capacity. Shorten VM names or remove a VM before saving.");
        document.cookie = `${CATALOG_COOKIE}=${value}; Path=/; Max-Age=31536000; SameSite=Strict${location.protocol === "https:" ? "; Secure" : ""}`;
        if (!document.cookie.split(";").some(part => part.trim() === `${CATALOG_COOKIE}=${value}`))
            throw new Error("Browser refused the VM catalog cookie. Enable cookies for this site.");
    }

    async open() {
        this.db = await new Promise((resolve, reject) => {
            const request = indexedDB.open("virtualbox-browser", 1);
            request.onupgradeneeded = () => request.result.createObjectStore("machines", { keyPath: "id" });
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
            request.onblocked = () => reject(new Error("Close other browser-edition tabs to upgrade storage."));
        });
        this.db.onversionchange = () => this.db.close();
        // Migrate the previous IndexedDB catalog without copying binary data into cookies.
        const legacy = (await this.transaction("readonly", store => store.getAll()))
            .filter(machine => machine.config);
        if (legacy.length) {
            const rows = this.catalog();
            for (const machine of legacy) {
                if (!rows.some(row => row.id === machine.id))
                    rows.push({ id: machine.id, config: validateConfig(machine.config) });
            }
            this.writeCatalog(rows);
            await new Promise((resolve, reject) => {
                const tx = this.db.transaction("machines", "readwrite");
                for (const { config, ...payload } of legacy) tx.objectStore("machines").put(payload);
                tx.oncomplete = resolve;
                tx.onerror = tx.onabort = () => reject(tx.error || new Error("VM migration failed."));
            });
        }
    }

    async transaction(mode, action) {
        return new Promise((resolve, reject) => {
            const tx = this.db.transaction("machines", mode);
            const request = action(tx.objectStore("machines"));
            tx.oncomplete = () => resolve(request.result);
            tx.onerror = tx.onabort = () => reject(tx.error || new Error("Browser storage transaction failed."));
        });
    }

    async list() {
        const payloads = await this.transaction("readonly", store => store.getAll());
        const byId = new Map(payloads.map(payload => [payload.id, payload]));
        return this.catalog().map(row => ({ media: {}, saved: null, snapshots: [], ...byId.get(row.id), ...row }));
    }

    async get(id) {
        const row = this.catalog().find(row => row.id === id);
        if (!row) return undefined;
        const payload = await this.transaction("readonly", store => store.get(id));
        return { media: {}, saved: null, snapshots: [], ...payload, ...row };
    }

    async put(machine) {
        const previous = this.catalog();
        const row = { id: machine.id, config: validateConfig(machine.config) };
        if (typeof row.id !== "string" || !/^[a-zA-Z0-9-]{1,64}$/.test(row.id))
            throw new Error("Invalid VM identifier.");
        const rows = previous.filter(entry => entry.id !== machine.id);
        const index = previous.findIndex(entry => entry.id === machine.id);
        rows.splice(index < 0 ? rows.length : index, 0, row);
        this.writeCatalog(rows);
        const { config, ...payload } = machine;
        try {
            return await this.transaction("readwrite", store => store.put(payload));
        } catch (error) {
            this.writeCatalog(previous);
            throw error;
        }
    }

    async remove(id) {
        const previous = this.catalog();
        this.writeCatalog(previous.filter(row => row.id !== id));
        try {
            return await this.transaction("readwrite", store => store.delete(id));
        } catch (error) {
            this.writeCatalog(previous);
            throw error;
        }
    }
}
