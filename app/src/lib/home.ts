// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Words for Life Home (handbook P3). Calm, specific, never shaming.

import type { DayLoad, Promise as BindlePromise } from "./commands";

/** "Monday" within the last week, otherwise "2 Oct". */
export function pastDayWords(day: Date, today: Date): string {
  const days = Math.round((startOf(today).getTime() - startOf(day).getTime()) / 86_400_000);
  if (days === 0) return "today";
  if (days === 1) return "yesterday";
  if (days > 1 && days < 7) return day.toLocaleDateString("en-GB", { weekday: "long" });
  return `${day.getDate()} ${day.toLocaleDateString("en-GB", { month: "short" })}`;
}

/** "Running late since Monday". */
export function lateWords(promise: BindlePromise, today: Date): string {
  const when = promise.due ?? promise.do_on;
  return when ? `Running late since ${pastDayWords(localDay(when.day), today)}` : "Running late";
}

/** "They'll do it · since Thursday · nudged yesterday". */
export function waitingWords(promise: BindlePromise, lastNudged: string | null, today: Date): string {
  const asked = new Date(promise.created_at);
  const recent = startOf(today).getTime() - startOf(asked).getTime() < 7 * 86_400_000;
  const parts = ["They'll do it", `${recent ? "since" : "promised"} ${pastDayWords(asked, today)}`];
  const due = promise.due ?? promise.do_on;
  if (due) parts.push(dueWords(localDay(due.day), today));
  if (lastNudged) parts.push(`nudged ${pastDayWords(new Date(lastNudged), today)}`);
  return parts.join(" · ");
}

/** The note under the week strip, about the first full day from today on. */
export function weekNote(week: DayLoad[], today: Date): string | null {
  const todayText = dayText(today);
  const full = week.find((d) => d.full && d.day >= todayText);
  if (!full) return null;
  const name = full.day === todayText ? "Today" : localDay(full.day).toLocaleDateString("en-GB", { weekday: "long" });
  return `${name} already has ${full.count} things. Think before you say yes.`;
}

/** A friendly message the person can paste to whoever owes them. */
export function nudgeMessage(promise: BindlePromise): string {
  const what = promise.title.charAt(0).toLowerCase() + promise.title.slice(1);
  return `Hi${promise.person ? ` ${promise.person}` : ""}, just checking in on ${what}. Thanks!`;
}

/** "Rohan · pay the invoice", or the title alone when there is no person. */
export function withPerson(promise: BindlePromise): string {
  if (!promise.person) return promise.title;
  return `${promise.person} · ${promise.title.charAt(0).toLowerCase()}${promise.title.slice(1)}`;
}

/** "due Friday", or "was due Friday" once that day has passed. */
function dueWords(day: Date, today: Date): string {
  const days = Math.round((startOf(day).getTime() - startOf(today).getTime()) / 86_400_000);
  if (days < 0) return `was due ${pastDayWords(day, today)}`;
  if (days === 0) return "due today";
  if (days === 1) return "due tomorrow";
  if (days < 7) return `due ${day.toLocaleDateString("en-GB", { weekday: "long" })}`;
  return `due ${day.getDate()} ${day.toLocaleDateString("en-GB", { month: "short" })}`;
}

function startOf(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

function localDay(text: string): Date {
  const [y, m, d] = text.split("-").map(Number);
  return new Date(y, m - 1, d);
}

function dayText(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}
