// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import { AVATARS } from "../lib/commands";
import { avatarLabels } from "./Avatar";

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
});
