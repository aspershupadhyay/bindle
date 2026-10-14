// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import { dateLine, greeting } from "./days";

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
