// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Settings, Your data: export everything, and safety copies made once a day
// that can be put back. Putting one back copies today's data first.

import { useCallback, useEffect, useState } from "react";
import { commands, errorMessage, type SafetyCopy } from "../../lib/commands";
import { useNotify } from "../Undo";

const SHOWN = 5;

export function DataSection() {
  const notify = useNotify();
  const [copies, setCopies] = useState<SafetyCopy[]>([]);
  const [showAll, setShowAll] = useState(false);
  const [asking, setAsking] = useState<string | null>(null);

  const refresh = useCallback(() => {
    commands.safetyCopies().then(setCopies, (e) => notify(errorMessage(e)));
  }, [notify]);

  useEffect(refresh, [refresh]);

  const tryTo = async (work: () => globalThis.Promise<string>) => {
    try {
      notify(await work());
    } catch (error) {
      notify(errorMessage(error));
    }
    refresh();
  };

  const exportAs = (format: "json" | "csv") => tryTo(async () => `Saved to ${await commands.exportData(format)}`);
  const copyNow = () => tryTo(async () => (await commands.makeSafetyCopy(), "Safety copy made."));
  const restore = (copy: SafetyCopy) =>
    tryTo(async () => {
      setAsking(null);
      const before = await commands.restoreSafetyCopy(copy.file_name);
      return `Put back the copy from ${madeAt(copy)}. What you had before is the copy from ${madeAt(before)}.`;
    });

  return (
    <section className="card settings-card" aria-labelledby="data">
      <h2 id="data">Your data</h2>
      <p className="muted small">Everything is yours to take. JSON holds all of it; CSV holds your promises for a spreadsheet. Files go to Downloads.</p>
      <div className="pills">
        <button type="button" className="button-quiet" onClick={() => void exportAs("json")}>Export everything (JSON)</button>
        <button type="button" className="button-quiet" onClick={() => void exportAs("csv")}>Export promises (CSV)</button>
      </div>

      <h3 className="small">Safety copies</h3>
      <p className="muted small">Bindle copies your data once a day and keeps the last 14 copies on this device.</p>
      {copies.length === 0 ? (
        <p className="muted small">No copies yet. The first one is made today.</p>
      ) : (
        <ul className="copies">
          {(showAll ? copies : copies.slice(0, SHOWN)).map((copy) => (
            <li key={copy.file_name}>
              <span>
                {madeAt(copy)} <span className="muted small">{size(copy.bytes)}</span>
              </span>
              {asking === copy.file_name ? (
                <span className="pills">
                  <span className="small">Put this back? Today's data is copied first.</span>
                  <button type="button" className="button-primary small-primary" onClick={() => void restore(copy)}>Put it back</button>
                  <button type="button" className="button-quiet" onClick={() => setAsking(null)}>Cancel</button>
                </span>
              ) : (
                <button type="button" className="link-button small" onClick={() => setAsking(copy.file_name)}>Put back</button>
              )}
            </li>
          ))}
        </ul>
      )}
      <div className="pills">
        <button type="button" className="button-quiet" onClick={() => void copyNow()}>Make a copy now</button>
        {copies.length > SHOWN && (
          <button type="button" className="link-button small" onClick={() => setShowAll(!showAll)}>
            {showAll ? "Show fewer" : `Show all ${copies.length}`}
          </button>
        )}
      </div>
    </section>
  );
}

function madeAt(copy: SafetyCopy): string {
  const date = new Date(copy.made_at);
  return date.toLocaleString(undefined, { weekday: "short", day: "numeric", month: "short", hour: "numeric", minute: "2-digit" });
}

function size(bytes: number): string {
  return bytes < 1024 * 1024 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
