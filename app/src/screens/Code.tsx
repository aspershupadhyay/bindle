// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Work, Code (v2 design, Code; handbook P14): the reviews people are waiting
// on, your pull requests waiting on others, and the issues assigned to you,
// read from GitHub. Any of them can become a promise, which waits in Sort.
// Bindle only reads: it never comments, merges or closes anything.

import { useCallback, useEffect, useState, type FormEvent } from "react";
import arcade from "../art/arcade.png";
import { useNotify, useOfferUndo } from "../components/Undo";
import { commands, errorMessage, onPromisesChanged, onWorkChanged, type CodeView, type WorkItem } from "../lib/commands";
import { age, codeSummary, lookedAgo, promiseNote, shortName } from "../lib/work";

export function Code({ onOpenSort }: { onOpenSort: () => void }) {
  const [view, setView] = useState<CodeView | null>(null);
  const [looking, setLooking] = useState(false);
  const notify = useNotify();

  const refresh = useCallback(() => {
    commands.codeView().then(setView, (e) => notify(errorMessage(e)));
  }, [notify]);

  useEffect(() => {
    refresh();
    const stopWork = onWorkChanged(refresh);
    const stopPromises = onPromisesChanged(refresh);
    return () => {
      stopWork();
      stopPromises();
    };
  }, [refresh]);

  async function lookNow() {
    setLooking(true);
    try {
      await commands.githubRefresh();
    } catch (error) {
      notify(errorMessage(error));
    } finally {
      setLooking(false);
      refresh();
    }
  }

  const now = new Date();
  const connected = Boolean(view?.account.login);

  return (
    <div className="page code-page">
      <header className="code-header">
        <div className="code-hello">
          <div className="muted small">Work · Code · GitHub</div>
          <h1 className="display">Code</h1>
          {view && connected && <p className="lead">{codeSummary(view, now)}</p>}
          {view && connected && (
            <p className="muted small code-synced" role="status">
              {lookedAgo(view.account.synced_at, now)}
              {" · "}
              <button type="button" className="link-button" disabled={looking} onClick={() => void lookNow()}>
                {looking ? "Looking…" : "Look now"}
              </button>
            </p>
          )}
        </div>
        <figure className="scene-frame">
          <img className="sprite scene" src={arcade} alt="Pixel art: a cosy arcade at night" width={384} height={160} />
        </figure>
      </header>

      {view && !connected && <Connect onConnected={refresh} />}

      {view && connected && (
        <>
          {view.account.error && (
            <p className="card code-problem" role="alert">
              {view.account.error}
            </p>
          )}
          <ul className="code-tiles" aria-label="At a glance">
            <Tile label="Reviews you owe" count={view.reviews.length} note={view.reviews_older_than_a_day > 0 ? `${view.reviews_older_than_a_day} older than a day` : "None older than a day"} />
            <Tile label="Your pull requests waiting" count={view.mine.length} note={view.mine.some((m) => m.draft) ? `${view.mine.filter((m) => m.draft).length} still a draft` : "Waiting on others"} />
            <Tile label="Issues assigned" count={view.issues.length} note={view.issues.length > 0 ? "Most recently active first" : "Nothing assigned"} />
          </ul>

          {view.finished.length > 0 && <Finished items={view.finished} />}

          <ItemList
            id="reviews"
            title="Reviews you owe"
            note="Aim for a first review within a day"
            empty="Nobody is waiting on a review from you."
            items={view.reviews}
            now={now}
            onOpenSort={onOpenSort}
          />
          <ItemList
            id="mine"
            title="Your pull requests, waiting on others"
            note="You are blocked until they move"
            empty="None of your pull requests are open."
            items={view.mine}
            now={now}
            onOpenSort={onOpenSort}
          />
          <ItemList
            id="issues"
            title="Issues assigned to you"
            note="Most recently active first"
            empty="No issues are assigned to you."
            items={view.issues}
            now={now}
            onOpenSort={onOpenSort}
          />

          <Disconnect login={view.account.login ?? ""} />
        </>
      )}
    </div>
  );
}

function Tile({ label, count, note }: { label: string; count: number; note: string }) {
  return (
    <li className="card code-tile">
      <span className="muted small">{label}</span>
      <span className="code-count">{count}</span>
      <span className="muted small">{note}</span>
    </li>
  );
}

function ItemList(props: {
  id: string;
  title: string;
  note: string;
  empty: string;
  items: WorkItem[];
  now: Date;
  onOpenSort: () => void;
}) {
  return (
    <section aria-labelledby={props.id}>
      <div className="section-head">
        <h2 id={props.id}>
          {props.title} <span className="muted">· {props.items.length}</span>
        </h2>
        <span className="muted small">{props.note}</span>
      </div>
      {props.items.length === 0 ? (
        <p className="card empty">{props.empty}</p>
      ) : (
        <ul className="card list">
          {props.items.map((item) => (
            <ItemRow key={item.id} item={item} now={props.now} onOpenSort={props.onOpenSort} />
          ))}
        </ul>
      )}
    </section>
  );
}

