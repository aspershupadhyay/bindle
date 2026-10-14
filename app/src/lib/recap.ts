// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Send recap (handbook P6, People): a short, friendly summary of where things
// stand with one person, for them to read. Bindle copies it; the person sends
// it themselves (handbook principle 6). Dates are written out in full, because
// the message is read later, when "tomorrow" would mean something else.

import { moneyLabel } from "./chips";
import type { PersonDetail, Promise as BindlePromise } from "./commands";

export function recapMessage(person: PersonDetail, today: Date): string | null {
  const lines: string[] = [];
  if (person.you_owe.length > 0) {
    lines.push("From me:", ...person.you_owe.map((p) => `• ${line(p, today)}`));
  }
  if (person.they_owe.length > 0) {
    if (lines.length > 0) lines.push("");
    lines.push("From you:", ...person.they_owe.map((p) => `• ${line(p, today)}`));
  }
  if (person.kept_lately.length > 0) {
    if (lines.length > 0) lines.push("");
    lines.push("Done lately:", ...person.kept_lately.map((p) => `• ${p.title}`));
  }
  if (lines.length === 0) return null;
  return [`Hi ${person.name}, a quick recap of where we are:`, "", ...lines, "", "Thanks!"].join("\n");
}

/** "Pay the invoice (₹18,000), was due Fri 9 Oct". */
function line(p: BindlePromise, today: Date): string {
  let text = p.title;
  // Titles can already name the sum ("Pay the ₹18,000 invoice"); say it once.
  if (p.amount && !text.includes(moneyLabel(p.amount))) text += ` (${moneyLabel(p.amount)})`;
  const when = p.due ?? p.do_on;
  if (when) {
    const day = localDay(when.day);
    const written = day.toLocaleDateString("en-GB", { weekday: "short", day: "numeric", month: "short" }).replace(",", "");
    const past = day < new Date(today.getFullYear(), today.getMonth(), today.getDate());
    text += p.due ? `, ${past ? "was due" : "by"} ${written}` : `, ${past ? "planned for" : "on"} ${written}`;
  }
  return text;
}

function localDay(text: string): Date {
  const [y, m, d] = text.split("-").map(Number);
  return new Date(y, m - 1, d);
}
