// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Sample email and Board for the browser preview, from the v2 Inbox and
// Board designs. None of it is real, and the preview never reads a mailbox.

import type { BoardView, InboxView, MailAccount, MailItem, MailKind, Promise as BindlePromise } from "./commands";

const hoursAgo = (hours: number) => new Date(Date.now() - hours * 3_600_000).toISOString();
const dayFromNow = (days: number) => new Date(Date.now() + days * 86_400_000).toISOString().slice(0, 10);

function sample(id: string, kind: MailKind, person: string, address: string, hours: number, subject: string, snippet: string, extra: Partial<MailItem> = {}): MailItem {
  return {
    id,
    kind,
    subject,
    person,
    address,
    at: hoursAgo(hours),
    snippet,
    url: `https://mail.google.com/mail/u/0/#search/rfc822msgid%3A${id}`,
    noticed: null,
    gone: false,
    promise: null,
    ...extra,
  };
}

export const sampleMail: MailItem[] = [
  sample("m1", "reply", "Priya Sharma", "priya@brand.example", 50, "Sponsor brief: Q4 carousel series", "Hi Sparsh, We would love 4 carousels between 20 October and 10 November, one a week, on how people in India actually pay online. Could you share your rate card and a first concept so we can lock dates?"),
  sample("m2", "reply", "Rohan Mehta", "rohan@studio.example", 26, "Collab on the Office clones video?", "Saw Case 018 do well. Want to do a split-screen video next week? I can shoot on Saturday if that works for you."),
  sample("m3", "waiting", "Neha Kapoor", "neha@gymbros.example", 30, "Re: GymBros animation timings", "Could you send the new curl timings when you have them?"),
  sample("m4", "promise", "Priya Sharma", "priya@brand.example", 46, "Re: Sponsor brief: Q4 carousel series", "Sounds great. I will send the first concept by Friday.", {
    noticed: { title: "Send the first concept", when: { day: dayFromNow(2), time: null }, is_deadline: true },
  }),
];

const hidden = new Set<string>();

export function sampleInbox(account: MailAccount): InboxView {
  if (!account.address) return { account, reply: [], waiting: [], noticed: [], finished: [], hidden: 0 };
  const shown = sampleMail.filter((m) => !hidden.has(m.id));
  return {
    account,
    reply: shown.filter((m) => m.kind === "reply"),
    waiting: shown.filter((m) => m.kind === "waiting"),
    noticed: shown.filter((m) => m.kind === "promise" && !m.promise),
    finished: [],
    hidden: hidden.size,
  };
}

export function hideMail(id: string, hide: boolean) {
  if (hide) hidden.add(id);
  else hidden.delete(id);
}

function samplePromise(id: string, title: string, state: BindlePromise["state"], days: number | null, extra: Partial<BindlePromise> = {}): BindlePromise {
  const now = new Date().toISOString();
  return {
    id, title, state, direction: "mine", strength: "firm", mode: "work", first_step: null,
    do_on: days === null ? null : { day: dayFromNow(days), time: null }, due: null, amount: null, person: null,
    created_at: now, updated_at: now, closed_at: state === "done" ? now : null, ...extra,
  };
}

/** Board cards from the v2 design, shown until the preview has its own work promises. */
const sampleCards: { promise: BindlePromise; from: string | null; making?: boolean }[] = [
  { promise: samplePromise("b1", "Case 020: pick a myth from the comments", "later", null), from: null },
  { promise: samplePromise("b2", "Bindle landing page, first sketch", "later", null), from: null },
  { promise: samplePromise("b3", "Send Priya the first concept", "open", 2, { person: "Priya Sharma" }), from: "Gmail" },
  { promise: samplePromise("b4", "Review latch #42: Permission prompts", "open", 0, { person: "kunal-m" }), from: "GitHub kunal-m/latch#42" },
  { promise: samplePromise("b5", "File GST for September", "open", 6), from: null },
  { promise: samplePromise("b6", "Fix codex-claude-bridge #17: Token refresh fails on Windows", "open", 1), from: "GitHub aspershupadhyay/codex-claude-bridge#17", making: true },
  { promise: samplePromise("b7", "Case 019: Incognito and your ISP", "open", 3), from: null, making: true },
  { promise: samplePromise("b8", "Hybrid logical clock for sync", "open", 4), from: "GitHub aspershupadhyay/bindle#3", making: true },
  { promise: samplePromise("b9", "Neha to reply: curl timings", "open", 1, { direction: "theirs", person: "Neha Kapoor" }), from: "Gmail" },
  { promise: samplePromise("b10", "Latch share sheet v2", "done", null), from: "GitHub kunal-m/latch#40" },
];

/** The Board from the preview's work promises; `making` holds the ids in Making. */
export function sampleBoard(promises: BindlePromise[], making: Set<string>): BoardView {
  const own = promises.filter((p) => p.mode === "work" && p.state !== "unsorted");
  if (own.length === 0) {
    const lane = (test: (c: (typeof sampleCards)[number]) => boolean) => sampleCards.filter(test).map(({ promise, from }) => ({ promise, from }));
    return {
      ideas: lane((c) => c.promise.state === "later"),
      next: lane((c) => c.promise.state === "open" && !c.making && c.promise.direction !== "theirs"),
      making: lane((c) => Boolean(c.making)),
      waiting: lane((c) => c.promise.direction === "theirs"),
      shipped: lane((c) => c.promise.state === "done"),
    };
  }
  const work = own;
  const card = (promise: BindlePromise) => ({ promise, from: promise.person ? "Gmail" : null });
  return {
    ideas: work.filter((p) => p.state === "later").map(card),
    next: work.filter((p) => p.state === "open" && !making.has(p.id) && p.direction !== "theirs").map(card),
    making: work.filter((p) => p.state === "open" && making.has(p.id)).map(card),
    waiting: work.filter((p) => p.state === "open" && !making.has(p.id) && p.direction === "theirs").map(card),
    shipped: work.filter((p) => p.state === "done").map(card),
  };
}
