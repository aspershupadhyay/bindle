// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Life Home (v2 design, Main). This first version has the greeting, Jot down
// and what is waiting in Sort. Top 3, Running late and Waiting on others
// arrive with week 4.

import { forwardRef } from "react";
import hills from "../art/hills.png";
import { Chips } from "../components/Chips";
import { JotField } from "../components/JotField";
import { chipsForPromise } from "../lib/chips";
import type { Promise as BindlePromise } from "../lib/commands";
import { dateLine, greeting } from "../lib/days";

interface HomeProps {
  unsorted: BindlePromise[];
}

export const Home = forwardRef<HTMLInputElement, HomeProps>(function Home({ unsorted }, jotRef) {
  const now = new Date();
  return (
    <div className="page">
      <header className="home-header">
        <div className="home-hello">
          <div className="muted small">{dateLine(now)}</div>
          <h1 className="display">{greeting(now)}.</h1>
          <p className="lead">{summary(unsorted.length)}</p>
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

      <section aria-labelledby="in-sort">
        <div className="section-head">
          <h2 id="in-sort">Waiting in Sort</h2>
          <span className="muted small">{unsorted.length === 0 ? "Nothing yet" : `${unsorted.length}, oldest first`}</span>
        </div>
        {unsorted.length === 0 ? (
          <p className="card empty">
            Jot down anything you promised, or anything someone promised you. It waits here until you sort it.
          </p>
        ) : (
          <ul className="card list">
            {unsorted.map((p) => (
              <li key={p.id}>
                <span className="promise-title">{p.title}</span>
                <Chips chips={chipsForPromise(p, now)} label="Details" />
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
});

function summary(unsorted: number): string {
  if (unsorted === 0) return "Nothing needs you right now.";
  return unsorted === 1 ? "1 thing waiting in Sort." : `${unsorted} things waiting in Sort.`;
}
