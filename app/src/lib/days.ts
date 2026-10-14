// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Words for the time of day and the date line on Home (handbook P3).

/** "Good morning", "Good afternoon" or "Good evening" for the given time. */
export function greeting(now: Date): string {
  const hour = now.getHours();
  if (hour >= 5 && hour < 12) return "Good morning";
  if (hour >= 12 && hour < 17) return "Good afternoon";
  return "Good evening";
}

/** The date line above the greeting, e.g. "Wednesday, 14 October". */
export function dateLine(now: Date): string {
  // Built from parts: locale punctuation differs between systems.
  const weekday = now.toLocaleDateString("en-GB", { weekday: "long" });
  const month = now.toLocaleDateString("en-GB", { month: "long" });
  return `${weekday}, ${now.getDate()} ${month}`;
}

/** A local day as `YYYY-MM-DD`, the form the Rust side expects. */
export function dayString(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** The date `days` after `date` (midnight, local). */
export function addDays(date: Date, days: number): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate() + days);
}

/** The next `weekday` (0 is Sunday) after today, never today itself. */
export function nextWeekday(today: Date, weekday: number): Date {
  const ahead = (weekday - today.getDay() + 7) % 7 || 7;
  return addDays(today, ahead);
}

/** Quick picks for "when" in Sort and Move it. */
export function quickDays(today: Date): { label: string; day: string }[] {
  return [
    { label: "Today", day: dayString(today) },
    { label: "Tomorrow", day: dayString(addDays(today, 1)) },
    { label: "This weekend", day: dayString(today.getDay() === 6 ? today : nextWeekday(today, 6)) },
    { label: "Next week", day: dayString(nextWeekday(today, 1)) },
  ];
}
