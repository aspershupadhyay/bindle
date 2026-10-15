// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A small WebDriver client for tauri-driver, so the desktop checks drive the
// real Bindle app (its Rust commands, database and webview), not the browser
// preview. Elements are found by XPath, mostly by the words people see.

import fs from "node:fs";

const ELEMENT = "element-6066-11e4-a07c-4ab5ec7c5d6f";
export const ENTER = "";

async function call(base, method, path, body) {
  const response = await fetch(base + path, {
    method,
    headers: { "content-type": "application/json" },
    body: body ? JSON.stringify(body) : undefined,
  });
  const reply = await response.json();
  if (reply.value?.error) throw new Error(`${path}: ${reply.value.error} ${reply.value.message}`);
  return reply.value;
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/** Starts the app at `application` through the tauri-driver on `port`. */
export async function open(application, port) {
  const base = `http://127.0.0.1:${port}`;
  const { sessionId } = await call(base, "POST", "/session", {
    capabilities: { alwaysMatch: { "tauri:options": { application } } },
  });
  const s = (method, path, body) => call(base, method, `/session/${sessionId}${path}`, body);

  async function find(xpath, tries = 24) {
    for (let i = 0; i < tries; i++) {
      try {
        const found = await s("POST", "/element", { using: "xpath", value: xpath });
        const id = found[ELEMENT] ?? Object.values(found)[0];
        if (id) return id;
      } catch {
        // Not there yet: screens load their data after they appear.
      }
      await sleep(250);
    }
    throw new Error(`not found: ${xpath}`);
  }

  const app = {
    sleep,
    find,
    async exists(xpath) {
      return (await s("POST", "/elements", { using: "xpath", value: xpath })).length > 0;
    },
    /** Clicks once the element can take a click (it may still be appearing). */
    async click(xpath) {
      for (let tries = 0; ; tries++) {
        try {
          await s("POST", `/element/${await find(xpath)}/click`, {});
          break;
        } catch (error) {
          if (tries >= 12 || !/interactable|intercepted|stale/i.test(error.message)) throw error;
          await sleep(250);
        }
      }
      await sleep(300);
    },
    /** Clicks the button whose words are exactly `words`. */
    button(words) {
      return app.click(`//button[normalize-space(.)=${JSON.stringify(words)}]`);
    },
    /** Clicks a sidebar entry that starts with `words`. */
    go(words) {
      return app.click(`//nav//*[starts-with(normalize-space(.),${JSON.stringify(words)})]`);
    },
    /** Replaces what the field at `xpath` holds with `text`. */
    async fill(xpath, text) {
      const id = await find(xpath);
      await s("POST", `/element/${id}/clear`, {});
      await s("POST", `/element/${id}/value`, { text });
    },
    async text(xpath = "//body") {
      return s("GET", `/element/${await find(xpath)}/text`);
    },
    js(script, args = []) {
      return s("POST", "/execute/sync", { script, args });
    },
    async screenshot(file) {
      fs.writeFileSync(file, Buffer.from(await s("GET", "/screenshot"), "base64"));
    },
    close() {
      return s("DELETE", "");
    },
  };
  return app;
}
