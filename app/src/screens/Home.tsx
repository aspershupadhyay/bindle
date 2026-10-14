// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Life Home (v2 design, Main). This first version has the greeting, Jot down
// and what is waiting in Sort. Top 3, Running late and Waiting on others
// arrive with week 4.

import { forwardRef, useState, type FormEvent } from "react";
import hills from "../art/hills.png";
import { Icon } from "../components/Icon";
import { commands, errorMessage, type Promise as BindlePromise } from "../lib/commands";
import { dateLine, greeting } from "../lib/days";

interface HomeProps {
  unsorted: BindlePromise[];
  onSaved: () => void;
}

export const Home = forwardRef<HTMLInputElement, HomeProps>(function Home({ unsorted, onSaved }, jotRef) {
  const now = new Date();
  return (
    <div className="page">
      <header className="home-header">
        <div className="home-hello">
          <div className="muted small">{dateLine(now)}</div>
          <h1 className="display">{greeting(now)}.</h1>
          <p className="lead">{summary(unsorted.length)}</p>
          <JotField ref={jotRef} onSaved={onSaved} />
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
              <li key={p.id}>{p.title}</li>
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

type Status = { kind: "idle" } | { kind: "saving" } | { kind: "saved" } | { kind: "error"; message: string };

/** The Jot down field. "Saved" shows only after the database says so. */
const JotField = forwardRef<HTMLInputElement, { onSaved: () => void }>(function JotField({ onSaved }, ref) {
  const [text, setText] = useState("");
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (text.trim() === "" || status.kind === "saving") return;
    setStatus({ kind: "saving" });
    try {
      await commands.jotDown(text);
      setText("");
      setStatus({ kind: "saved" });
      onSaved();
    } catch (error) {
      setStatus({ kind: "error", message: errorMessage(error) });
    }
  }

  return (
    <form className="jot" onSubmit={submit}>
      <label className="jot-field">
        <Icon name="pen" size={18} />
        <span className="visually-hidden">Jot down a promise</span>
        <input
          ref={ref}
          type="text"
          value={text}
          onChange={(e) => {
            setText(e.target.value);
            if (status.kind !== "saving") setStatus({ kind: "idle" });
          }}
          placeholder="Jot down… try “send reel edit to Rohan kal”"
          autoComplete="off"
        />
        <span className="key-hint">Enter</span>
      </label>
      <div className={`jot-status ${status.kind}`} role="status" aria-live="polite">
        {status.kind === "saving" && "Saving…"}
        {status.kind === "saved" && "Saved"}
        {status.kind === "error" && `Not saved: ${status.message}`}
      </div>
    </form>
  );
});
