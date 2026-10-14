// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Words for the Code screen (handbook P3 and P14): how long something has
// waited, and the one or two sentences at the top.

import type { CodeView, WorkItem } from "./commands";

const numberWords = ["No", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten"];

/** "One", "Two" … "Ten", then digits. */
export function countWord(n: number): string {
  return numberWords[n] ?? String(n);
}

/** How long ago, short: "just now", "40 min", "6 h", "2 days", "3 weeks". */
export function age(iso: string, now: Date): string {
  const minutes = Math.max(0, Math.floor((now.getTime() - Date.parse(iso)) / 60_000));
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes} min`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} h`;
  const days = Math.floor(hours / 24);
  if (days < 14) return days === 1 ? "1 day" : `${days} days`;
  return `${Math.floor(days / 7)} weeks`;
}

/** "Looked 5 min ago", or "Looked just now". */
export function lookedAgo(iso: string | null, now: Date): string {
  if (!iso) return "Not looked yet";
  const when = age(iso, now);
  return when === "just now" ? "Looked just now" : `Looked ${when} ago`;
}

/** "latch #42". */
export function shortName(item: Pick<WorkItem, "repo" | "number">): string {
  return `${item.repo.split("/").pop()} #${item.number}`;
}

/** The sentences under the Code title. */
export function codeSummary(view: CodeView, now: Date): string {
  const authors = new Set(view.reviews.map((r) => r.author));
  const first =
    view.reviews.length === 0
      ? "Nobody is waiting on a review from you."
      : authors.size === 1
        ? `${[...authors][0]} is waiting on ${view.reviews.length === 1 ? "your review" : "your reviews"}.`
        : `${countWord(authors.size)} people are waiting on your reviews.`;
  const oldest = view.reviews[0];
  let second: string;
  if (oldest && view.reviews_older_than_a_day > 0) {
    second = `Start with ${shortName(oldest)}, asked ${age(oldest.created_at, now)} ago.`;
  } else if (view.issues.length > 0) {
    second = `${countWord(view.issues.length)} ${view.issues.length === 1 ? "issue is" : "issues are"} assigned to you.`;
  } else if (view.mine.length > 0) {
    second = `${countWord(view.mine.length)} of your pull requests ${view.mine.length === 1 ? "is" : "are"} waiting on others.`;
  } else {
    second = "Nothing else needs you on GitHub.";
  }
  return `${first} ${second}`;
}

/** What the row says about its promise, if it has one. */
export function promiseNote(item: WorkItem): string | null {
  switch (item.promise?.state) {
    case undefined:
      return null;
    case "unsorted":
      return "Waiting in Sort";
    case "open":
    case "later":
      return "A promise";
    case "done":
      return "Kept";
    case "let_go":
      return null;
  }
}
