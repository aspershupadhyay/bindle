// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// "Running late": honey, never red (handbook principle 3). Each one offers a
// way forward: do it, move it to a day that works, or let it go.

import { useState } from "react";
import { commands, errorMessage, type Promise as BindlePromise } from "../../lib/commands";
import { dayString, addDays } from "../../lib/days";
import { lateWords } from "../../lib/home";
import { useNotify, useOfferUndo } from "../Undo";
import { WhenPicker, type WhenValue } from "../WhenPicker";

export function RunningLate({ promises }: { promises: BindlePromise[] }) {
  if (promises.length === 0) return null;
  return (
    <section aria-labelledby="late">
      <div className="section-head">
        <h2 id="late">Running late</h2>
        <span className="muted small">{promises.length}</span>
      </div>
      <div className="card late-card">
        {promises.map((p) => (
          <LateRow key={p.id} promise={p} />
        ))}
      </div>
    </section>
  );
}

function LateRow({ promise }: { promise: BindlePromise }) {
  const offerUndo = useOfferUndo();
  const notify = useNotify();
  const [moving, setMoving] = useState(false);
  const [when, setWhen] = useState<WhenValue>({
    day: dayString(addDays(new Date(), 1)),
    time: "",
    isDeadline: promise.due !== null,
    soft: false,
  });

  async function act(label: string, run: () => globalThis.Promise<import("../../lib/commands").Change>) {
    try {
      offerUndo(label, await run());
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  return (
    <div className="late-row">
      <div className="late-line">
        <button type="button" className="tick" aria-label={`Mark done: ${promise.title}`} onClick={() => void act(`Done: ${promise.title}`, () => commands.markDone(promise.id))} />
        <span className="promise-title">{promise.title}</span>
        <span className="late-words">
          <span className="late-dot" aria-hidden="true" />
          {lateWords(promise, new Date())}
        </span>
      </div>
      {moving ? (
        <div className="late-actions move">
          <WhenPicker value={when} onChange={setWhen} allowSoft={false} />
          <div className="late-buttons">
            <button
              type="button"
              className="button-primary small-primary"
              disabled={!when.day}
              onClick={() =>
                void act("Moved", () =>
                  commands.movePromise(promise.id, { day: when.day, time: when.time ? `${when.time}:00` : null }, when.isDeadline),
                )
              }
            >
              Move it
            </button>
            <button type="button" className="button-quiet" onClick={() => setMoving(false)}>
              Cancel
            </button>
          </div>
        </div>
      ) : (
        <div className="late-actions">
          <button type="button" className="button-quiet" onClick={() => setMoving(true)}>
            Move it
          </button>
          <button type="button" className="button-quiet" onClick={() => void act("Let go", () => commands.letGo(promise.id))}>
            Let go
          </button>
          <span className="muted small">{promise.person ? `A short update to ${promise.person} keeps the trust.` : "A new day that works is better than a day that didn't."}</span>
        </div>
      )}
    </div>
  );
}
