// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// "This week": how full each day is, Monday to Sunday. Today is outlined; a
// full day from today on turns honey with a gentle note. No scores, no streaks.

import type { HomeView } from "../../lib/commands";
import { dayString } from "../../lib/days";
import { weekNote } from "../../lib/home";

export function WeekStrip({ view }: { view: HomeView }) {
  const today = dayString(new Date());
  const note = weekNote(view.week, new Date());
  return (
    <section className="card week-card" aria-labelledby="this-week">
      <div className="section-head">
        <h2 id="this-week">This week</h2>
        <span className="muted small">How full each day is</span>
      </div>
      <ol className="week-strip">
        {view.week.map((d) => {
          const [y, m, day] = d.day.split("-").map(Number);
          const name = new Date(y, m - 1, day).toLocaleDateString("en-GB", { weekday: "short" });
          const height = Math.min(d.count / view.day_capacity, 1) * 100;
          const isToday = d.day === today;
          // Honey asks you to think before saying yes, so only days ahead get it.
          const full = d.full && d.day >= today;
          return (
            <li key={d.day} className={isToday ? "today" : undefined} aria-label={`${name}: ${d.count} ${d.count === 1 ? "thing" : "things"}${full ? ", full" : ""}`}>
              <div className="week-bar">
                <div className={`week-fill${full ? " full" : ""}${isToday ? " now" : ""}`} style={{ height: `${height}%` }} />
              </div>
              {name}
            </li>
          );
        })}
      </ol>
      {note && <p className="note">{note}</p>}
      <div className="week-counts">
        <span>Kept <b className="kept-count">{view.this_week.kept}</b></span>
        <span>Moved <b>{view.this_week.moved}</b></span>
        <span>Let go <b>{view.this_week.let_go}</b></span>
      </div>
    </section>
  );
}