function ItemRow({ item, now, onOpenSort }: { item: WorkItem; now: Date; onOpenSort: () => void }) {
  const notify = useNotify();
  const [busy, setBusy] = useState(false);
  const note = promiseNote(item);

  async function makePromise() {
    setBusy(true);
    try {
      await commands.makePromise(item.id);
      notify(`Sent to Sort: ${shortName(item)}`);
    } catch (error) {
      notify(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <li className="work-row">
      <div className="work-main">
        <span className="work-title">{item.title}</span>
        <span className="muted small">
          {shortName(item)}
          {item.kind !== "mine" && ` · ${item.author}`}
          {item.draft && " · draft"}
          {item.labels.length > 0 && ` · ${item.labels.join(", ")}`}
        </span>
      </div>
      <div className="row-end">
        <span className="muted small work-age" title={`Opened ${new Date(item.created_at).toLocaleString()}`}>
          {age(item.created_at, now)}
        </span>
        {note === "Waiting in Sort" ? (
          <button type="button" className="link-button small" onClick={onOpenSort}>
            {note}
          </button>
        ) : note ? (
          <span className="chip plain">{note}</span>
        ) : (
          <button type="button" className="button-quiet small-button" disabled={busy} onClick={() => void makePromise()} aria-label={`Make it a promise: ${item.title}`}>
            Make it a promise
          </button>
        )}
        <button
          type="button"
          className="button-quiet small-button"
          aria-label={`Open on GitHub: ${item.title}`}
          onClick={() => commands.openOnGithub(item.url).catch((e) => notify(errorMessage(e)))}
        >
          Open
        </button>
      </div>
    </li>
  );
}

/** Items GitHub no longer lists while their promise is still open. */
function Finished({ items }: { items: WorkItem[] }) {
  const offerUndo = useOfferUndo();
  const notify = useNotify();

  async function close(item: WorkItem) {
    const promise = item.promise;
    if (!promise) return;
    try {
      // A promise still in Sort was never planned, so it is let go, not done.
      const change = promise.state === "unsorted" ? await commands.letGo(promise.id) : await commands.markDone(promise.id);
      offerUndo(promise.state === "unsorted" ? `Let go: ${shortName(item)}` : `Done: ${shortName(item)}`, change);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  return (
    <section aria-labelledby="finished" className="card code-finished">
      <h2 id="finished">Finished on GitHub?</h2>
      <p className="muted small">GitHub no longer lists these. If the promise is kept, mark it so.</p>
      <ul className="stack">
        {items.map((item) => (
          <li key={item.id} className="finished-row">
            <span>
              {item.kind === "review" ? "Review for " : item.kind === "issue" ? "Issue " : "Pull request "}
              <strong>{shortName(item)}</strong>: {item.title}
            </span>
            <button type="button" className="button-quiet small-button" onClick={() => void close(item)}>
              {item.promise?.state === "unsorted" ? "Let it go" : "Mark done"}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

function Connect({ onConnected }: { onConnected: () => void }) {
  const notify = useNotify();
  const [token, setToken] = useState("");
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  async function openTokenPage() {
    try {
      await commands.openOnGithub(await commands.githubTokenPage());
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  async function connect(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setProblem(null);
    try {
      const account = await commands.githubConnect(token);
      setToken("");
      notify(`Connected to GitHub as ${account.login}`);
      onConnected();
    } catch (error) {
      setProblem(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="card code-connect" aria-labelledby="connect">
      <h2 id="connect">Connect GitHub</h2>
      <p className="muted">
        See the reviews people are waiting on, your pull requests and your issues, and make any of them a promise. Bindle only reads. It
        never comments, merges or closes anything.
      </p>
      <ol className="connect-steps">
        <li>
          <button type="button" className="button-quiet small-button" onClick={() => void openTokenPage()}>
            Make a token on GitHub
          </button>
          <span className="muted small"> The page opens with the right box ticked (repo). Choose how long it lasts, then make it.</span>
        </li>
        <li>
          <form className="connect-form" onSubmit={(e) => void connect(e)}>
            <label className="field">
              <span className="field-label">Paste the token here</span>
              <input type="password" autoComplete="off" spellCheck={false} value={token} onChange={(e) => setToken(e.target.value)} placeholder="ghp_…" />
            </label>
            <button type="submit" className="button-primary small-primary" disabled={busy || token.trim() === ""}>
              {busy ? "Checking…" : "Connect"}
            </button>
          </form>
        </li>
      </ol>
      {problem && (
        <p className="error-text small" role="alert">
          {problem}
        </p>
      )}
      <p className="muted small">The token stays in this computer's keychain. It is never saved in Bindle's data, its exports or its safety copies.</p>
    </section>
  );
}

function Disconnect({ login }: { login: string }) {
  const notify = useNotify();
  return (
    <p className="muted small code-account">
      Connected to GitHub as <strong>{login}</strong>.{" "}
      <button
        type="button"
        className="link-button"
        onClick={() =>
          commands.githubDisconnect().then(
            () => notify("Disconnected from GitHub. Promises you made stay."),
            (e) => notify(errorMessage(e)),
          )
        }
      >
        Disconnect
      </button>
    </p>
  );
}
