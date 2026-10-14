// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Bindle Cloud for the browser preview: a pretend account and the v2 Ours
// design's sample space. Nothing is sealed or sent anywhere.

import type { CloudStatus, OursView, Promise as BindlePromise, SharedPromise, Space } from "./commands";

const dayFromNow = (days: number) => new Date(Date.now() + days * 86_400_000).toISOString().slice(0, 10);
const now = () => new Date().toISOString();

export const cloud: CloudStatus = {
  account: { relay: null, name: null, account: null, device: null, kit_saved_at: null },
  spaces: [],
};

export const SAMPLE_KIT = "7KQ2-M9XD-4TVB-PH3N-8RCA-W2YF-6GJE-5SZK-1QNM-T4DV-B8XH-3PWC-R7KA-9F";

function promise(id: string, title: string, state: BindlePromise["state"], days: number | null, extra: Partial<BindlePromise> = {}): BindlePromise {
  return {
    id, title, state, direction: "mine", strength: "firm", mode: "life", first_step: null,
    do_on: days === null ? null : { day: dayFromNow(days), time: null }, due: null, amount: null, person: null,
    created_at: now(), updated_at: now(), closed_at: state === "done" ? new Date(Date.now() - (days ?? 0) * -86_400_000).toISOString() : null, ...extra,
  };
}

const shared: SharedPromise[] = [
  { promise: promise("o1", "Pay electricity bill", "open", 0, { amount: { minor: 234000, currency: "INR" } }), added_by: "Wifey", done_by: null },
  { promise: promise("o2", "Book Diwali train tickets to Lucknow", "open", 3, { first_step: "Check Tatkal timing" }), added_by: "Wifey", done_by: null },
  { promise: promise("o3", "Call the landlord about the leaking tap", "open", 2), added_by: null, done_by: null },
  { promise: promise("o4", "Mummy's birthday gift", "open", 6, { amount: { minor: 300000, currency: "INR" } }), added_by: null, done_by: null },
  { promise: promise("k1", "Refill gas cylinder", "done", 0), added_by: "Wifey", done_by: "Wifey" },
  { promise: promise("k2", "Renew car insurance", "done", -2), added_by: null, done_by: null },
];

export function oursView(id: string): OursView {
  const space = cloud.spaces.find((s) => s.id === id) ?? cloud.spaces[0];
  return {
    space,
    members: [
      { account: "me", name: cloud.account.name ?? "Sparsh", joined_at: "2026-10-10T10:00:00Z" },
      { account: "wifey", name: "Wifey", joined_at: "2026-10-12T10:00:00Z" },
    ],
    me: "me",
    open: shared.filter((s) => s.promise.state !== "done"),
    kept_lately: shared.filter((s) => s.promise.state === "done"),
    kept_this_month: 14,
  };
}

export function addSpace(name: string): Space {
  const space: Space = { id: crypto.randomUUID().replace(/-/g, "").slice(0, 32), name, relay: cloud.account.relay, promises: 6, synced_at: now(), waiting: 0 };
  cloud.spaces.push(space);
  return space;
}
