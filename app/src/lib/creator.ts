// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Words and numbers for the Creator screen. A channel is only ever compared
// with its own past: no scores, no other channels.

import type { CreatorView } from "./commands";

/** 642, 2.1K, 12K, 1.4M: how YouTube itself writes counts. */
export function compact(n: number): string {
  if (n < 1000) return String(n);
  const [value, unit] = n < 1_000_000 ? [n / 1000, "K"] : [n / 1_000_000, "M"];
  const shown = value < 10 ? Math.floor(value * 10) / 10 : Math.floor(value);
  return `${String(shown).replace(/\.0$/, "")}${unit}`;
}

/** "18:04", "1:02:00". */
export function duration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  const two = (n: number) => String(n).padStart(2, "0");
  return h > 0 ? `${h}:${two(m)}:${two(s)}` : `${m}:${two(s)}`;
}

/** "+28 this week", "−3 this week", "Same as last week"; null without a week of history. */
export function weekChange(change: number | null, what: string): string | null {
  if (change === null) return null;
  if (change === 0) return `No change in ${what} this week`;
  return `${change > 0 ? "+" : "−"}${compact(Math.abs(change))} this week`;
}

/** The lead sentence: who is waiting, then how the channel is doing. */
export function creatorSummary(view: CreatorView): string {
  const waiting = view.comments.filter((c) => !c.promise).length;
  const first =
    waiting === 0
      ? "No comment is waiting on your reply."
      : waiting === 1
        ? `${view.comments.find((c) => !c.promise)?.author ?? "Someone"} is waiting on your reply.`
        : `${waiting} comments are waiting on your reply.`;
  const growth = view.growth;
  if (growth.length >= 2) {
    const start = growth[0].followers;
    const end = growth[growth.length - 1].followers;
    if (start > 0 && end !== start) {
      const percent = Math.round(((end - start) / start) * 100);
      const weeks = growth.length;
      return `${first} ${end > start ? "Up" : "Down"} ${Math.abs(percent)}% in subscribers over ${weeks} weeks.`;
    }
  }
  return first;
}

/** Long videos and Shorts as whole percentages that add up to 100. */
export function mix(long: number, short: number): { long: number; short: number } {
  const total = long + short;
  if (total === 0) return { long: 0, short: 0 };
  const longPercent = Math.round((long / total) * 100);
  return { long: longPercent, short: 100 - longPercent };
}
