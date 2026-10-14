// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Wrap up (handbook P6 and P13): about one minute to close the day. What you
// kept, what is still open (done, tomorrow, or let go), tomorrow's top 3, and
// how full your head feels. That last answer stays private: never a score.

import { useCallback, useEffect, useState } from "react";
import logo from "../art/bindle-24.png";
import { Chips } from "../components/Chips";
import { Icon } from "../components/Icon";
import { useNotify, useOfferUndo } from "../components/Undo";
import { chipsForPromise, timeLabel } from "../lib/chips";
import { commands, errorMessage, onPromisesChanged, type Change, type Promise as BindlePromise, type WrapUpView } from "../lib/commands";
import { addDays, dateLine, dayString } from "../lib/days";

const loads = ["Light", "Calm", "Busy", "Heavy", "Full"];

export function WrapUp({ onDone }: { onDone: () => void }) {
  const [view, setView] = useState<WrapUpView | null>(null);
  const [mindLoad, setMindLoad] = useState<number | null>(null);
  const [note, setNote] = useState("");
  const [finished, setFinished] = useState(false);
  const notify = useNotify();

  const refresh = useCallback(() => {
    commands.wrapUpView().then(setView, (e) => notify(errorMessage(e)));
  }, [notify]);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  if (!view) return <div className="page" />;

  async function finish() {
    try {
      await commands.finishWrapUp(mindLoad, note.trim() || null);
      setFinished(true);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  if (finished) {
    return (
      <div className="page wrap-page">
        <div className="card all-sorted">
          <img className="sprite" src={logo} alt="" width={96} height={96} />
          <h1 className="display">Day closed.</h1>
          <p>Everything is in the bag. Rest well.</p>
          <button type="button" className="button-quiet" onClick={onDone}>Back to Home</button>
        </div>
      </div>
    );
  }

  const tomorrow = dayString(addDays(new Date(), 1));
  return (
    <div className="page wrap-page">
      <header className="sort-head">
        <div className="muted small">{dateLine(new Date())}</div>
        <h1 className="display">Wrap up</h1>
        <p className="lead">
          {view.wrapped_up_at
            ? `You wrapped up at ${timeLabel(new Date(view.wrapped_up_at).toTimeString().slice(0, 8))}. You can go again.`
            : "About one minute. Close the day so your head can rest."}
        </p>
      </header>

      <section aria-labelledby="kept">
        <div className="section-head">
          <h2 id="kept">Kept today</h2>
          <span className="muted small">{view.kept_today.length || ""}</span>
        </div>
        {view.kept_today.length === 0 ? (
          <p className="card empty">Nothing kept today. Some days are like that, and that is fine.</p>
        ) : (
          <ul className="card list">
            {view.kept_today.map((p) => (
              <li key={p.id}>
                <span className="tick ticked" aria-hidden="true"><Icon name="check" size={14} /></span>
                <span className="promise-title done">{p.title}</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section aria-labelledby="open">
        <div className="section-head">
          <h2 id="open">Still open</h2>
          <span className="muted small">{view.still_open.length === 0 ? "" : "Decide each one"}</span>
        </div>
        {view.still_open.length === 0 ? (
          <p className="card empty">Nothing left open. Nice and light.</p>
        ) : (
          <ul className="card list">
            {view.still_open.map((p) => (
              <OpenRow key={p.id} promise={p} tomorrow={tomorrow} />
            ))}
          </ul>
        )}
      </section>

      {view.tomorrow.length > 0 && (
        <section aria-labelledby="tomorrow">
          <div className="section-head">
            <h2 id="tomorrow">Tomorrow's top 3</h2>
            <span className="muted small">{view.tomorrow_top3.length} of 3 chosen</span>
          </div>
          <ul className="card list">
            {view.tomorrow.map((p) => (
              <TomorrowRow key={p.id} promise={p} day={tomorrow} chosen={view.tomorrow_top3.includes(p.id)} full={view.tomorrow_top3.length >= 3} />
            ))}
          </ul>
        </section>
      )}

      <section aria-labelledby="head" className="card mind">
        <h2 id="head">How full does your head feel?</h2>
        <div className="pills" role="group" aria-labelledby="head">
          {loads.map((label, i) => (
            <button key={label} type="button" className="pill" aria-pressed={mindLoad === i + 1} onClick={() => setMindLoad(mindLoad === i + 1 ? null : i + 1)}>
              {label}
            </button>
          ))}
        </div>
        <label className="field">
          <span className="field-label">Anything to set down before sleep? <span className="muted">Optional, only for you</span></span>
          <textarea value={note} onChange={(e) => setNote(e.target.value)} rows={2} />
        </label>
      </section>

      <div className="sort-actions">
        <button type="button" className="button-primary" onClick={() => void finish()}>Close the day</button>
      </div>
    </div>
  );
}

function OpenRow({ promise, tomorrow }: { promise: BindlePromise; tomorrow: string }) {
  const offerUndo = useOfferUndo();
  const notify = useNotify();
  async function act(label: string, run: () => globalThis.Promise<Change>) {
    try {
      offerUndo(label, await run());
    } catch (error) {
      notify(errorMessage(error));
    }
  }
  const when = promise.due ?? promise.do_on;
  return (
    <li className="wrap-row">
      <span className="promise-main">
        <span className="promise-title">{promise.title}</span>
        <Chips chips={chipsForPromise(promise, new Date())} label="Details" />
      </span>
      <span className="wrap-actions">
        <button type="button" className="button-quiet small-button" onClick={() => void act(`Done: ${promise.title}`, () => commands.markDone(promise.id))}>Done</button>
        <button
          type="button"
          className="button-quiet small-button"
          onClick={() => void act("Moved to tomorrow", () => commands.movePromise(promise.id, { day: tomorrow, time: when?.time ?? null }, promise.due !== null))}
        >
          Tomorrow
        </button>
        <button type="button" className="button-quiet small-button" onClick={() => void act("Let go", () => commands.letGo(promise.id))}>Let go</button>
      </span>
    </li>
  );
}

function TomorrowRow({ promise, day, chosen, full }: { promise: BindlePromise; day: string; chosen: boolean; full: boolean }) {
  const notify = useNotify();
  // The change is announced, so Wrap up refreshes by itself.
  const toggle = () =>
    (chosen ? commands.removeFromTop3(promise.id, day) : commands.addToTop3(promise.id, day)).catch((e) =>
      notify(errorMessage(e)),
    );
  return (
    <li>
      <span className="promise-title">{promise.title}</span>
      <button type="button" className="pill" aria-pressed={chosen} disabled={!chosen && full} onClick={() => void toggle()}>
        <Icon name="star" size={13} /> {chosen ? "In top 3" : "Top 3"}
      </button>
    </li>
  );
}
