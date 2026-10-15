// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import { AVATARS } from "../lib/commands";
import { avatarLabels, pictureFor } from "./Avatar";

describe("people's pictures", () => {
  it("every picture has words describing what is drawn", () => {
    expect(Object.keys(avatarLabels).sort()).toEqual([...AVATARS].sort());
  });

  it("no picture is labelled with a gender", () => {
    const words = Object.values(avatarLabels).join(" ").toLowerCase();
    for (const word of ["male", "female", "man", "woman", "boy", "girl"]) {
      expect(words.split(/\W+/)).not.toContain(word);
    }
  });

  it("someone with no picture gets one from their name, always the same", () => {
    expect(pictureFor("Priya")).toBe(pictureFor("  priya "));
    expect(AVATARS).toContain(pictureFor("Rohan"));
    const spread = new Set(["Priya", "Rohan", "Kabir", "Neha", "Meera", "Aarav", "Ishaan", "Mummy"].map(pictureFor));
    expect(spread.size).toBeGreaterThan(3);
  });
});
