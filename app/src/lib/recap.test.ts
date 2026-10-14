// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import type { PersonDetail, Promise as BindlePromise } from "./commands";
import { recapMessage } from "./recap";

const wednesday = new Date(2026, 9, 14, 18, 30);

function promise(title: string, patch: Partial<BindlePromise> = {}): BindlePromise {
  return {
    id: title, title, state: "open", direction: "mine", strength: "firm", mode: "life", first_step: null,
    do_on: null, due: null, amount: null, person: "Rohan", created_at: "", updated_at: "", closed_at: null, ...patch,
  };
}

function rohan(patch: Partial<PersonDetail>): PersonDetail {
  return { id: "r", name: "Rohan", avatar: null, pronouns: null, hidden_from_ai: false, you_owe: [], they_owe: [], kept_lately: [], ...patch };
}

describe("recapMessage", () => {
  it("lists both sides with dates written out in full", () => {
    const message = recapMessage(
      rohan({
        you_owe: [promise("Send the lesson plan", { due: { day: "2026-10-16", time: null } })],
        they_owe: [promise("Pay the invoice", { direction: "theirs", amount: { minor: 1_800_000, currency: "INR" }, due: { day: "2026-10-09", time: null } })],
        kept_lately: [promise("Share notes", { state: "done" })],
      }),
      wednesday,
    );
    expect(message).toBe(
      [
        "Hi Rohan, a quick recap of where we are:",
        "",
        "From me:",
        "• Send the lesson plan, by Fri 16 Oct",
        "",
        "From you:",
        "• Pay the invoice (₹18,000), was due Fri 9 Oct",
        "",
        "Done lately:",
        "• Share notes",
        "",
        "Thanks!",
      ].join("\n"),
    );
  });

  it("has nothing to say when there is nothing between you", () => {
    expect(recapMessage(rohan({}), wednesday)).toBeNull();
  });

  it("names the amount once when the title already has it", () => {
    const invoice = promise("Pay the ₹18,000 invoice", { direction: "theirs", amount: { minor: 1_800_000, currency: "INR" } });
    expect(recapMessage(rohan({ they_owe: [invoice] }), wednesday)).toContain("• Pay the ₹18,000 invoice\n");
  });

  it("uses on for a planned day", () => {
    const message = recapMessage(rohan({ you_owe: [promise("Call", { do_on: { day: "2026-10-15", time: null } })] }), wednesday);
    expect(message).toContain("• Call, on Thu 15 Oct");
  });
});
