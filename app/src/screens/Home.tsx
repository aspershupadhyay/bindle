// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Life Home (v2 design, Main). This version has the greeting, Jot down, what
// is coming up and what waits in Sort. Top 3, Running late and Waiting on
// others arrive with week 4.

import { forwardRef, useCallback, useEffect, useState } from "react";
import hills from "../art/hills.png";
import { Chips } from "../components/Chips";
import { JotField } from "../components/JotField";
import { PromiseRow } from "../components/PromiseRow";
import { chipsForPromise } from "../lib/chips";
import { commands, onPromisesChanged, type Promise as BindlePromise } from "../lib/commands";
import { dateLine, greeting } from "../lib/days";

export const Home = forwardRef<HTMLInputElement, { onOpenSort: () => void }>(function Home({ onOpenSort }, jotRef) {
  const now = new Date();
  const [unsorted, setUnsorted] = useState<BindlePromise[]>([]);
  const [open, setOpen] = useState<BindlePromise[]>([]);

  const refresh = useCallback(() => {
    commands.listPromises("unsorted").then(setUnsorted, () => setUnsorted([]));
    commands.listPromises("open").then((list) => setOpen(byDay(list)), () => setOpen([]));
  }, []);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  return (
    <div className="page">
      <header className="home-header">
        <div className="home-hello">
          <div className="muted small">{dateLine(now)}</div>
          <h1 className="display">{greeting(now)}.</h1>
          <p className="lead">{summary(open.length, unsorted.length)}</p>
          <JotField ref={jotRef} />
        </div>
        <figure className="scene-frame">
          <img
            className="sprite scene"
            src={hills}
            alt="Pixel art: a traveller with a bindle walking over green hills at sunset"
            width={384}
            height={160}
          />
        </figure>
      </header>

      <section aria-labelledby="coming-up">
        <div className="section-head">
          <h2 id="coming-up">Coming up</h2>
          <span className="muted small">{open.length === 0 ? "Nothing yet" : `${open.length}, soonest first`}</span>
        </div>
        {open.length === 0 ? (
          <p className="card empty">Promises you keep in Sort show up here, soonest first.</p>
        ) : (
          <ul className="card list">
            {open.map((p) => (
              <PromiseRow key={p.id} promise={p} />
            ))}
          </ul>
        )}
      </section>

      {unsorted.length > 0 && (
        <section aria-labelledby="in-sort">
          <div className="section-head">
            <h2 id="in-sort">Waiting in Sort</h2>
            <button type="button" className="button-quiet small-button" onClick={onOpenSort}>
              Sort now
            </button>
          </div>
          <ul className="card list">
            {unsorted.map((p) => (
              <li key={p.id}>
                <span className="promise-title">{p.title}</span>
                <Chips chips={chipsForPromise(p, now)} label="Details" />
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
});

function summary(open: number, unsorted: number): string {
  const parts: string[] = [];
  if (open > 0) parts.push(open === 1 ? "1 promise coming up" : `${open} promises coming up`);
  if (unsorted > 0) parts.push(`${unsorted} waiting in Sort`);
  return parts.length === 0 ? "Nothing needs you right now." : `${parts.join(", ")}.`;
}

/** Soonest first; promises with no day (soft ones) go last. */
function byDay(list: BindlePromise[]): BindlePromise[] {
  const key = (p: BindlePromise) => {
    const when = p.due ?? p.do_on;
    return when ? `${when.day}T${when.time ?? "99"}` : "9999";
  };
  return [...list].sort((a, b) => key(a).localeCompare(key(b)));
}
