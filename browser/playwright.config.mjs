/* $Id$ | Real-browser guest tests | SPDX-License-Identifier: GPL-3.0-only */
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
import { defineConfig } from "@playwright/test";
export default defineConfig({
    testDir: "./tests",
    testMatch: "*.browser.mjs",
    timeout: 60000,
    workers: 1,
    use: {
        baseURL: "http://127.0.0.1:8080",
        launchOptions: { executablePath: process.env.CHROMIUM_PATH || "/usr/bin/chromium", args: ["--no-sandbox"] }
    },
    webServer: { command: "npm start", url: "http://127.0.0.1:8080", reuseExistingServer: !process.env.CI }
});
