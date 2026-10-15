// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Hands-on checks of the real desktop app, run by `run.sh` on a virtual
// display: two copies of Bindle, each with its own data folder, and a local
// Bindle Cloud relay between them. Each check prints ✓ or ✗, and a
// screenshot of every screen it reaches lands in the output folder.

import { ENTER, open } from "./webdriver.mjs";

const APP = process.env.BINDLE_APP;
const RELAY = process.env.BINDLE_RELAY_URL;
const OUT = process.env.BINDLE_SHOTS;
const failures = [];

async function check(name, run) {
  try {
    await run();
    console.log(`✓ ${name}`);
  } catch (error) {
    console.log(`✗ ${name}: ${error.message}`);
    failures.push(name);
    const file = name.replace(/[^a-z0-9]+/gi, "-").toLowerCase();
    await a.screenshot(`${OUT}/failed-${file}.png`).catch(() => {});
  }
}

/** Fails when a screen shows an error, broken text, or threw in the page. */
async function looksRight(app, name) {
  await app.sleep(900);
  await app.screenshot(`${OUT}/${name}.png`);
  const text = await app.text();
  for (const bad of [/could not[^.\n]*/i, /something went wrong[^.\n]*/i, /\bundefined\b|\bNaN\b|\[object Object\]/]) {
    const found = text.match(bad);
    if (found) throw new Error(`${name} shows “${found[0]}”`);
  }
  const thrown = await app.js("return (window.__thrown || []).splice(0)");
  if (thrown.length) throw new Error(`${name} threw: ${thrown.join("; ")}`);
}

async function firstRun(app, name, jots) {
  await app.find("//button[normalize-space(.)='Start']");
  await app.js(
    "window.__thrown = [];" +
      "addEventListener('error', (e) => window.__thrown.push(e.message));" +
      "addEventListener('unhandledrejection', (e) => window.__thrown.push(String(e.reason?.message ?? JSON.stringify(e.reason))));",
  );
  await app.button("Start");
  await app.fill("//input[@placeholder='Your first name']", name);
  await app.button("Next");
  await app.button("Next");
  for (const words of jots) {
    await app.fill("//input[contains(@placeholder,'Jot down')]", words + ENTER);
    await app.sleep(400);
  }
  await app.button(jots.length ? "Go to Home" : "Skip for now");
}

async function startCloud(app) {
  await app.click("//*[normalize-space(.)='Settings']");
  await app.fill("//input[@placeholder='https://cloud.example.com']", RELAY);
  await app.button("Start Bindle Cloud");
  await app.find("//*[contains(normalize-space(.),'Your recovery kit.')]");
  await app.button("I saved it");
}

const a = await open(APP, 4444);
const b = await open(APP, 4445);
try {
  await check("first run, with three things jotted down", () =>
    firstRun(a, "Sparsh", ["send the deck to Priya by Friday", "Rohan will pay 500 kal", "call mom sunday"]),
  );

  await check("Sort keeps each promise once you say who does it", async () => {
    await a.go("Sort");
    for (let kept = 0; kept < 3; kept++) {
      await a.find("//button[normalize-space(.)='Keep it']");
      if (!(await a.exists("//div[@aria-labelledby='who']/button[@aria-pressed='true']"))) await a.button("I'll do it");
      await a.button("Keep it");
      await a.sleep(500);
    }
    await a.find("//p[normalize-space(.)='All sorted.']");
  });

  await check("People: archive someone once nothing is open, then bring them back", async () => {
    await a.go("People");
    await a.click("//button[contains(@class,'person-button')][.//*[normalize-space(.)='Priya']]");
    if (await a.exists("//button[normalize-space(.)='Archive Priya']")) throw new Error("Archive offered while a promise is open");
    await a.go("Home");
    await a.click("//button[@aria-label='Mark done: Send the deck to Priya']");
    await a.go("People");
    await a.click("//button[contains(@class,'person-button')][.//*[normalize-space(.)='Priya']]");
    await a.button("Archive Priya");
    await a.sleep(500);
    if (await a.exists("//button[contains(@class,'person-button')][.//*[normalize-space(.)='Priya']]")) throw new Error("Priya is still listed");
    await a.click("//summary[starts-with(normalize-space(.),'Archived')]");
    await a.screenshot(`${OUT}/people-archived.png`);
    await a.click("//button[@aria-label='Bring back Priya']");
    await a.find("//button[contains(@class,'person-button')][.//*[normalize-space(.)='Priya']]");
  });

  for (const [mode, screens] of [
    ["Life", ["Home", "Sort", "Promises", "People", "Wrap up"]],
    ["Work", ["Desk", "Inbox", "Code", "Board", "Creator"]],
  ]) {
    await check(`every ${mode} screen opens cleanly`, async () => {
      await a.button(mode);
      for (const screen of screens) {
        await a.go(screen);
        await looksRight(a, `${mode}-${screen}`.toLowerCase().replace(" ", "-"));
      }
    });
  }
  await check("Settings opens cleanly", async () => {
    await a.click("//*[normalize-space(.)='Settings']");
    await looksRight(a, "settings");
  });

  await check("Bindle Cloud starts on both devices", async () => {
    await startCloud(a);
    await firstRun(b, "Ananya", []);
    await startCloud(b);
  });

  await check("Ours: a space, an invite, and a promise both people see", async () => {
    await a.fill("//input[@placeholder='Upadhyay Home']", "Upadhyay Home");
    await a.button("Make it");
    await a.find("//*[normalize-space(.)='Who carries Upadhyay Home']");
    await a.find("//ul[contains(@class,'ours-members')]//*[contains(normalize-space(.),'Sparsh')]");
    await a.button("Make an invite link");
    const link = await a.text("//p[contains(@class,'kit-code')]");
    await b.fill("//input[contains(@placeholder,'bindle/join')]", link);
    await b.button("Join");
    await b.find("//*[normalize-space(.)='Who carries Upadhyay Home']");
    await a.fill("//input[@placeholder='Pay electricity bill by Friday']", "Pay electricity bill by Friday");
    await a.button("Save");
    for (const app of [a, b, a]) {
      await app.button("Sync now");
      await app.sleep(1200);
    }
    await a.screenshot(`${OUT}/ours-sparsh.png`);
    await b.screenshot(`${OUT}/ours-ananya.png`);
    await b.find("//*[normalize-space(.)='Pay electricity bill']");
    await b.find("//ul[contains(@class,'ours-members')]//*[contains(normalize-space(.),'Sparsh')]");
    await a.find("//ul[contains(@class,'ours-members')]//*[contains(normalize-space(.),'Ananya')]");
  });
} finally {
  await a.close().catch(() => {});
  await b.close().catch(() => {});
}

if (failures.length) {
  console.log(`\n${failures.length} desktop check(s) failed. Screenshots are in ${OUT}.`);
  process.exit(1);
}
console.log("\nAll desktop checks passed.");
