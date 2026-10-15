// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import type { Promise as BindlePromise } from "./commands";
import { lateWords, nudgeMessage, waitingWords, weekNote, withPerson } from "./home";

const wednesday = new Date(2026, 9, 14, 18, 30);

function promise(patch: Partial<BindlePromise>): BindlePromise {
  return {
    id: "p1", title: "Pay the invoice", state: "open", direction: "theirs", strength: "firm", mode: "life",
    first_step: null, do_on: null, due: null, amount: null, person: "Rohan",
    created_at: "2026-10-02T09:00:00.000Z", updated_at: "", closed_at: null, ...patch,
  };
}

describe("lateWords", () => {
  it("names the weekday within a week", () => {
    expect(lateWords(promise({ do_on: { day: "2026-10-12", time: null } }), wednesday)).toBe("Running late since Monday");
  });
  it("says yesterday and gives older dates", () => {
    expect(lateWords(promise({ due: { day: "2026-10-13", time: null } }), wednesday)).toBe("Running late since yesterday");
    expect(lateWords(promise({ due: { day: "2026-10-01", time: null } }), wednesday)).toBe("Running late since 1 Oct");
  });
});

describe("waitingWords", () => {
  it("says when it was promised, and the nudge", () => {
    expect(waitingWords(promise({}), null, wednesday)).toBe("They'll do it · promised 2 Oct");
    const recent = promise({ created_at: "2026-10-08T09:00:00.000Z", due: { day: "2026-10-16", time: null } });
    expect(waitingWords(recent, "2026-10-13T10:00:00.000Z", wednesday)).toBe(
      "They'll do it · since Thursday · due Friday · nudged yesterday",
    );
  });
});

describe("waitingWords after the due day", () => {
  it("says it was due, without blame", () => {
    const late = promise({ due: { day: "2026-10-09", time: null } });
    expect(waitingWords(late, null, wednesday)).toBe("They'll do it · promised 2 Oct · was due Friday");
  });
});

describe("weekNote", () => {
  const day = (d: number, count: number) => ({ day: `2026-10-${d}`, count, full: count >= 5 });
  it("warns gently about the first full day from today on", () => {
    expect(weekNote([day(12, 6), day(14, 2), day(16, 5)], wednesday)).toBe(
      "Friday already has 5 things. Think before you say yes.",
    );
  });
  it("says nothing when no day ahead is full", () => {
    expect(weekNote([day(12, 6), day(14, 2)], wednesday)).toBeNull();
  });
});

describe("nudge and names", () => {
  it("writes a friendly nudge and joins the person to the title", () => {
    expect(nudgeMessage(promise({}))).toBe("Hi Rohan, just checking in on pay the invoice. Thanks!");
    expect(withPerson(promise({}))).toBe("Rohan · pay the invoice");
  });
  it("keeps names, I and short capitals as they are", () => {
    const rohanPays = promise({ title: "Rohan will pay 500" });
    expect(withPerson(rohanPays)).toBe("Rohan will pay 500");
    expect(nudgeMessage(rohanPays)).toBe("Hi Rohan, just checking in on Rohan will pay 500. Thanks!");
    expect(withPerson(promise({ title: "GST papers" }))).toBe("Rohan · GST papers");
    expect(withPerson(promise({ title: "I’ll get the keys back" }))).toBe("Rohan · I’ll get the keys back");
    expect(withPerson(promise({ title: "Rohanbhai's invoice" }))).toBe("Rohan · rohanbhai's invoice");
    expect(withPerson(promise({ title: "A gift" }))).toBe("Rohan · a gift");
  });
});
