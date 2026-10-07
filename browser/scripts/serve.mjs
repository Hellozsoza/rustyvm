/* $Id$ */
/** @file Static browser edition development server. */
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
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { resolve, extname, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../dist", import.meta.url));
const types = { ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript",
    ".css": "text/css", ".wasm": "application/wasm", ".png": "image/png", ".json": "application/json" };
createServer(async (request, response) => {
    try {
        const path = resolve(root, `.${decodeURIComponent(new URL(request.url, "http://localhost").pathname)}`);
        if (path !== root && !path.startsWith(root + sep)) {
            response.writeHead(403).end();
            return;
        }
        const file = (await stat(path)).isDirectory() ? `${path}/index.html` : path;
        const content = await readFile(file);
        response.writeHead(200, {
            "Content-Type": types[extname(file)] || "application/octet-stream",
            "Cache-Control": "no-cache",
            "X-Content-Type-Options": "nosniff",
            "Content-Security-Policy": "default-src 'self'; script-src 'self' blob: 'wasm-unsafe-eval'; worker-src 'self' blob:; style-src 'self' 'unsafe-inline'; img-src 'self' blob: data:; connect-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'"
        }).end(content);
    } catch {
        response.writeHead(404).end("Not found. Run npm run build first.");
    }
}).listen(Number(process.env.PORT || 8080), process.env.HOST || "127.0.0.1", () => {
    console.log(`Browser edition server listening on port ${process.env.PORT || 8080}`);
});
