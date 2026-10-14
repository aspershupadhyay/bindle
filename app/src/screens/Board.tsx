// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Work, Board (v2 design, Board; handbook P14): work promises in lanes.
// Ideas are kept for later, Next are open, Making are being worked on now,
// Waiting are owed by someone else, Shipped were kept this week. Each card
// moves with buttons, so the Board works from the keyboard too.

import { useCallback, useEffect, useState } from "react";
import workshop from "../art/workshop.png";
import { Chips } from "../components/Chips";
import { useNotify, useOfferUndo } from "../components/Undo";
import { WhenPicker, type WhenValue } from "../components/WhenPicker";
import { chipsForPromise } from "../lib/chips";
import { commands, errorMessage, onPromisesChanged, type BoardCard, type BoardView } from "../lib/commands";
import { addDays, dayString } from "../lib/days";
import { useSettings } from "../lib/settings";
import { boardSummary, cardFrom, cardSource, countWord } from "../lib/work";

type Filter = "all" | "code" | "inbox" | "other";
type Lane = "ideas" | "next" | "making" | "waiting" | "shipped";

const filters: { value: Filter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "code", label: "Code" },
  { value: "inbox", label: "Inbox" },
  { value: "other", label: "Other" },
];

const lanes: { lane: Lane; title: string; note: string }[] = [
  { lane: "ideas", title: "Ideas", note: "Kept for later" },
  { lane: "next", title: "Next", note: "Soonest first" },
  { lane: "making", title: "Making", note: "" },
  { lane: "waiting", title: "Waiting", note: "On other people, not you" },
  { lane: "shipped", title: "Shipped", note: "This week" },
];

export function Board() {
  const { settings } = useSettings();
  const [view, setView] = useState<BoardView | null>(null);
  const [filter, setFilter] = useState<Filter>("all");
  const notify = useNotify();

  const refresh = useCallback(() => {
    commands.boardView().then(setView, (e) => notify(errorMessage(e)));
  }, [notify]);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  const limit = settings.making_limit;
  const shown = (cards: BoardCard[]) => (filter === "all" ? cards : cards.filter((c) => cardSource(c.from) === filter));
  const empty = view && Object.values(view).every((cards) => cards.length === 0);

  return (
    <div className="page board-page">
      <header className="code-header">
        <div className="code-hello">
          <div className="muted small">Work · Board</div>
          <h1 className="display">Board</h1>
          {view && <p className="lead">{boardSummary(view, limit)}</p>}
          <p className="muted small">Work promises land here once they are sorted, from Code, Inbox or Jot down in Work.</p>
        </div>
        <figure className="scene-frame">
          <img className="sprite scene" src={workshop} alt="Pixel art: a woodworker's workshop with tools on the wall" width={384} height={160} />
        </figure>
      </header>

      {empty ? (
        <p className="card empty">
          No work promises yet. Switch to Work and jot one down, or make one from Code or Inbox; once it is sorted it shows up here.
        </p>
      ) : (
        view && (
          <>
            <div className="pills" role="group" aria-label="Show">
              {filters.map((f) => (
                <button key={f.value} type="button" className="pill" aria-pressed={filter === f.value} onClick={() => setFilter(f.value)}>
                  {f.label}
                </button>
              ))}
            </div>
            <div className="board-lanes">
              {lanes.map(({ lane, title, note }) => {
                const cards = shown(view[lane]);
                const over = lane === "making" && view.making.length > limit;
                return (
                  <section key={lane} className={`board-lane ${over ? "over" : ""}`} aria-labelledby={`lane-${lane}`}>
                    <div className="board-lane-head">
                      <h2 id={`lane-${lane}`}>{title}</h2>
                      <span className="muted small">{lane === "making" ? `${view.making.length} of ${limit}` : cards.length}</span>
                    </div>
                    {lane === "making" ? (
                      <p className="muted small">
                        {over
                          ? `${countWord(view.making.length - limit)} more than your limit. Finish one before starting another.`
                          : "What you are working on now."}
                      </p>
                    ) : (
                      <p className="muted small">{note}</p>
                    )}
                    <ul className="board-cards">
                      {cards.map((card) => (
                        <CardView key={card.promise.id} card={card} lane={lane} />
                      ))}
                    </ul>
                  </section>
                );
              })}
            </div>
          </>
        )
      )}
    </div>
  );
}

function CardView({ card, lane }: { card: BoardCard; lane: Lane }) {
  const offerUndo = useOfferUndo();
  const notify = useNotify();
  const [planning, setPlanning] = useState(false);
  const [when, setWhen] = useState<WhenValue>({ day: dayString(addDays(new Date(), 1)), time: "", isDeadline: false, soft: false });
  const p = card.promise;
  const from = cardFrom(card.from);

  const making = (on: boolean) => commands.setMaking(p.id, on).catch((e) => notify(errorMessage(e)));
  async function done() {
    try {
      offerUndo(`Shipped: ${p.title}`, await commands.markDone(p.id));
    } catch (error) {
      notify(errorMessage(error));
    }
  }
  async function plan() {
    try {
      offerUndo("Planned", await commands.movePromise(p.id, { day: when.day, time: when.time ? `${when.time}:00` : null }, when.isDeadline));
      setPlanning(false);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  return (
    <li className={`card board-card ${lane === "shipped" ? "shipped" : ""}`}>
      <span className="board-card-title">{p.title}</span>
      {from && <span className="muted small">{from}</span>}
      {lane !== "shipped" && <Chips chips={chipsForPromise(p, new Date()).filter((c) => c.key !== "mode" && c.key !== "direction")} label="Details" />}
      <div className="board-card-actions">
        {lane === "ideas" && !planning && (
          <button type="button" className="button-quiet small-button" onClick={() => setPlanning(true)} aria-label={`Plan it: ${p.title}`}>
            Plan it
          </button>
        )}
        {lane === "next" && (
          <button type="button" className="button-quiet small-button" onClick={() => void making(true)} aria-label={`Start: ${p.title}`}>
            Start
          </button>
        )}
        {lane === "making" && (
          <button type="button" className="button-quiet small-button" onClick={() => void making(false)} aria-label={`Back to Next: ${p.title}`}>
            Not now
          </button>
        )}
        {(lane === "next" || lane === "making" || lane === "waiting") && (
          <button type="button" className="button-quiet small-button" onClick={() => void done()} aria-label={`Done: ${p.title}`}>
            Done
          </button>
        )}
      </div>
      {planning && (
        <div className="board-plan">
          <WhenPicker value={when} onChange={setWhen} allowSoft={false} />
          <div className="update-actions">
            <button type="button" className="button-primary small-button" disabled={!when.day} onClick={() => void plan()}>
              Plan it
            </button>
            <button type="button" className="button-quiet small-button" onClick={() => setPlanning(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}
    </li>
  );
}
