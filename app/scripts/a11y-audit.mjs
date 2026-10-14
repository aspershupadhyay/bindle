// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Accessibility audit (docs/ACCEPTANCE.md). With `npm run dev` running, walks
// through first run, every Life screen, and Work's Desk, Code, Inbox, Board and Creator, and
// Bindle Cloud with an Ours space, in
// the browser preview, and lists unnamed buttons and links, images without
// alt text, unlabelled fields, duplicate ids, missing landmarks, and a skip
// link that is not first.
// Needs Playwright: `npm i -g playwright`, then
// `PLAYWRIGHT_DIR=$(npm root -g) node scripts/a11y-audit.mjs`.

import { createRequire } from "node:module";
const require = createRequire(`${process.env.PLAYWRIGHT_DIR ?? process.cwd() + "/node_modules"}/`);
const { chromium } = require("playwright");
const url = process.env.BINDLE_URL ?? "http://localhost:1420/";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 860 } });
await page.goto(url);
await page.waitForTimeout(800);
const problems = [];
async function audit(where) {
  const found = await page.evaluate(() => {
    const out = [];
    const name = (el) => (el.getAttribute("aria-label") || el.getAttribute("title") || el.textContent || "").trim()
      || [...el.querySelectorAll("img[alt]")].map((i) => i.alt).join("").trim()
      || (el.getAttribute("aria-labelledby") ? "x" : "");
    for (const b of document.querySelectorAll("button, a[href]")) if (!name(b)) out.push(`unnamed ${b.tagName} ${b.className}`);
    for (const i of document.querySelectorAll("img")) if (!i.hasAttribute("alt")) out.push(`img without alt ${i.src.slice(-30)}`);
    for (const f of document.querySelectorAll("input, textarea, select")) {
      if (f.type === "hidden") continue;
      const labelled = f.getAttribute("aria-label") || f.getAttribute("aria-labelledby") || f.closest("label") || (f.id && document.querySelector(`label[for="${f.id}"]`));
      if (!labelled) out.push(`unlabelled ${f.tagName} ${f.type} ${f.placeholder}`);
    }
    const ids = [...document.querySelectorAll("[id]")].map((e) => e.id);
    for (const id of new Set(ids.filter((id, i) => ids.indexOf(id) !== i))) out.push(`duplicate id ${id}`);
    if (document.querySelectorAll("h1").length !== 1) out.push(`h1 count ${document.querySelectorAll("h1").length}`);
    if (!document.querySelector("main")) out.push("no main landmark");
    return out;
  });
  for (const f of found) problems.push(`${where}: ${f}`);
  console.log(`${where}: ${found.length} problems`);
}
await audit("first run");
// Skip through first run.
for (let i = 0; i < 6; i++) {
  const skip = page.getByRole("button", { name: /^(skip|start|next|go to home)/i }).first();
  if (await skip.count() === 0) break;
  await skip.click(); await page.waitForTimeout(300);
}
const first = await page.evaluate(() => document.querySelector("a[href], button, input, textarea, select, [tabindex='0']")?.textContent);
if (!/skip/i.test(first ?? "")) problems.push(`first Tab is "${first}", not the skip link`);
for (const screen of ["Home", "Sort", "Promises", "People", "Wrap up", "Settings"]) {
  await page.getByRole("button", { name: new RegExp(`^${screen}`) }).first().click();
  await page.waitForTimeout(400);
  await audit(screen);
}
// Work: Code, before and after connecting GitHub (the preview's sample data).
await page.getByRole("button", { name: /^Work$/ }).click();
await page.getByRole("button", { name: /^Code/ }).click();
await page.waitForTimeout(400);
await audit("Code, not connected");
await page.getByLabel("Paste the token here").fill("preview-token");
await page.getByRole("button", { name: /^Connect$/ }).click();
await page.waitForTimeout(400);
await audit("Code, connected");
await page.getByRole("button", { name: /^Inbox/ }).click();
await page.waitForTimeout(400);
await audit("Inbox, not connected");
await page.getByLabel("Your Gmail address").fill("sparsh@gmail.com");
await page.getByLabel("The app password").fill("abcd efgh ijkl mnop");
await page.getByRole("button", { name: /^Connect$/ }).click();
await page.waitForTimeout(400);
await audit("Inbox, connected");
await page.getByRole("button", { name: /^Board/ }).click();
await page.waitForTimeout(400);
await audit("Board");
await page.getByRole("button", { name: /^Desk/ }).click();
await page.waitForTimeout(400);
await audit("Desk");
await page.getByRole("button", { name: /^Creator/ }).click();
await page.waitForTimeout(400);
await audit("Creator, not connected");
await page.getByLabel("Your channel's handle").fill("@aspersh");
await page.getByLabel("The API key").fill("AIza-preview");
await page.getByRole("button", { name: /^Connect$/ }).click();
await page.waitForTimeout(400);
await audit("Creator, connected");
await page.getByRole("button", { name: /^Life$/ }).click();
// Bindle Cloud: start, the recovery kit, then an Ours space.
await page.getByRole("button", { name: /^Settings/ }).first().click();
await page.getByLabel("Bindle Cloud address").fill("https://cloud.example.com");
await page.getByLabel("The name others see").fill("Sparsh");
await page.getByRole("button", { name: /^Start Bindle Cloud$/ }).click();
await page.waitForTimeout(400);
await audit("Recovery kit");
await page.getByRole("button", { name: /^I saved it$/ }).click();
await page.getByLabel("Make a shared space").fill("Upadhyay Home");
await page.getByRole("button", { name: /^Make it$/ }).click();
await page.waitForTimeout(400);
await audit("Ours");
await page.getByRole("button", { name: /^Make an invite link$/ }).click();
await page.waitForTimeout(300);
await audit("Ours, invite");
console.log(problems.join("\n") || "no problems");
await browser.close();
process.exitCode = problems.length > 0 ? 1 : 0;
