// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Colour contrast for every text colour on every surface it sits on, in both
// themes (handbook P2, accessibility): WCAG AA, 4.5:1 for text and 3:1 for
// focus rings and borders that carry meaning.

import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("../tokens.css", import.meta.url), "utf8");

/** The custom properties in the first block that follows `marker`. */
function block(marker: string): Record<string, string> {
  const start = css.indexOf(marker);
  const body = css.slice(css.indexOf("{", start) + 1, css.indexOf("}", start));
  return Object.fromEntries([...body.matchAll(/--([\w-]+):\s*(#[0-9A-Fa-f]{6})/g)].map((m) => [m[1], m[2]]));
}

const light = block(":root {");
const dark = { ...light, ...block(':root[data-theme="dark"]') };

function luminance(hex: string): number {
  const [r, g, b] = [1, 3, 5].map((i) => {
    const c = parseInt(hex.slice(i, i + 2), 16) / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

const text = ["ink", "ink-mid", "ink-soft", "clay-text", "sage-text", "honey-text"];
const surfaces = ["paper", "card", "sand-soft"];
const chips: [string, string][] = [
  ["clay-text", "clay-wash"],
  ["sage-text", "sage-wash"],
  ["honey-text", "honey-wash"],
  ["card", "clay-button"], // the primary button
];

describe.each([
  ["light", light],
  ["dark", dark],
])("the %s theme", (_name, theme) => {
  it("has every token the checks need", () => {
    for (const token of [...text, ...surfaces, "clay", "clay-button", "clay-wash", "sage-wash", "honey-wash"]) {
      expect(theme[token], token).toMatch(/^#/);
    }
  });

  it.each(text.flatMap((t) => surfaces.map((s) => [t, s])))("%s on %s is readable (4.5:1)", (fg, bg) => {
    expect(contrast(theme[fg], theme[bg])).toBeGreaterThanOrEqual(4.5);
  });

  it.each(chips)("%s on %s is readable (4.5:1)", (fg, bg) => {
    expect(contrast(theme[fg], theme[bg])).toBeGreaterThanOrEqual(4.5);
  });

  it("the focus ring stands out from the page (3:1)", () => {
    expect(contrast(theme.clay, theme.paper)).toBeGreaterThanOrEqual(3);
  });
});
