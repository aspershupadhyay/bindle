// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// One promise being sorted. Everything Jot down understood is filled in; the
// person confirms or changes it, then keeps it, puts it off, or lets it go.

import { useEffect, useRef, useState, type FormEvent } from "react";
import { moneyLabel } from "../lib/chips";
import { commands, errorMessage, type Direction, type SortItem, type Sorting } from "../lib/commands";
import { useOfferUndo } from "./Undo";
import { WhenPicker, type WhenValue } from "./WhenPicker";

const directions: { value: Direction; label: string }[] = [
  { value: "mine", label: "I'll do it" },
  { value: "theirs", label: "They'll do it" },
  { value: "self", label: "Just for me" },
];

type Choice = "open" | "later" | "let_go";
const doneWords: Record<Choice, string> = { open: "Kept", later: "Kept for later", let_go: "Let go" };

export function SortCard({ item }: { item: SortItem }) {
  const { promise } = item;
  const planned = promise.due ?? promise.do_on;
  const [title, setTitle] = useState(promise.title);
  const [direction, setDirection] = useState<Direction | null>(promise.direction);
  const [person, setPerson] = useState(promise.person ?? "");
  const [firstStep, setFirstStep] = useState(promise.first_step ?? "");
  const [when, setWhen] = useState<WhenValue>({
    day: planned?.day ?? "",
    time: planned?.time?.slice(0, 5) ?? "",
    isDeadline: promise.due !== null,
    soft: promise.strength === "soft",
  });
  const [problem, setProblem] = useState<string | null>(null);
  // Any change may fix what the note asked for, so it goes until the next try.
  const changed = <T,>(set: (value: T) => void) => (value: T) => {
    set(value);
    setProblem(null);
  };
  const [busy, setBusy] = useState(false);
  const offerUndo = useOfferUndo();
  const keepButton = useRef<HTMLButtonElement>(null);

  // Focus "Keep it", not a field: Enter keeps, and Ctrl+Z still undoes.
  useEffect(() => keepButton.current?.focus(), []);

  async function choose(to: Choice) {
    if (busy) return;
    setBusy(true);
    setProblem(null);
    try {
      const change = await commands.sortPromise(promise.id, sorting(), to);
      offerUndo(doneWords[to], change);
    } catch (error) {
      setProblem(errorMessage(error));
      setBusy(false);
    }
  }

  function sorting(): Sorting {
    return {
      title,
      direction,
      strength: when.soft ? "soft" : "firm",
      person: person.trim() || null,
      first_step: firstStep.trim() || null,
      when: when.day && !when.soft ? { day: when.day, time: when.time ? `${when.time}:00` : null } : null,
      is_deadline: when.isDeadline,
    };
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    void choose("open");
  }

  const asksFirstStep = direction !== "theirs";
  return (
    <form className="card sort-card" onSubmit={submit} aria-label={`Sort: ${promise.title}`}>
      {item.suggested_by ? (
        <p className="suggested">
          <span className="suggested-by">Suggested by {item.suggested_by}</span>
          <span className="muted small">{item.reason ?? "Keep it only if it is yours."}</span>
        </p>
      ) : (
        item.typed && item.typed.trim() !== promise.title && <p className="muted small typed">You typed “{item.typed.trim()}”</p>
      )}
      {item.unsure_words && (
        <p className="note">
          “{item.unsure_words}” can mean more than one thing. Check the day and time below.
        </p>
      )}

      {promise.amount && <p className="muted small typed">Amount {moneyLabel(promise.amount)}, kept as it is.</p>}

      <label className="field">
        <span className="field-label">Promise</span>
        <input className="title-input" value={title} onChange={(e) => setTitle(e.target.value)} />
      </label>

      <div className="field">
        <span className="field-label" id="who">Who will do it</span>
        <div className="pills" role="group" aria-labelledby="who">
          {directions.map((d) => (
            <button key={d.value} type="button" className="pill" aria-pressed={direction === d.value} onClick={() => changed(setDirection)(d.value)}>
              {d.label}
            </button>
          ))}
        </div>
      </div>

      <label className="field">
        <span className="field-label">With someone? <span className="muted">Optional</span></span>
        <input value={person} onChange={(e) => setPerson(e.target.value)} placeholder="A name" />
      </label>

      <div className="field">
        <span className="field-label">When</span>
        <WhenPicker value={when} onChange={changed(setWhen)} />
      </div>

      {asksFirstStep && (
        <label className="field">
          <span className="field-label">First step <span className="muted">The very first small thing</span></span>
          <input value={firstStep} onChange={(e) => setFirstStep(e.target.value)} placeholder="Like “open the electricity app”" />
        </label>
      )}

      {problem && <p className="note" role="alert">{problem}</p>}

      <div className="sort-actions">
        <button ref={keepButton} type="submit" className="button-primary" disabled={busy}>Keep it</button>
        <button type="button" className="button-quiet" disabled={busy} onClick={() => void choose("later")}>Later</button>
        <button type="button" className="button-quiet" disabled={busy} onClick={() => void choose("let_go")}>Let go</button>
        <span className="muted small">Enter keeps it. Everything can be undone.</span>
      </div>
    </form>
  );
}
