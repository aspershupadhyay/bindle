// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The Desk for the browser preview, from the sample Code and Inbox items
// and made-up wait history in the shape of the v2 Desk design.

import type { CodeView, DeskItem, DeskView, InboxView } from "./commands";
import { addDays, dayString } from "./days";

export function sampleDesk(code: CodeView, inbox: InboxView): DeskView {
  const connected = Boolean(code.account.login || inbox.account.address);
  const needs: DeskItem[] = [
    ...[...code.reviews, ...code.issues].map((i): DeskItem => ({
      id: i.id,
      source: "github",
      kind: i.kind === "review" ? "review" : "issue",
      title: i.title,
      person: i.author,
      reference: `${i.repo.split("/").pop()} #${i.number}`,
      at: i.created_at,
      url: i.url,
      promise: i.promise,
    })),
    ...inbox.reply.map((m): DeskItem => ({ id: m.id, source: "gmail", kind: "reply", title: m.subject, person: m.person, reference: "", at: m.at, url: m.url, promise: m.promise })),
  ].sort((a, b) => Date.parse(a.at) - Date.parse(b.at));
  const today = new Date();
  const monday = addDays(today, -((today.getDay() + 6) % 7));
  const week = Array.from({ length: 7 }, (_, i) => ({ day: dayString(addDays(monday, i)), open: [1, 2, 1, 3, 2, 0, 0][i], done: [2, 1, 2, 0, 0, 0, 0][i], email: [1, 1, 1, 1, 0, 0, 0][i], code: [2, 1, 1, 1, 1, 0, 0][i], content: [0, 1, 1, 1, 1, 0, 0][i], other: [0, 0, 0, 0, 0, 0, 0][i] }));
  return {
    needs_you: needs,
    email_reply: connected ? { recent: 3.2, before: 4.1, weekly: [5.6, 5.1, null, 4.8, 4.4, 4.0, 3.6, 3.2] } : { recent: null, before: null, weekly: Array(8).fill(null) },
    review_wait: connected ? { recent: 18, before: 11, weekly: [10, 12, 9, 11, 12, 13, 15, 18] } : { recent: null, before: null, weekly: Array(8).fill(null) },
    shipped: connected ? { this_week: 5, weekly: [2, 3, 1, 4, 3, 2, 4, 5], from_code: 3, from_inbox: 1, other: 1 } : { this_week: 0, weekly: Array(8).fill(0), from_code: 0, from_inbox: 0, other: 0 },
    week,
    github: code.account,
    gmail: inbox.account,
    reviews_older_than_a_day: code.reviews_older_than_a_day,
    replies_owed: inbox.reply.length,
  };
}
