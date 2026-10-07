/* $Id$ */
/** @file Atomic IndexedDB persistence for isolated browser VMs. */
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
export class MachineStore {
    async open() {
        this.db = await new Promise((resolve, reject) => {
            const request = indexedDB.open("virtualbox-browser", 1);
            request.onupgradeneeded = () => request.result.createObjectStore("machines", { keyPath: "id" });
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
            request.onblocked = () => reject(new Error("Close other browser-edition tabs to upgrade storage."));
        });
        this.db.onversionchange = () => this.db.close();
    }

    async transaction(mode, action) {
        return new Promise((resolve, reject) => {
            const tx = this.db.transaction("machines", mode);
            const request = action(tx.objectStore("machines"));
            tx.oncomplete = () => resolve(request.result);
            tx.onerror = tx.onabort = () => reject(tx.error || new Error("Browser storage transaction failed."));
        });
    }

    list() { return this.transaction("readonly", store => store.getAll()); }
    get(id) { return this.transaction("readonly", store => store.get(id)); }
    put(machine) { return this.transaction("readwrite", store => store.put(machine)); }
    remove(id) { return this.transaction("readwrite", store => store.delete(id)); }
}
