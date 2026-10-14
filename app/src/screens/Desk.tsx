// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Work, Desk (v2 design, Desk; handbook P14): what needs you now from every
// source, how long people wait on you compared with your own last four
// weeks, what shipped, how your sources are doing, and the week's work.
// Numbers are about you against yourself, never a score against others.

import { useCallback, useEffect, useState } from "react";
import station from "../art/station.png";
import { useNotify } from "../components/Undo";
import {
  commands,
  errorMessage,
  onMailChanged,
  onPromisesChanged,
  onWorkChanged,
  type DeskItem,
  type DeskView,
  type WaitTrend,
} from "../lib/commands";
import { dateLine, dayString } from "../lib/days";
import type { ScreenId } from "../screens";
import { age, compareWait, deskSummary, hoursLabel, lookedAgo, waitStory } from "../lib/work";

export function Desk({ onScreen, onOpenSort }: { onScreen: (id: ScreenId) => void; onOpenSort: () => void }) {
  const [view, setView] = useState<DeskView | null>(null);
  const notify = useNotify();

  const refresh = useCallback(() => {
    commands.deskView().then(setView, (e) => notify(errorMessage(e)));
  }, [notify]);

  useEffect(() => {
    refresh();
    const stops = [onWorkChanged(refresh), onMailChanged(refresh), onPromisesChanged(refresh)];
    return () => stops.forEach((stop) => stop());
  }, [refresh]);

  const now = new Date();
  if (!view) return <div className="page" />;
  const github = view.needs_you.filter((i) => i.source === "github").length;
  const email = view.needs_you.length - github;

  return (
    <div className="page desk-page">
      <header className="code-header">
        <div className="code-hello">
          <div className="muted small">Work · {dateLine(now)}</div>
          <h1 className="display">Your desk</h1>
          <p className="lead">{deskSummary(view, now)}</p>
        </div>
        <figure className="scene-frame">
          <img className="sprite scene" src={station} alt="Pixel art: a railway platform at dusk with a waiting train" width={384} height={160} />
        </figure>
      </header>

      <ul className="code-tiles desk-tiles" aria-label="At a glance">
        <li className="card code-tile">
          <span className="muted small">Needs you now</span>
          <span className="code-count">{view.needs_you.length}</span>
          <span className="muted small">GitHub {github} · Email {email}</span>
        </li>
        <WaitTile label="Email reply time, median" trend={view.email_reply} source="Gmail" />
        <WaitTile label="Review wait, median" trend={view.review_wait} source="GitHub" />
        <li className="card code-tile">
          <span className="muted small">Shipped this week</span>
          <span className="code-count">{view.shipped.this_week}</span>
          <span className="muted small">
            {bestNote(view.shipped.weekly) ?? `Code ${view.shipped.from_code} · Inbox ${view.shipped.from_inbox} · Other ${view.shipped.other}`}
          </span>
        </li>
      </ul>

      <div className="home-columns">
        <div className="home-today">
          <section aria-labelledby="needs-you">
            <div className="section-head">
              <h2 id="needs-you">
                Needs you now <span className="muted">· oldest first</span>
              </h2>
              <span className="desk-links">
                <button type="button" className="link-button small" onClick={() => onScreen("inbox")}>
                  Open Inbox
                </button>
                <button type="button" className="link-button small" onClick={() => onScreen("code")}>
                  Open Code
                </button>
              </span>
            </div>
            {view.needs_you.length === 0 ? (
              <p className="card empty">
                {view.github.login || view.gmail.address
                  ? "Nobody is waiting on you at work. Enjoy it."
                  : "Connect GitHub in Code and Gmail in Inbox, and whatever needs you shows up here."}
              </p>
            ) : (
              <ul className="card list">
                {view.needs_you.map((item) => (
                  <NeedsYouRow key={item.id} item={item} now={now} onOpenSort={onOpenSort} />
                ))}
              </ul>
            )}
          </section>

          <section aria-labelledby="who-waits" className="card desk-card">
            <h2 id="who-waits">Who waits longer</h2>
            <p className="muted">{waitStory(view.email_reply, view.review_wait)}</p>
            <div className="desk-sparks">
              <Spark title="Your email replies" weekly={view.email_reply.weekly} />
              <Spark title="Reviews you owe" weekly={view.review_wait.weekly} />
            </div>
            <WaitTable email={view.email_reply.weekly} reviews={view.review_wait.weekly} />
          </section>
        </div>

        <aside className="home-side" aria-label="This week">
          <section aria-labelledby="sources" className="card desk-card">
            <h2 id="sources">How your sources are doing</h2>
            <ul className="desk-sources">
              <SourceRow
                name="GitHub"
                connected={Boolean(view.github.login)}
                error={view.github.error}
                syncedAt={view.github.synced_at}
                line={view.reviews_older_than_a_day > 0 ? `${view.reviews_older_than_a_day} ${view.reviews_older_than_a_day === 1 ? "review" : "reviews"} older than a day` : "No review older than a day"}
                behind={view.reviews_older_than_a_day > 0}
                now={now}
                onOpen={() => onScreen("code")}
              />
              <SourceRow
                name="Gmail"
                connected={Boolean(view.gmail.address)}
                error={view.gmail.error}
                syncedAt={view.gmail.synced_at}
                line={view.replies_owed > 0 ? `${view.replies_owed} waiting on your reply` : "Nothing waiting on your reply"}
                behind={false}
                now={now}
                onOpen={() => onScreen("inbox")}
              />
            </ul>
          </section>
          <WeekLoad week={view.week} now={now} />
        </aside>
      </div>
    </div>
  );
}

