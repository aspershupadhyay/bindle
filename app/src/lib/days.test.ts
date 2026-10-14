// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import { dateLine, dayString, greeting, quickDays } from "./days";

const at = (hour: number) => new Date(2026, 9, 14, hour, 0);

describe("greeting", () => {
  it("says good morning from five until noon", () => {
    expect(greeting(at(5))).toBe("Good morning");
    expect(greeting(at(11))).toBe("Good morning");
  });

  it("says good afternoon from noon until five", () => {
    expect(greeting(at(12))).toBe("Good afternoon");
  });

  it("says good evening late at night too", () => {
    expect(greeting(at(18))).toBe("Good evening");
    expect(greeting(at(2))).toBe("Good evening");
  });
});

describe("dateLine", () => {
  it("names the weekday, day and month", () => {
    expect(dateLine(at(9))).toBe("Wednesday, 14 October");
  });
});

describe("quickDays", () => {
  it("offers today, tomorrow, the weekend and next Monday", () => {
    expect(quickDays(at(9)).map((q) => q.day)).toEqual(["2026-10-14", "2026-10-15", "2026-10-17", "2026-10-19"]);
  });

  it("on a Saturday, this weekend is today", () => {
    expect(quickDays(new Date(2026, 9, 17))[2].day).toBe("2026-10-17");
  });

  it("dayString pads months and days", () => {
    expect(dayString(new Date(2026, 0, 5))).toBe("2026-01-05");
  });
});
