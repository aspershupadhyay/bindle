// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// "Your top 3": up to three things the person chose for today. Done ones stay,
// ticked, so the day shows what was kept. Suggestions only ever suggest.

import { chipsForPromise } from "../../lib/chips";
import { commands, errorMessage, type Promise as BindlePromise } from "../../lib/commands";
import { timeLabel } from "../../lib/chips";
import { Chips } from "../Chips";
import { Icon } from "../Icon";
import { useNotify, useOfferUndo } from "../Undo";

export function TopThree({ top3, suggestions }: { top3: BindlePromise[]; suggestions: BindlePromise[] }) {
  const kept = top3.filter((p) => p.state === "done").length;
  return (
    <section aria-labelledby="top3">
      <div className="section-head">
        <h2 id="top3">Your top 3</h2>
        {top3.length > 0 && <span className="muted small">{kept} of {top3.length} kept</span>}
      </div>
      <div className="stack">
        {top3.map((p) => (
          <TopRow key={p.id} promise={p} />
        ))}
        {top3.length < 3 && suggestions.length > 0 && (
          <div className="card suggestions">
            <p className="muted small">{top3.length === 0 ? "Pick up to three for today." : "Room for more today."}</p>
            {suggestions.map((p) => (
              <Suggestion key={p.id} promise={p} />
            ))}
          </div>
        )}
        {top3.length === 0 && suggestions.length === 0 && (
          <p className="card empty">Choose up to three things for today with “Top 3” on any promise below.</p>
        )}
      </div>
    </section>
  );
}

function TopRow({ promise }: { promise: BindlePromise }) {
  const offerUndo = useOfferUndo();
  const notify = useNotify();
  const done = promise.state === "done";

  async function toggle() {
    try {
      const change = done ? await commands.reopen(promise.id) : await commands.markDone(promise.id);
      offerUndo(done ? "Opened again" : `Done: ${promise.title}`, change);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  return (
    <div className="card top-row">
      <button
        type="button"
        className={done ? "tick ticked" : "tick"}
        aria-label={done ? `Done: ${promise.title}. Open again` : `Mark done: ${promise.title}`}
        onClick={() => void toggle()}
      >
        {done && <Icon name="check" size={14} />}
      </button>
      <span className={done ? "promise-title done" : "promise-title"}>{promise.title}</span>
      {done ? (
        <ul className="chips">
          <li className="chip kept">Done {promise.closed_at ? timeLabel(new Date(promise.closed_at).toTimeString().slice(0, 8)) : ""}</li>
        </ul>
      ) : (
        <Chips chips={chipsForPromise(promise, new Date())} label="Details" />
      )}
      <button
        type="button"
        className="icon-button"
        aria-label={`Take out of top 3: ${promise.title}`}
        title="Take out of top 3"
        onClick={() => commands.removeFromTop3(promise.id).catch((e) => notify(errorMessage(e)))}
      >
        <Icon name="close" size={14} />
      </button>
    </div>
  );
}

function Suggestion({ promise }: { promise: BindlePromise }) {
  const notify = useNotify();
  return (
    <div className="suggestion">
      <span className="promise-title">{promise.title}</span>
      <Chips chips={chipsForPromise(promise, new Date())} label="Details" />
      <button
        type="button"
        className="button-quiet small-button"
        onClick={() => commands.addToTop3(promise.id).catch((e) => notify(errorMessage(e)))}
      >
        Add
      </button>
    </div>
  );
}
