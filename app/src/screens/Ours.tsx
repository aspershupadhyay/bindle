// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Ours (v2 design, Ours; handbook P15): a shared space. Promises either of
// you add, who added and kept them, who carries the space, and an invite
// link that works once. No scores, no blame, just a shared bag.

import { useCallback, useEffect, useState, type FormEvent } from "react";
import house from "../art/house.png";
import { Chips } from "../components/Chips";
import { useNotify, useOfferUndo } from "../components/Undo";
import { chipsForPromise } from "../lib/chips";
import { commands, errorMessage, onPromisesChanged, type OursView, type Promise as BindlePromise, type SharedPromise } from "../lib/commands";
import { lookedAgo } from "../lib/work";

type Filter = "all" | "mine" | "theirs";

export function Ours({ spaceId, onLeft }: { spaceId: string; onLeft: () => void }) {
  const [view, setView] = useState<OursView | null>(null);
  const [filter, setFilter] = useState<Filter>("all");
  const [syncing, setSyncing] = useState(false);
  const notify = useNotify();

  const refresh = useCallback(() => {
    commands.oursView(spaceId).then(setView, (e) => notify(errorMessage(e)));
  }, [spaceId, notify]);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  async function syncNow() {
    setSyncing(true);
    try {
      await commands.cloudSync();
    } catch (error) {
      notify(errorMessage(error));
    } finally {
      setSyncing(false);
      refresh();
    }
  }

  if (!view) return <div className="page" />;
  const others = view.members.filter((m) => m.account !== view.me).map((m) => m.name);
  const shown = view.open.filter((s) => (filter === "all" ? true : filter === "mine" ? s.added_by === null : s.added_by !== null));
  const together = others.length > 0 ? `you and ${others.join(" and ")} carry this one together.` : "invite someone to carry this one with you.";

  return (
    <div className="page ours-page">
      <header className="code-header">
        <div className="code-hello">
          <div className="muted small">Shared space</div>
          <h1 className="display">{view.space.name}</h1>
          <p className="lead">{together.charAt(0).toUpperCase() + together.slice(1)}</p>
          <p className="muted small code-synced" role="status">
            {lookedAgo(view.space.synced_at, new Date()).replace("Looked", "Synced").replace("Not looked yet", "Not synced yet")}
            {view.space.waiting > 0 && ` · ${view.space.waiting} to send`} · End-to-end encrypted ·{" "}
            <button type="button" className="link-button" disabled={syncing} onClick={() => void syncNow()}>
              {syncing ? "Syncing…" : "Sync now"}
            </button>
          </p>
        </div>
        <figure className="scene-frame">
          <img className="sprite scene" src={house} alt="Pixel art: a small house with two windows lit" width={384} height={168} />
        </figure>
      </header>

      <div className="home-columns">
        <div className="home-today">
          <JotToOurs spaceId={spaceId} name={view.space.name} />
          <section aria-labelledby="shared">
            <div className="section-head">
              <h2 id="shared">Shared promises</h2>
              <div className="pills" role="group" aria-label="Show">
                {(
                  [
                    ["all", `All ${view.open.length}`],
                    ["mine", "Added by me"],
                    ["theirs", others.length === 1 ? `Added by ${others[0]}` : "Added by others"],
                  ] as const
                ).map(([value, label]) => (
                  <button key={value} type="button" className="pill" aria-pressed={filter === value} onClick={() => setFilter(value)}>
                    {label}
                  </button>
                ))}
              </div>
            </div>
            {shown.length === 0 ? (
              <p className="card empty">Nothing here yet. Jot one down above, or share one of yours below.</p>
            ) : (
              <ul className="card list">
                {shown.map((s) => (
                  <SharedRow key={s.promise.id} shared={s} />
                ))}
              </ul>
            )}
          </section>
          <ShareExisting spaceId={spaceId} name={view.space.name} />
          {view.kept_lately.length > 0 && (
            <section aria-labelledby="kept">
              <div className="section-head">
                <h2 id="kept">Kept lately</h2>
              </div>
              <ul className="card list">
                {view.kept_lately.map((s) => (
                  <li key={s.promise.id} className="promise-row">
                    <span className="tick-space" aria-hidden="true" />
                    <div className="promise-main">
                      <span className="promise-title done">{s.promise.title}</span>
                      <span className="muted small">Done by {s.done_by ?? "you"}</span>
                    </div>
                  </li>
                ))}
              </ul>
              <p className="muted small">
                Kept together this month: <strong>{view.kept_this_month}</strong>. No scores, no blame, just a shared bag.
              </p>
            </section>
          )}
        </div>

        <aside className="home-side">
          <section className="card desk-card" aria-labelledby="carry">
            <h2 id="carry">Who carries {view.space.name}</h2>
            <ul className="stack ours-members">
              {view.members.map((m) => (
                <li key={m.account}>
                  <strong>{m.name || "Someone"}</strong>
                  <span className="muted small">{m.account === view.me ? " · you" : ` · since ${new Date(m.joined_at).toLocaleDateString("en-GB", { day: "numeric", month: "short" })}`}</span>
                </li>
              ))}
            </ul>
            <p className="muted small">Each of you is yourself. {view.space.name} is shared; your Mine stays yours.</p>
          </section>
          <Invite spaceId={spaceId} name={view.space.name} />
          <section className="card desk-card" aria-labelledby="safety">
            <h2 id="safety">Safety</h2>
            <p className="muted small">Locked on your device before it is sent. Bindle Cloud cannot read {view.space.name}.</p>
            <p className="muted small">Works offline. Changes go up when you are back online.</p>
            <button
              type="button"
              className="link-button small"
              onClick={() => {
                if (!window.confirm(`Leave ${view.space.name} on this device? Its promises stay here, in Mine. The others keep theirs.`)) return;
                commands.oursLeave(spaceId).then(onLeft, (e) => notify(errorMessage(e)));
              }}
            >
              Leave {view.space.name} on this device
            </button>
          </section>
        </aside>
      </div>
    </div>
  );
}

