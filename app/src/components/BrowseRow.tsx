// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A promise in any state, with the one action that fits it: open ones can be
// marked done, later ones planned, done and let-go ones opened again.

import { useState } from "react";
import { chipsForPromise } from "../lib/chips";
import { addDays, dayString } from "../lib/days";
import { commands, errorMessage, type Change, type Promise as BindlePromise, type State } from "../lib/commands";
import { Chips } from "./Chips";
import { useNotify, useOfferUndo } from "./Undo";
import { WhenPicker, type WhenValue } from "./WhenPicker";

const stateWords: Record<State, string> = {
  unsorted: "In Sort",
  open: "Open",
  later: "Later",
  done: "Done",
  let_go: "Let go",
};

interface BrowseRowProps {
  promise: BindlePromise;
  /** Show which state it is in, for mixed lists such as search results. */
  showState?: boolean;
  onOpenSort: () => void;
}

export function BrowseRow({ promise, showState = false, onOpenSort }: BrowseRowProps) {
  const offerUndo = useOfferUndo();
  const notify = useNotify();
  const [planning, setPlanning] = useState(false);
  const [when, setWhen] = useState<WhenValue>({ day: dayString(addDays(new Date(), 1)), time: "", isDeadline: false, soft: false });

  async function act(label: string, run: () => globalThis.Promise<Change>) {
    try {
      offerUndo(label, await run());
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  const chips = chipsForPromise(promise, new Date());
  const stateChip = showState ? [{ key: "state", label: stateWords[promise.state], tone: "plain" as const }] : [];
  const closed = promise.state === "done" || promise.state === "let_go";

  return (
    <li className="browse-row">
      <div className="browse-line">
        {promise.state === "open" ? (
          <button type="button" className="tick" aria-label={`Mark done: ${promise.title}`} onClick={() => void act(`Done: ${promise.title}`, () => commands.markDone(promise.id))} />
        ) : (
          <span className="tick-space" aria-hidden="true" />
        )}
        <span className="promise-main">
          <span className={closed ? "promise-title done" : "promise-title"}>{promise.title}</span>
          {promise.first_step && promise.state === "open" && <span className="muted small">First step: {promise.first_step}</span>}
        </span>
        <div className="row-end">
          <Chips chips={[...stateChip, ...chips]} label="Details" />
          {closed && (
            <button type="button" className="button-quiet small-button" onClick={() => void act("Opened again", () => commands.reopen(promise.id))}>
              Open again
            </button>
          )}
          {promise.state === "later" && !planning && (
            <button type="button" className="button-quiet small-button" onClick={() => setPlanning(true)}>
              Plan it
            </button>
          )}
          {promise.state === "unsorted" && (
            <button type="button" className="button-quiet small-button" onClick={onOpenSort}>
              Sort
            </button>
          )}
        </div>
      </div>
      {planning && (
        <div className="late-actions move">
          <WhenPicker value={when} onChange={setWhen} allowSoft={false} />
          <div className="late-buttons">
            <button
              type="button"
              className="button-primary small-primary"
              disabled={!when.day}
              onClick={() =>
                void act("Planned", () =>
                  commands.movePromise(promise.id, { day: when.day, time: when.time ? `${when.time}:00` : null }, when.isDeadline),
                )
              }
            >
              Plan it
            </button>
            <button type="button" className="button-quiet" onClick={() => setPlanning(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}
    </li>
  );
}
