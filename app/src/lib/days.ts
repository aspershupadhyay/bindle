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
