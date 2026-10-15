// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Catch from a chat: paste a WhatsApp export, a Slack thread or an email and
// see every promise in it, yours and theirs. The chosen ones go to Sort.
// The chat is read on this computer and never saved.

import { useEffect, useRef, useState } from "react";
import { chipsForReading } from "../lib/chips";
import { commands, errorMessage, type Catch } from "../lib/commands";
import { Chips } from "./Chips";
import { Icon } from "./Icon";
import { useNotify } from "./Undo";

export function CatchChat({ onClose, onOpenSort }: { onClose: () => void; onOpenSort: () => void }) {
  const notify = useNotify();
  const [text, setText] = useState("");
  const [me, setMe] = useState<string | null>(null);
  const [result, setResult] = useState<Catch | null>(null);
  const [skipped, setSkipped] = useState<Set<number>>(new Set());
  const [busy, setBusy] = useState(false);
  const dialog = useRef<HTMLDivElement>(null);

  // Reads the chat a moment after typing or pasting stops.
  useEffect(() => {
    if (!text.trim()) {
      setResult(null);
      return;
    }
    const timer = setTimeout(() => {
      commands.catchChat(text, me).then(
        (found) => {
          setResult(found);
          setSkipped(new Set());
        },
        (e) => notify(errorMessage(e)),
      );
    }, 250);
    return () => clearTimeout(timer);
  }, [text, me, notify]);

  useEffect(() => {
    dialog.current?.querySelector("textarea")?.focus();
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  async function openFile(file: File | undefined) {
    if (file) setText(await file.text());
  }

  const chosen = result ? result.found.filter((_, i) => !skipped.has(i)) : [];
  async function keep() {
    setBusy(true);
    try {
      await commands.catchKeep(chosen.map((c) => ({ said: c.said, reading: c.reading })));
      notify(`${chosen.length} ${chosen.length === 1 ? "promise is" : "promises are"} waiting in Sort.`);
      onClose();
      onOpenSort();
    } catch (error) {
      notify(errorMessage(error));
      setBusy(false);
    }
  }

  const today = new Date();
  return (
    <div className="catch-backdrop" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div ref={dialog} className="card catch-dialog" role="dialog" aria-modal="true" aria-labelledby="catch-title">
        <div className="catch-head">
          <div>
            <h2 id="catch-title" className="catch-title">Catch promises from a chat</h2>
            <p className="muted small">
              Paste a WhatsApp chat, a Slack thread or an email. Bindle finds what each of you said you would do. It is read on this
              computer and never saved.
            </p>
          </div>
          <button type="button" className="link-button small" onClick={onClose}>
            Close
          </button>
        </div>

        <label className="field">
          <span className="field-label">The chat</span>
          <textarea
            className="catch-text"
            value={text}
            onChange={(e) => setText(e.target.value)}
            rows={7}
            spellCheck={false}
            placeholder={"[12/10/26, 9:15 PM] Rohan: I'll pay you back the 500 kal\n[12/10/26, 9:20 PM] You: I'll send the trip photos by Friday"}
          />
        </label>
        <div className="catch-tools">
          <label className="button-quiet small-button catch-file">
            <Icon name="promises" size={16} />
            Open a WhatsApp export (.txt)
            <input type="file" accept=".txt,text/plain" className="visually-hidden" onChange={(e) => void openFile(e.target.files?.[0])} />
          </label>
          <span className="muted small">In WhatsApp: open the chat, then More, Export chat, Without media.</span>
        </div>

        {result && result.speakers.length > 1 && (
          <div className="field">
            <span className="field-label" id="who-you">Which one is you?</span>
            <div className="pills" role="group" aria-labelledby="who-you">
              {result.speakers.map((s) => (
                <button key={s} type="button" className="pill" aria-pressed={result.me === s} onClick={() => setMe(s)}>
                  {s}
                </button>
              ))}
            </div>
          </div>
        )}

        {result && (
          <section aria-labelledby="caught" className="catch-found">
            <h3 id="caught" className="small">
              {result.found.length === 0
                ? "No promises found yet."
                : `${result.found.length} ${result.found.length === 1 ? "promise" : "promises"} found`}
            </h3>
            {result.found.length === 0 && (
              <p className="muted small">Bindle looks for “I’ll…”, “will do”, “bhej dunga”, and a “sure” after “can you…?”.</p>
            )}
            <ul className="card list catch-list">
              {result.found.map((c, i) => (
                <li key={`${c.reading.title}-${i}`}>
                  <label className="catch-row">
                    <input
                      type="checkbox"
                      checked={!skipped.has(i)}
                      onChange={() => {
                        const next = new Set(skipped);
                        if (next.has(i)) next.delete(i);
                        else next.add(i);
                        setSkipped(next);
                      }}
                    />
                    <span className="promise-main">
                      <span className="promise-title">{c.reading.title}</span>
                      <span className="muted small catch-said">“{c.said}”</span>
                      <Chips chips={chipsForReading(c.reading, today)} label="What Bindle understood" />
                    </span>
                  </label>
                </li>
              ))}
            </ul>
          </section>
        )}

        <div className="catch-actions">
          <span className="muted small">They wait in Sort, so you decide each one.</span>
          <button type="button" className="button-primary" disabled={busy || chosen.length === 0} onClick={() => void keep()}>
            {chosen.length === 0 ? "Add to Sort" : `Add ${chosen.length} to Sort`}
          </button>
        </div>
      </div>
    </div>
  );
}