/** "Best in 8 weeks" when this week beat every week shown, and something shipped. */
function bestNote(weekly: number[]): string | null {
  const last = weekly[weekly.length - 1] ?? 0;
  const before = weekly.slice(0, -1);
  return last > 0 && before.length > 0 && before.every((w) => w < last) ? `Best in ${weekly.length} weeks` : null;
}

function WaitTile({ label, trend, source }: { label: string; trend: WaitTrend; source: string }) {
  const compare = compareWait(trend);
  return (
    <li className="card code-tile">
      <span className="muted small">{label}</span>
      <span className="code-count">{trend.recent === null ? "–" : hoursLabel(trend.recent)}</span>
      <span className="muted small">
        {trend.recent === null ? `Not enough answers this week · ${source}` : compare ? `${compare}, your last 4 weeks · ${source}` : `This week · ${source}`}
      </span>
    </li>
  );
}

function NeedsYouRow({ item, now, onOpenSort }: { item: DeskItem; now: Date; onOpenSort: () => void }) {
  const notify = useNotify();
  const [busy, setBusy] = useState(false);
  const detail =
    item.kind === "review"
      ? `${item.reference} · review asked by ${item.person}`
      : item.kind === "issue"
        ? `${item.reference} · assigned to you`
        : `${item.person} · Gmail`;

  async function makePromise() {
    setBusy(true);
    try {
      await (item.source === "github" ? commands.makePromise(item.id) : commands.mailMakePromise(item.id));
      notify(`Sent to Sort: ${item.title}`);
    } catch (error) {
      notify(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }
  const open = () => (item.source === "github" ? commands.openOnGithub(item.url) : commands.openInGmail(item.url)).catch((e) => notify(errorMessage(e)));

  return (
    <li className="work-row">
      <div className="work-main">
        <span className="work-title">{item.title}</span>
        <span className="muted small">{detail}</span>
      </div>
      <div className="row-end">
        <span className="muted small work-age">{age(item.at, now)}</span>
        {item.promise?.state === "unsorted" ? (
          <button type="button" className="link-button small" onClick={onOpenSort}>
            Waiting in Sort
          </button>
        ) : item.promise && item.promise.state !== "let_go" ? (
          <span className="chip plain">A promise</span>
        ) : (
          <button type="button" className="button-quiet small-button" disabled={busy} onClick={() => void makePromise()} aria-label={`Make it a promise: ${item.title}`}>
            Make it a promise
          </button>
        )}
        <button type="button" className="button-quiet small-button" onClick={() => void open()} aria-label={`Open: ${item.title}`}>
          Open
        </button>
      </div>
    </li>
  );
}

function SourceRow(props: {
  name: string;
  connected: boolean;
  error: string | null;
  syncedAt: string | null;
  line: string;
  behind: boolean;
  now: Date;
  onOpen: () => void;
}) {
  const state = !props.connected ? "Not connected" : props.error ? "Needs a look" : props.behind ? "A little behind" : "Running smoothly";
  const tone = !props.connected ? "off" : props.error ? "attention" : props.behind ? "behind" : "fine";
  return (
    <li className="desk-source">
      <span className="desk-source-name">
        <strong>{props.name}</strong>
        <span className={`desk-state ${tone}`}>
          <span className="desk-dot" aria-hidden="true" />
          {state}
        </span>
      </span>
      <span className="muted small">
        {!props.connected ? (
          <button type="button" className="link-button small" onClick={props.onOpen}>
            Connect
          </button>
        ) : props.error ? (
          props.error
        ) : (
          `${props.line} · ${lookedAgo(props.syncedAt, props.now).toLowerCase()}`
        )}
      </span>
    </li>
  );
}

/** Work promises per day this week: done (solid) and still planned (faded). */
function WeekLoad({ week, now }: { week: DeskView["week"]; now: Date }) {
  const most = Math.max(5, ...week.map((d) => d.open + d.done));
  const today = dayString(now);
  return (
    <section aria-labelledby="load" className="card desk-card">
      <h2 id="load">Work this week</h2>
      <ul className="desk-legend" aria-label="Key">
        <li>
          <span className="desk-swatch done" aria-hidden="true" />
          Done
        </li>
        <li>
          <span className="desk-swatch planned" aria-hidden="true" />
          Planned
        </li>
      </ul>
      <ol className="desk-week">
        {week.map((d) => {
          const name = new Date(`${d.day}T12:00:00`).toLocaleDateString("en-GB", { weekday: "short" });
          const tip = `${name}: ${d.done} done, ${d.open} planned`;
          return (
            <li key={d.day} className={d.day === today ? "today" : undefined} tabIndex={0} aria-label={tip} data-tip={tip}>
              <span className="desk-bar" aria-hidden="true">
                {d.done > 0 && <span className="desk-fill done" style={{ height: `${(d.done / most) * 100}%` }} />}
                {d.open > 0 && <span className="desk-fill planned" style={{ height: `${(d.open / most) * 100}%` }} />}
              </span>
              <span className="muted small" aria-hidden="true">
                {name}
              </span>
            </li>
          );
        })}
      </ol>
    </section>
  );
}

/** A small line of 8 weekly medians. Weeks with no answers leave a gap. */
function Spark({ title, weekly }: { title: string; weekly: (number | null)[] }) {
  const width = 240;
  const height = 64;
  const pad = 6;
  const known = weekly.filter((w): w is number => w !== null);
  const top = Math.max(1, ...known);
  const x = (i: number) => pad + (i * (width - pad * 2)) / Math.max(1, weekly.length - 1);
  const y = (h: number) => height - pad - (h / top) * (height - pad * 2);
  const segments: string[] = [];
  let current = "";
  weekly.forEach((w, i) => {
    if (w === null) {
      if (current) segments.push(current);
      current = "";
    } else {
      current += `${current ? "L" : "M"}${x(i).toFixed(1)},${y(w).toFixed(1)}`;
    }
  });
  if (current) segments.push(current);
  const last = weekly[weekly.length - 1];
  return (
    <figure className="desk-spark">
      <figcaption>
        <span>{title}</span>
        <span className="muted small">{last === null || last === undefined ? "No answers this week" : `This week ${hoursLabel(last)}`}</span>
      </figcaption>
      {known.length === 0 ? (
        <p className="muted small">Shows once there are answers to measure.</p>
      ) : (
        <svg viewBox={`0 0 ${width} ${height}`} role="img" aria-label={`${title}: median wait per week, last 8 weeks`}>
          <line x1={pad} x2={width - pad} y1={height - pad} y2={height - pad} className="desk-axis" />
          {segments.map((d) => (
            <path key={d} d={d} className="desk-line" />
          ))}
          {weekly.map((w, i) =>
            w === null ? null : (
              <circle key={i} cx={x(i)} cy={y(w)} r={4} className="desk-point">
                <title>{`${weekly.length - 1 - i === 0 ? "This week" : `${weekly.length - 1 - i} weeks ago`}: ${hoursLabel(w)}`}</title>
              </circle>
            ),
          )}
        </svg>
      )}
    </figure>
  );
}

function WaitTable({ email, reviews }: { email: (number | null)[]; reviews: (number | null)[] }) {
  const cell = (w: number | null | undefined) => (w === null || w === undefined ? "–" : hoursLabel(w));
  return (
    <details className="desk-table">
      <summary>See the numbers</summary>
      <table>
        <thead>
          <tr>
            <th scope="col">Week</th>
            <th scope="col">Email replies</th>
            <th scope="col">Reviews</th>
          </tr>
        </thead>
        <tbody>
          {email.map((w, i) => (
            <tr key={i}>
              <th scope="row">{email.length - 1 - i === 0 ? "This week" : `${email.length - 1 - i} weeks ago`}</th>
              <td>{cell(w)}</td>
              <td>{cell(reviews[i])}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </details>
  );
}
