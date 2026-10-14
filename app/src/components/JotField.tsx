// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The Jot down field, used on Home and in the Jot down window.
// While typing, chips show what Bindle understood. "Saved" shows only after
// the database says so (handbook principle 2).

import { forwardRef, useEffect, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import { chipsForReading } from "../lib/chips";
import { commands, errorMessage, type Promise as BindlePromise, type Reading } from "../lib/commands";
import { Chips } from "./Chips";
import { Icon } from "./Icon";

type Status = { kind: "idle" } | { kind: "saving" } | { kind: "saved" } | { kind: "error"; message: string };

interface JotFieldProps {
  onSaved?: (promise: BindlePromise) => void;
  onEscape?: () => void;
  /** Set to a new value to empty the field and forget the last status. */
  resetKey?: number;
}

export const JotField = forwardRef<HTMLInputElement, JotFieldProps>(function JotField(
  { onSaved, onEscape, resetKey },
  ref,
) {
  const [text, setText] = useState("");
  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const reading = useReading(text);

  useEffect(() => {
    setText("");
    setStatus({ kind: "idle" });
  }, [resetKey]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (text.trim() === "" || status.kind === "saving") return;
    setStatus({ kind: "saving" });
    try {
      const promise = await commands.jotDown(text);
      setText("");
      setStatus({ kind: "saved" });
      onSaved?.(promise);
    } catch (error) {
      setStatus({ kind: "error", message: errorMessage(error) });
    }
  }

  function onKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape" && onEscape) {
      event.preventDefault();
      onEscape();
    }
  }

  const chips = reading && text.trim() !== "" ? chipsForReading(reading, new Date()) : [];
  return (
    <form className="jot" onSubmit={submit}>
      <label className="jot-field">
        <Icon name="pen" size={18} />
        <span className="visually-hidden">Jot down a promise</span>
        <input
          ref={ref}
          type="text"
          value={text}
          onKeyDown={onKeyDown}
          onChange={(e) => {
            setText(e.target.value);
            if (status.kind !== "saving") setStatus({ kind: "idle" });
          }}
          placeholder="Jot down… try “send reel edit to Rohan kal”"
          autoComplete="off"
          spellCheck={false}
        />
        <span className="key-hint">Enter</span>
      </label>
      <div className="jot-under">
        <Chips chips={chips} label="What Bindle understood" />
        <div className={`jot-status ${status.kind}`} role="status" aria-live="polite">
          {status.kind === "saving" && "Saving…"}
          {status.kind === "saved" && "Saved"}
          {status.kind === "error" && `Not saved: ${status.message}`}
        </div>
      </div>
    </form>
  );
});

/** Reads the text as it is typed. Late replies for older text are ignored. */
function useReading(text: string): Reading | null {
  const [reading, setReading] = useState<Reading | null>(null);
  const latest = useRef(0);
  useEffect(() => {
    const ask = ++latest.current;
    if (text.trim() === "") {
      setReading(null);
      return;
    }
    const timer = setTimeout(() => {
      commands.readJot(text).then(
        (r) => ask === latest.current && setReading(r),
        () => ask === latest.current && setReading(null),
      );
    }, 80);
    return () => clearTimeout(timer);
  }, [text]);
  return reading;
}
