// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import { DOWNLOAD_PAGE, percent, updateNote } from "./updates";

describe("percent", () => {
  it("is empty when the size is unknown", () => {
    expect(percent(500, null)).toBe("");
    expect(percent(500, 0)).toBe("");
  });

  it("rounds down and never passes 100", () => {
    expect(percent(1, 3)).toBe("33%");
    expect(percent(12, 10)).toBe("100%");
  });
});

describe("updateNote", () => {
  it("says what is happening in plain words", () => {
    expect(updateNote({ kind: "idle" })).toBe("");
    expect(updateNote({ kind: "none" })).toBe("You have the newest version.");
    expect(updateNote({ kind: "available", version: "0.0.2", notes: null })).toBe("Bindle 0.0.2 is ready to install.");
    expect(updateNote({ kind: "installing", version: "0.0.2", done: 50, total: 200 })).toBe("Getting Bindle 0.0.2… 25%");
    expect(updateNote({ kind: "installing", version: "0.0.2", done: 50, total: null })).toBe("Getting Bindle 0.0.2…");
  });

  it("points to the download page when updating fails", () => {
    expect(updateNote({ kind: "error", message: "offline" })).toContain(DOWNLOAD_PAGE);
  });
});
