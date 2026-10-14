// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// One open promise: a box to mark it done, its first step, chips, and its
// history on request.

import { useState } from "react";
import { chipsForPromise } from "../lib/chips";
import { commands, errorMessage, type HistoryEntry, type Promise as BindlePromise } from "../lib/commands";
import { Chips } from "./Chips";
import { useOfferUndo } from "./Undo";

const historyWords: Record<string, string> = {
  created: "Jotted down",
  sorted: "Sorted",
  edited: "Changed",
  moved: "Moved",
  done: "Done",
  reopened: "Opened again",
  let_go: "Let go",
  later: "Kept for later",
  undone: "Undone",
};

export function PromiseRow({ promise }: { promise: BindlePromise }) {
  const offerUndo = useOfferUndo();
  const [problem, setProblem] = useState<string | null>(null);

  async function done() {
    try {
      offerUndo(`Done: ${promise.title}`, await commands.markDone(promise.id));
    } catch (error) {
      setProblem(errorMessage(error));
    }
  }

  return (
    <li className="promise-row">
      <button type="button" className="tick" aria-label={`Mark done: ${promise.title}`} onClick={() => void done()} />
      <div className="promise-main">
        <span className="promise-title">{promise.title}</span>
        {promise.first_step && <span className="muted small">First step: {promise.first_step}</span>}
        {problem && <span className="error-text small">{problem}</span>}
        <History id={promise.id} />
      </div>
      <Chips chips={chipsForPromise(promise, new Date())} label="Details" />
    </li>
  );
}

function History({ id }: { id: string }) {
  const [entries, setEntries] = useState<HistoryEntry[] | null>(null);
  return (
    <details
      className="history"
      onToggle={(e) => {
        if ((e.target as HTMLDetailsElement).open) commands.history(id).then(setEntries, () => setEntries([]));
      }}
    >
      <summary>History</summary>
      {entries && (
        <ol>
          {entries.map((entry) => (
            <li key={entry.id} className={entry.undone ? "undone" : undefined}>
              {historyWords[entry.kind] ?? entry.kind}
              <span className="muted"> · {new Date(entry.at).toLocaleString("en-GB", { dateStyle: "medium", timeStyle: "short" })}</span>
              {entry.undone && <span className="muted"> · undone</span>}
            </li>
          ))}
        </ol>
      )}
    </details>
  );
}
