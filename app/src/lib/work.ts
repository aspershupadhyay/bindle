// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Words for the Code screen (handbook P3 and P14): how long something has
// waited, and the one or two sentences at the top.

import type { BoardView, CodeView, DeskView, InboxView, WaitTrend, WorkItem } from "./commands";

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

/** The sentences under the Inbox title. */
const MAIL_NAMES: Record<string, string> = {
  gmail: "Gmail",
  icloud: "iCloud Mail",
  fastmail: "Fastmail",
  yahoo: "Yahoo Mail",
  zoho: "Zoho Mail",
  outlook: "Outlook",
};

/**
 * The mailbox's name as people know it: "Gmail", "iCloud Mail", or its
 * server. An account connected before other mailboxes existed is Gmail;
 * with nothing connected it is just "Email".
 */
export function mailName(account: { provider?: string | null; host?: string | null; address?: string | null }): string {
  if (!account.provider && account.address === null) return "Email";
  const id = account.provider ?? "gmail";
  return MAIL_NAMES[id] ?? account.host ?? "your mailbox";
}

export function inboxSummary(view: InboxView): string {
  const replies = view.reply.length;
  const first =
    replies === 0
      ? "No email is waiting on your reply."
      : replies === 1
        ? `${view.reply[0].person} is waiting on your reply.`
        : `${countWord(replies)} emails are waiting on your reply.`;
  const extras: string[] = [];
  if (view.noticed.length > 0) {
    extras.push(
      view.noticed.length === 1
        ? "Bindle noticed a promise you made in an email."
        : `Bindle noticed ${countWord(view.noticed.length).toLowerCase()} promises you made in emails.`,
    );
  }
  if (view.waiting.length > 0) {
    extras.push(
      `${countWord(view.waiting.length)} ${view.waiting.length === 1 ? "question of yours is" : "questions of yours are"} still waiting.`,
    );
  }
  return [first, ...extras].join(" ");
}

/** The sentence under the Board title. Never a score: just where things are. */
export function boardSummary(view: BoardView, limit: number): string {
  const making = view.making.length;
  if (making === 0) return view.next.length > 0 ? "Nothing in progress. Start one from Next." : "Nothing in progress, and nothing next.";
  const things = making === 1 ? "One thing is" : `${countWord(making)} things are`;
  return `${things} in progress, and your limit is ${countWord(limit).toLowerCase()}.`;
}

/** Where a card came from: "from Code · latch #42", "from Inbox", or nothing. */
export function cardFrom(from: string | null): string | null {
  if (!from) return null;
  if (from.startsWith("GitHub ")) {
    const ref = from.slice("GitHub ".length);
    const [repo, number] = ref.split("#");
    return `from Code · ${repo.split("/").pop()} #${number}`;
  }
  if (from === "Gmail") return "from Inbox";
  return null;
}

/** Which source a card belongs to, for the Board's filter. */
export function cardSource(from: string | null): "code" | "inbox" | "other" {
  if (from?.startsWith("GitHub ")) return "code";
  if (from === "Gmail") return "inbox";
  return "other";
}

/** Hours, the way people say them: "40 min", "3.2 h", "2 days". */
export function hoursLabel(hours: number): string {
  if (hours < 1) return `${Math.max(1, Math.round(hours * 60))} min`;
  if (hours < 10) return `${Math.round(hours * 10) / 10} h`;
  if (hours < 48) return `${Math.round(hours)} h`;
  return `${Math.round(hours / 24)} days`;
}

/** "faster than 4.1 h" or "slower than 11 h", against your own last four weeks. */
export function compareWait(trend: WaitTrend): string | null {
  if (trend.recent === null || trend.before === null) return null;
  if (Math.abs(trend.recent - trend.before) < 0.05 * trend.before) return `about the same as ${hoursLabel(trend.before)}`;
  return `${trend.recent < trend.before ? "faster" : "slower"} than ${hoursLabel(trend.before)}`;
}

/** How many weeks in a row a wait got shorter (positive) or longer (negative), up to this week. */
export function streak(weekly: (number | null)[]): number {
  const known = weekly.filter((w): w is number => w !== null);
  let run = 0;
  for (let i = known.length - 1; i > 0; i--) {
    const step = Math.sign(known[i - 1] - known[i]);
    if (step === 0 || (run !== 0 && Math.sign(run) !== step)) break;
    run += step;
  }
  return run;
}

/** The sentence under "Who waits longer". Plain, and about you against yourself. */
export function waitStory(email: WaitTrend, reviews: WaitTrend): string {
  const say = (name: string, trend: WaitTrend) => {
    const run = streak(trend.weekly);
    if (Math.abs(run) >= 2) return `${name} got ${run > 0 ? "faster" : "slower"} ${countWord(Math.abs(run)).toLowerCase()} weeks running.`;
    if (Math.abs(run) === 1) return `${name} got ${run > 0 ? "faster" : "slower"} this week.`;
    return null;
  };
  const parts = [say("Email replies", email), say("Reviews", reviews)].filter(Boolean);
  if (parts.length === 0) return "Bindle needs a few weeks of replies and reviews before it can tell a story.";
  return parts.join(" ");
}

/** The sentences under the Desk title. */
export function deskSummary(view: DeskView, now: Date): string {
  const n = view.needs_you.length;
  if (n === 0) return "Nothing needs you right now.";
  const first = n === 1 ? "One thing needs you." : `${countWord(n)} things need you.`;
  const oldest = view.needs_you[0];
  const what = oldest.kind === "review" ? "a pull request review" : oldest.kind === "issue" ? "an issue" : `an email from ${oldest.person}`;
  const day = new Date(oldest.at).toLocaleDateString("en-GB", { weekday: "long" });
  const ageDays = (now.getTime() - Date.parse(oldest.at)) / 86_400_000;
  return `${first} The oldest is ${what} from ${ageDays < 1 ? "today" : ageDays < 7 ? day : age(oldest.at, now) + " ago"}.`;
}
