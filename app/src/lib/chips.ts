// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The small labels next to a promise: when, how much, with whom (handbook P3).

import type { Direction, Money, Promise as BindlePromise, Reading, When } from "./commands";

export interface Chip {
  key: string;
  label: string;
  /** "check" asks the person to confirm; "soft" marks a soft promise. */
  tone: "plain" | "check" | "soft";
  /** Longer words for screen readers and tooltips. */
  hint?: string;
}

const directionWords: Record<Direction, string> = {
  mine: "I'll do it",
  theirs: "They'll do it",
  self: "Just for me",
};

/** Chips for a line being typed, before it is saved. */
export function chipsForReading(reading: Reading, today: Date): Chip[] {
  const chips: Chip[] = [];
  if (reading.when) {
    const label = whenLabel(reading.when, today, reading.is_deadline);
    chips.push(
      reading.needs_check
        ? { key: "when", label: `${label}?`, tone: "check", hint: `You typed “${reading.unsure_words ?? label}”. Sort will ask to be sure.` }
        : { key: "when", label, tone: "plain" },
    );
  }
  return [...chips, ...restOf(reading)];
}

/** Chips for a saved promise. */
export function chipsForPromise(promise: BindlePromise, today: Date): Chip[] {
  const chips: Chip[] = [];
  if (promise.due) chips.push({ key: "when", label: whenLabel(promise.due, today, true), tone: "plain" });
  else if (promise.do_on) chips.push({ key: "when", label: whenLabel(promise.do_on, today, false), tone: "plain" });
  return [...chips, ...restOf(promise)];
}

function restOf(p: { amount: Money | null; person: string | null; direction: Direction | null; strength: string }): Chip[] {
  const chips: Chip[] = [];
  if (p.amount) chips.push({ key: "amount", label: moneyLabel(p.amount), tone: "plain" });
  if (p.person) chips.push({ key: "person", label: p.person, tone: "plain" });
  if (p.direction) chips.push({ key: "direction", label: directionWords[p.direction], tone: "plain" });
  if (p.strength === "soft") chips.push({ key: "soft", label: "Sometime", tone: "soft" });
  return chips;
}

/** "Today", "Tomorrow 8:30 PM", "Fri", "By Fri 23 Oct". */
export function whenLabel(when: When, today: Date, deadline: boolean): string {
  const day = parseDay(when.day);
  const start = new Date(today.getFullYear(), today.getMonth(), today.getDate());
  const days = Math.round((day.getTime() - start.getTime()) / 86_400_000);
  const weekday = day.toLocaleDateString("en-GB", { weekday: "short" });
  let label: string;
  if (days === 0) label = "Today";
  else if (days === 1) label = "Tomorrow";
  else if (days > 1 && days < 7) label = weekday;
  else label = `${weekday} ${day.getDate()} ${day.toLocaleDateString("en-GB", { month: "short" })}`;
  if (when.time) label += ` ${timeLabel(when.time)}`;
  return deadline ? `By ${label.charAt(0).toLowerCase() === "t" ? label.toLowerCase() : label}` : label;
}

/** "20:30:00" becomes "8:30 PM"; "09:00:00" becomes "9 AM". */
export function timeLabel(time: string): string {
  const [h, m] = time.split(":").map(Number);
  const half = h < 12 ? "AM" : "PM";
  const hour = h % 12 === 0 ? 12 : h % 12;
  return m === 0 ? `${hour} ${half}` : `${hour}:${String(m).padStart(2, "0")} ${half}`;
}

/** "₹2,340", "₹99.50", "$12". Indian grouping for rupees. */
export function moneyLabel(money: Money): string {
  const whole = money.minor % 100 === 0;
  return new Intl.NumberFormat(money.currency === "INR" ? "en-IN" : "en-GB", {
    style: "currency",
    currency: money.currency,
    minimumFractionDigits: whole ? 0 : 2,
    maximumFractionDigits: whole ? 0 : 2,
  }).format(money.minor / 100);
}

/** Reads `YYYY-MM-DD` as a local day, not UTC midnight. */
function parseDay(text: string): Date {
  const [y, m, d] = text.split("-").map(Number);
  return new Date(y, m - 1, d);
}
