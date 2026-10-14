// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import { chipsForReading, moneyLabel, timeLabel, whenLabel } from "./chips";
import type { Reading } from "./commands";

const wednesday = new Date(2026, 9, 14, 18, 30);

describe("whenLabel", () => {
  it("says today and tomorrow in words", () => {
    expect(whenLabel({ day: "2026-10-14", time: null }, wednesday, false)).toBe("Today");
    expect(whenLabel({ day: "2026-10-15", time: "20:30:00" }, wednesday, false)).toBe("Tomorrow 8:30 PM");
  });

  it("names the weekday within a week and the date after that", () => {
    expect(whenLabel({ day: "2026-10-16", time: null }, wednesday, false)).toBe("Fri");
    expect(whenLabel({ day: "2026-10-23", time: null }, wednesday, false)).toBe("Fri 23 Oct");
  });

  it("starts deadlines with By", () => {
    expect(whenLabel({ day: "2026-10-16", time: null }, wednesday, true)).toBe("By Fri");
    expect(whenLabel({ day: "2026-10-15", time: null }, wednesday, true)).toBe("By tomorrow");
  });
});

describe("timeLabel", () => {
  it("leaves out zero minutes", () => {
    expect(timeLabel("09:00:00")).toBe("9 AM");
    expect(timeLabel("12:15:00")).toBe("12:15 PM");
    expect(timeLabel("00:00:00")).toBe("12 AM");
  });
});

describe("moneyLabel", () => {
  it("groups rupees the Indian way and drops zero paise", () => {
    expect(moneyLabel({ minor: 1_800_000_00, currency: "INR" })).toBe("₹18,00,000");
    expect(moneyLabel({ minor: 9950, currency: "INR" })).toBe("₹99.50");
  });
});

describe("chipsForReading", () => {
  it("adds a question mark when a word could mean two days", () => {
    const reading: Reading = {
      title: "Send reel edit to Rohan",
      when: { day: "2026-10-15", time: null },
      is_deadline: false,
      needs_check: true,
      unsure_words: "kal",
      amount: null,
      person: "Rohan",
      direction: null,
      strength: "firm",
    };
    const chips = chipsForReading(reading, wednesday);
    expect(chips.map((c) => c.label)).toEqual(["Tomorrow?", "Rohan"]);
    expect(chips[0].tone).toBe("check");
    expect(chips[0].hint).toContain("kal");
  });
});