function SharedRow({ shared }: { shared: SharedPromise }) {
  const offerUndo = useOfferUndo();
  const notify = useNotify();
  const p = shared.promise;
  const who = shared.added_by ? `Added by ${shared.added_by}` : "Added by you";
  return (
    <li className="promise-row">
      {p.state === "open" ? (
        <button
          type="button"
          className="tick"
          aria-label={`Mark done: ${p.title}`}
          onClick={() => commands.markDone(p.id).then((c) => offerUndo(`Done: ${p.title}`, c), (e) => notify(errorMessage(e)))}
        />
      ) : (
        <span className="tick-space" aria-hidden="true" />
      )}
      <div className="promise-main">
        <span className="promise-title">{p.title}</span>
        <span className="muted small">
          {who}
          {p.state === "unsorted" && " · waiting in Sort"}
          {p.first_step && ` · first step: ${p.first_step}`}
        </span>
      </div>
      <div className="row-end">
        <Chips chips={chipsForPromise(p, new Date())} label="Details" />
      </div>
    </li>
  );
}

function JotToOurs({ spaceId, name }: { spaceId: string; name: string }) {
  const notify = useNotify();
  const [text, setText] = useState("");
  async function save(e: FormEvent) {
    e.preventDefault();
    if (!text.trim()) return;
    try {
      await commands.oursJot(spaceId, text);
      setText("");
      notify(`Saved to ${name}. It waits in Sort for whoever keeps it.`);
    } catch (error) {
      notify(errorMessage(error));
    }
  }
  return (
    <form className="jot-ours" onSubmit={(e) => void save(e)}>
      <label className="field">
        <span className="field-label">Jot down to {name}</span>
        <input value={text} onChange={(e) => setText(e.target.value)} placeholder="Pay electricity bill by Friday" />
      </label>
      <button type="submit" className="button-primary small-primary" disabled={!text.trim()}>
        Save
      </button>
    </form>
  );
}

function ShareExisting({ spaceId, name }: { spaceId: string; name: string }) {
  const notify = useNotify();
  const [query, setQuery] = useState("");
  const [found, setFound] = useState<BindlePromise[]>([]);
  useEffect(() => {
    if (!query.trim()) {
      setFound([]);
      return;
    }
    const timer = setTimeout(() => commands.searchPromises(query).then((all) => setFound(all.filter((p) => p.state !== "done" && p.state !== "let_go").slice(0, 5)), () => {}), 200);
    return () => clearTimeout(timer);
  }, [query]);
  return (
    <section className="card desk-card" aria-labelledby="share-existing">
      <h2 id="share-existing">Share one of your promises</h2>
      <label className="field">
        <span className="field-label">Find a promise</span>
        <input type="search" value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search your promises" />
      </label>
      {found.length > 0 && (
        <ul className="stack">
          {found.map((p) => (
            <li key={p.id} className="finished-row">
              <span>{p.title}</span>
              <button
                type="button"
                className="button-quiet small-button"
                onClick={() =>
                  commands.oursShare(p.id, spaceId).then(
                    () => {
                      notify(`Shared in ${name}`);
                      setQuery("");
                    },
                    (e) => notify(errorMessage(e)),
                  )
                }
              >
                Share
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function Invite({ spaceId, name }: { spaceId: string; name: string }) {
  const notify = useNotify();
  const [link, setLink] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  async function make() {
    setBusy(true);
    try {
      setLink(await commands.oursInvite(spaceId));
    } catch (error) {
      notify(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="card desk-card" aria-labelledby="invite">
      <h2 id="invite">Invite someone to {name}</h2>
      {link ? (
        <>
          <p className="kit-code small-code">{link}</p>
          <div className="update-actions">
            <button type="button" className="button-primary small-button" onClick={() => void navigator.clipboard?.writeText(link).then(() => notify("Invite link copied"))}>
              Copy link
            </button>
            <button type="button" className="button-quiet small-button" onClick={() => void make()}>
              New link
            </button>
          </div>
        </>
      ) : (
        <button type="button" className="button-quiet small-button" disabled={busy} onClick={() => void make()}>
          Make an invite link
        </button>
      )}
      <p className="muted small">It works once, for 7 days. Send it the way you like; they paste it in Settings → Bindle Cloud. The secret part of the link never reaches the server, so only the person you send it to can open {name}.</p>
    </section>
  );
}
