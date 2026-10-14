// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Creator (v2 design, Creator; handbook P14): your YouTube channel's own
// numbers, the posts doing best lately, and the comments waiting on your
// reply. Any comment can become a promise in Sort. Bindle only reads public
// data and never replies or posts for you.

import { useCallback, useEffect, useState, type FormEvent } from "react";
import studio from "../art/studio.png";
import { useNotify } from "../components/Undo";
import { commands, errorMessage, onCreatorChanged, onPromisesChanged, type CreatorComment, type CreatorView } from "../lib/commands";
import { compact, creatorSummary, duration, mix, weekChange } from "../lib/creator";
import { age, lookedAgo } from "../lib/work";

export function Creator({ onOpenSort }: { onOpenSort: () => void }) {
  const notify = useNotify();
  const [view, setView] = useState<CreatorView | null>(null);
  const [looking, setLooking] = useState(false);
  const [hiddenNow, setHiddenNow] = useState<CreatorComment | null>(null);
  const [now, setNow] = useState(() => new Date());

  const refresh = useCallback(() => {
    setNow(new Date());
    commands.creatorView().then(setView, (e) => notify(errorMessage(e)));
  }, [notify]);

  useEffect(() => {
    refresh();
    const stopCreator = onCreatorChanged(refresh);
    const stopPromises = onPromisesChanged(refresh);
    return () => {
      stopCreator();
      stopPromises();
    };
  }, [refresh]);

  async function lookNow() {
    setLooking(true);
    try {
      await commands.youtubeRefresh();
    } catch (error) {
      notify(errorMessage(error));
    } finally {
      setLooking(false);
      refresh();
    }
  }

  async function hide(comment: CreatorComment) {
    try {
      await commands.commentHide(comment.id);
      setHiddenNow(comment);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  async function unhide() {
    if (!hiddenNow) return;
    await commands.commentUnhide(hiddenNow.id).catch((e) => notify(errorMessage(e)));
    setHiddenNow(null);
  }

  const channel = view?.account.found ?? null;
  const connected = Boolean(view?.account.channel);

  return (
    <div className="page creator-page">
      <header className="code-header">
        <div className="code-hello">
          <div className="muted small">Work · Creator{channel ? ` · YouTube · ${channel.title}` : ""}</div>
          <h1 className="display">{channel?.title ?? "Creator"}</h1>
          {view && connected && <p className="lead">{creatorSummary(view)}</p>}
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
          <img className="sprite scene" src={studio} alt="Pixel art: a small recording studio with a camera on a tripod and a ring light" width={384} height={160} />
        </figure>
      </header>

      {view && !connected && <ConnectYoutube onConnected={refresh} />}

      {view && connected && (
        <>
          {view.account.error && (
            <p className="card code-problem" role="alert">
              {view.account.error}
            </p>
          )}
          <ul className="code-tiles creator-tiles" aria-label="At a glance">
            <li className="card code-tile">
              <span className="muted small">Subscribers</span>
              <span className="code-count">{channel?.followers === null || channel?.followers === undefined ? "Hidden" : compact(channel.followers)}</span>
              <span className="muted small">{weekChange(view.followers_this_week, "subscribers") ?? "Changes show after a week"}</span>
            </li>
            <li className="card code-tile">
              <span className="muted small">Views, all time</span>
              <span className="code-count">{compact(channel?.views ?? 0)}</span>
              <span className="muted small">{weekChange(view.views_this_week, "views") ?? "Changes show after a week"}</span>
            </li>
            <li className="card code-tile">
              <span className="muted small">Views on recent posts</span>
              <span className="code-count">{compact(view.views_last_30_days)}</span>
              <span className="muted small">Posted in the last 30 days</span>
            </li>
            <li className="card code-tile">
              <span className="muted small">Comments waiting</span>
              <span className="code-count">{view.comments.filter((c) => !c.promise).length}</span>
              <span className="muted small">From the last two weeks</span>
            </li>
          </ul>

          <div className="home-columns">
            <div className="home-today">
              <Growth points={view.growth} />
              <section aria-labelledby="comments">
                <div className="section-head">
                  <h2 id="comments">Comments waiting on your reply</h2>
                </div>
                {hiddenNow && (
                  <p className="muted small" role="status">
                    Hidden in Bindle: {hiddenNow.author}'s comment.{" "}
                    <button type="button" className="link-button" onClick={() => void unhide()}>
                      Undo
                    </button>
                  </p>
                )}
                {view.comments.length === 0 ? (
                  <p className="card empty">Every comment from the last two weeks has your reply. Bindle looks again every hour.</p>
                ) : (
                  <ul className="card list">
                    {view.comments.map((c) => (
                      <CommentRow key={c.id} comment={c} now={now} onHide={() => void hide(c)} onOpenSort={onOpenSort} />
                    ))}
                  </ul>
                )}
              </section>
              <TopPosts view={view} />
            </div>
            <aside className="home-side">
              <WhatYouPost long={view.long_posts} short={view.short_posts} />
              <section className="card desk-card" aria-labelledby="not-yet">
                <h2 id="not-yet">Not in Bindle yet</h2>
                <p className="muted small">
                  Watch time, click-through, returning viewers and when people are around need YouTube Analytics, which needs Sign in with Google. Bindle reads only public numbers for now.
                </p>
                <p className="muted small">Instagram needs a Meta app and its review before Bindle can read it.</p>
              </section>
              <p className="muted small code-account">
                Reading <strong>{channel?.handle ?? channel?.title}</strong>. Bindle never replies or posts for you.{" "}
                <button
                  type="button"
                  className="link-button"
                  onClick={() =>
                    commands.youtubeDisconnect().then(
                      () => notify("Disconnected from YouTube. Promises you made stay."),
                      (e) => notify(errorMessage(e)),
                    )
                  }
                >
                  Disconnect
                </button>
              </p>
            </aside>
          </div>
        </>
      )}
    </div>
  );
}

function CommentRow({ comment, now, onHide, onOpenSort }: { comment: CreatorComment; now: Date; onHide: () => void; onOpenSort: () => void }) {
  const notify = useNotify();
  const [busy, setBusy] = useState(false);
  const inSort = comment.promise?.state === "unsorted";
  const promised = comment.promise && comment.promise.state !== "let_go";

  async function makePromise() {
    setBusy(true);
    try {
      await commands.commentMakePromise(comment.id);
      notify(`Sent to Sort: reply to ${comment.author}`);
    } catch (error) {
      notify(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <li className="work-row creator-comment">
      <div className="work-main">
        <span className="muted small">
          <strong>{comment.author}</strong> on {comment.post_title} · {age(comment.published_at, now)} ago
          {comment.likes > 0 && ` · ${comment.likes} ${comment.likes === 1 ? "like" : "likes"}`}
        </span>
        <span className="creator-comment-text">{comment.text}</span>
      </div>
      <div className="row-end">
        {inSort ? (
          <button type="button" className="link-button small" onClick={onOpenSort}>
            Waiting in Sort
          </button>
        ) : promised ? (
          <span className="chip plain">A promise</span>
        ) : (
          <button type="button" className="button-primary small-button" disabled={busy} onClick={() => void makePromise()}>
            Make it a promise
          </button>
        )}
        <button type="button" className="button-quiet small-button" onClick={onHide} aria-label={`Not needed: ${comment.author}'s comment`}>
          Not needed
        </button>
        <button
          type="button"
          className="button-quiet small-button"
          aria-label={`Open ${comment.author}'s comment on YouTube`}
          onClick={() => void commands.openOnYoutube(comment.url).catch((e) => notify(errorMessage(e)))}
        >
          Open
        </button>
      </div>
    </li>
  );
}

/** Subscribers at the end of each week, as one line over a light area. */
function Growth({ points }: { points: CreatorView["growth"] }) {
  const width = 560;
  const height = 180;
  const pad = { top: 12, right: 12, bottom: 24, left: 44 };
  if (points.length < 2) {
    return (
      <section className="card desk-card" aria-labelledby="growth">
        <h2 id="growth">Growth</h2>
        <p className="muted small">Bindle saves your subscriber count each day. The chart fills in after a second week.</p>
      </section>
    );
  }
  const values = points.map((p) => p.followers);
  const low = Math.min(...values);
  const high = Math.max(...values);
  const span = Math.max(1, high - low);
  const floor = Math.max(0, low - span * 0.15);
  const ceiling = high + span * 0.1;
  const x = (i: number) => pad.left + (i * (width - pad.left - pad.right)) / (points.length - 1);
  const y = (v: number) => pad.top + (1 - (v - floor) / (ceiling - floor)) * (height - pad.top - pad.bottom);
  const line = points.map((p, i) => `${i ? "L" : "M"}${x(i).toFixed(1)},${y(p.followers).toFixed(1)}`).join("");
  const area = `${line}L${x(points.length - 1).toFixed(1)},${height - pad.bottom}L${pad.left},${height - pad.bottom}Z`;
  const first = values[0];
  const last = values[values.length - 1];
  const change = first > 0 ? Math.round(((last - first) / first) * 100) : 0;
  const label = (week: string) => new Date(`${week}T12:00:00`).toLocaleDateString("en-GB", { day: "numeric", month: "short" });
  const ticks = [floor, (floor + ceiling) / 2, ceiling].map((v) => Math.round(v));
  return (
    <section className="card desk-card" aria-labelledby="growth">
      <div className="section-head">
        <h2 id="growth">Growth, {points.length} weeks</h2>
        <span className="muted small">
          {change === 0 ? "Steady" : `${change > 0 ? "+" : "−"}${Math.abs(change)}% in ${points.length} weeks`}
        </span>
      </div>
      <svg className="creator-growth" viewBox={`0 0 ${width} ${height}`} role="img" aria-label={`Subscribers at the end of each week, from ${compact(first)} to ${compact(last)}`}>
        {ticks.map((t) => (
          <g key={t}>
            <line x1={pad.left} x2={width - pad.right} y1={y(t)} y2={y(t)} className="creator-grid" />
            <text x={pad.left - 8} y={y(t) + 4} textAnchor="end" className="creator-tick">
              {compact(t)}
            </text>
          </g>
        ))}
        <path d={area} className="creator-area" />
        <path d={line} className="creator-line" />
        {points.map((p, i) => (
          <circle key={p.week} cx={x(i)} cy={y(p.followers)} r={5} className="creator-point">
            <title>{`Week of ${label(p.week)}: ${p.followers} subscribers`}</title>
          </circle>
        ))}
        {[0, points.length - 1].map((i) => (
          <text key={i} x={x(i)} y={height - 6} textAnchor={i === 0 ? "start" : "end"} className="creator-tick">
            {label(points[i].week)}
          </text>
        ))}
      </svg>
      <details className="desk-table">
        <summary>See the numbers</summary>
        <table>
          <thead>
            <tr>
              <th scope="col">Week of</th>
              <th scope="col">Subscribers</th>
            </tr>
          </thead>
          <tbody>
            {points.map((p) => (
              <tr key={p.week}>
                <th scope="row">{label(p.week)}</th>
                <td>{p.followers}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </details>
    </section>
  );
}

function TopPosts({ view }: { view: CreatorView }) {
  const notify = useNotify();
  if (view.top_posts.length === 0) return null;
  return (
    <section className="card desk-card" aria-labelledby="top-posts">
      <h2 id="top-posts">Top posts, last 30 days</h2>
      <table className="creator-posts">
        <thead>
          <tr>
            <th scope="col">Post</th>
            <th scope="col">Views</th>
            <th scope="col">Likes</th>
            <th scope="col">Comments</th>
          </tr>
        </thead>
        <tbody>
          {view.top_posts.map((p) => (
            <tr key={p.id}>
              <th scope="row">
                <button type="button" className="link-button" onClick={() => void commands.openOnYoutube(p.url).catch((e) => notify(errorMessage(e)))}>
                  {p.title}
                </button>
                <span className="muted small">
                  {p.short ? "Short" : "Video"} · {duration(p.duration_seconds)}
                </span>
              </th>
              <td>{compact(p.views)}</td>
              <td>{compact(p.likes)}</td>
              <td>{compact(p.comments)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

function WhatYouPost({ long, short }: { long: number; short: number }) {
  const share = mix(long, short);
  if (long + short === 0) return null;
  return (
    <section className="card desk-card" aria-labelledby="what-you-post">
      <h2 id="what-you-post">What you post</h2>
      <p className="muted small">Your last 90 days.</p>
      <dl className="creator-mix">
        {(
          [
            ["Long videos", share.long, long],
            ["Shorts", share.short, short],
          ] as const
        ).map(([name, percent, count]) => (
          <div key={name} className="creator-mix-row">
            <dt>{name}</dt>
            <dd>
              <span className="creator-mix-bar" aria-hidden="true">
                <span style={{ width: `${percent}%` }} />
              </span>
              <span>
                {percent}% <span className="muted small">({count})</span>
              </span>
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function ConnectYoutube({ onConnected }: { onConnected: () => void }) {
  const notify = useNotify();
  const [channel, setChannel] = useState("");
  const [key, setKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  async function openKeyPage() {
    try {
      await commands.openOnYoutube(await commands.youtubeKeyPage());
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  async function connect(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setProblem(null);
    try {
      const account = await commands.youtubeConnect(channel, key);
      setKey("");
      notify(`Connected to ${account.found?.title ?? "your channel"}`);
      onConnected();
    } catch (error) {
      setProblem(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="card code-connect" aria-labelledby="connect-youtube">
      <h2 id="connect-youtube">Connect YouTube</h2>
      <p className="muted">
        See your channel's numbers grow, which posts are doing best, and the comments still waiting on your reply. Bindle only reads public data. It
        never replies, likes or posts.
      </p>
      <ol className="connect-steps">
        <li>
          <span>Make an API key in Google Cloud, then turn on "YouTube Data API v3" for it. It is free for the few looks Bindle makes.</span>{" "}
          <button type="button" className="button-quiet small-button" onClick={() => void openKeyPage()}>
            Make an API key
          </button>
        </li>
        <li>
          <form className="connect-form" onSubmit={(e) => void connect(e)}>
            <label className="field">
              <span className="field-label">Your channel's handle</span>
              <input value={channel} onChange={(e) => setChannel(e.target.value)} placeholder="@yourchannel" spellCheck={false} autoCapitalize="off" />
            </label>
            <label className="field">
              <span className="field-label">The API key</span>
              <input type="password" autoComplete="off" spellCheck={false} value={key} onChange={(e) => setKey(e.target.value)} placeholder="AIza…" />
            </label>
            <button type="submit" className="button-primary small-primary" disabled={busy || !channel.trim() || !key.trim()}>
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
      <p className="muted small">The key stays in this computer's keychain. It is never saved in Bindle's data, its exports or its safety copies.</p>
    </section>
  );
}
