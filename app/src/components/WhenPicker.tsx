// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Picks when a promise happens: quick days, a date, an optional time, whether
// the day is a deadline, or "Sometime" for a soft promise with no day.

import { quickDays } from "../lib/days";

export interface WhenValue {
  /** `YYYY-MM-DD`, or "" for no day. */
  day: string;
  /** `HH:MM`, or "" for no time. */
  time: string;
  isDeadline: boolean;
  /** "Sometime": a soft promise that needs no day. */
  soft: boolean;
}

export function WhenPicker({ value, onChange }: { value: WhenValue; onChange: (next: WhenValue) => void }) {
  const set = (patch: Partial<WhenValue>) => onChange({ ...value, ...patch });
  return (
    <div className="when-picker">
      <div className="pills" role="group" aria-label="Quick days">
        {quickDays(new Date()).map((q) => (
          <button
            key={q.label}
            type="button"
            className="pill"
            aria-pressed={!value.soft && value.day === q.day}
            onClick={() => set({ day: q.day, soft: false })}
          >
            {q.label}
          </button>
        ))}
        <button
          type="button"
          className="pill"
          aria-pressed={value.soft}
          onClick={() => set({ soft: !value.soft, day: value.soft ? value.day : "", time: value.soft ? value.time : "" })}
        >
          Sometime
        </button>
      </div>
      {!value.soft && (
        <div className="when-exact">
          <label>
            <span className="visually-hidden">Day</span>
            <input type="date" value={value.day} onChange={(e) => set({ day: e.target.value })} />
          </label>
          <label>
            <span className="visually-hidden">Time, optional</span>
            <input type="time" value={value.time} onChange={(e) => set({ time: e.target.value })} />
          </label>
          <label className="check">
            <input type="checkbox" checked={value.isDeadline} onChange={(e) => set({ isDeadline: e.target.checked })} />
            It must be done by then
          </label>
        </div>
      )}
    </div>
  );
}
