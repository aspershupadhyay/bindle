// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A tiny stand-in for the Rust commands, used only when the views run in a
// plain browser. Nothing here is saved, nothing is parsed, and the lifecycle
// rules are not checked: the real ones live in Rust. It exists so screens can
// be tried and screenshotted without building the desktop app.

import type { Change, Promise as BindlePromise, Reading, SortItem, Sorting, State, When } from "./commands";

const promises: BindlePromise[] = [];
const changes = new Map<string, { before: BindlePromise; undone: boolean }>();
const listeners = new Map<string, Set<() => void>>();

/** Events between views in the preview, standing in for Tauri events. */
export const previewEvents = {
  on(name: string, handler: () => void): () => void {
    const set = listeners.get(name) ?? new Set();
    set.add(handler);
    listeners.set(name, set);
    return () => set.delete(handler);
  },
  emit(name: string) {
    listeners.get(name)?.forEach((handler) => handler());
  },
};

export async function preview<T>(name: string, args: Record<string, unknown> = {}): globalThis.Promise<T> {
  const id = args.id as string;
  switch (name) {
    case "app_info":
      return { version: "preview", schema_version: 3 } as T;
    case "jot_down":
      return jotDown(String(args.text)) as T;
    case "read_jot":
      return plainReading(String(args.text)) as T;
    case "list_promises":
      return promises.filter((p) => p.state === (args.state as State)) as T;
    case "count_promises":
      return promises.filter((p) => p.state === (args.state as State)).length as T;
    case "sort_queue":
      return promises
        .filter((p) => p.state === "unsorted")
        .map((promise): SortItem => ({ promise, typed: promise.title, unsure_words: null })) as T;
    case "sort_promise":
      return change(id, (p) => applySorting(p, args.sorting as Sorting, args.to as State)) as T;
    case "mark_done":
      return change(id, (p) => ({ ...p, state: "done" })) as T;
    case "reopen":
      return change(id, (p) => ({ ...p, state: "open" })) as T;
    case "let_go":
      return change(id, (p) => ({ ...p, state: "let_go" })) as T;
    case "move_promise":
      return change(id, (p) => applyWhen({ ...p, state: "open" }, args.when as When, Boolean(args.isDeadline))) as T;
    case "undo":
      return undo(String(args.eventId)) as T;
    case "undo_last": {
      const last = [...changes.entries()].reverse().find(([, c]) => !c.undone);
      return (last ? undo(last[0]) : null) as T;
    }
    case "history":
      return [] as T;
    default:
      throw { message: `the preview does not know "${name}"` };
  }
}

function change(id: string, edit: (p: BindlePromise) => BindlePromise): Change {
  const index = promises.findIndex((p) => p.id === id);
  if (index < 0) throw { message: "that promise is not here any more" };
  const eventId = crypto.randomUUID();
  changes.set(eventId, { before: promises[index], undone: false });
  promises[index] = edit(promises[index]);
  previewEvents.emit("promises-changed");
  return { promise: promises[index], event_id: eventId };
}

function undo(eventId: string): Change {
  const entry = changes.get(eventId);
  if (!entry || entry.undone) throw { message: "nothing to undo" };
  entry.undone = true;
  const index = promises.findIndex((p) => p.id === entry.before.id);
  promises[index] = entry.before;
  previewEvents.emit("promises-changed");
  return { promise: entry.before, event_id: crypto.randomUUID() };
}

function applySorting(p: BindlePromise, s: Sorting, to: State): BindlePromise {
  const sorted = { ...p, title: s.title, direction: s.direction, strength: s.strength, person: s.person, first_step: s.first_step, state: to };
  return s.when ? applyWhen(sorted, s.when, s.is_deadline) : { ...sorted, do_on: null, due: null };
}

function applyWhen(p: BindlePromise, when: When, deadline: boolean): BindlePromise {
  return deadline ? { ...p, due: when, do_on: null } : { ...p, do_on: when, due: null };
}

function plainReading(text: string): Reading {
  return {
    title: text.trim(),
    when: null,
    is_deadline: false,
    needs_check: false,
    unsure_words: null,
    amount: null,
    person: null,
    direction: null,
    strength: "firm",
  };
}

function jotDown(raw: string): BindlePromise {
  const title = raw.trim();
  if (title === "") throw { message: "a promise needs a few words" };
  const now = new Date().toISOString();
  const promise: BindlePromise = {
    id: crypto.randomUUID(),
    title,
    state: "unsorted",
    direction: null,
    strength: "firm",
    mode: "life",
    first_step: null,
    do_on: null,
    due: null,
    amount: null,
    person: null,
    created_at: now,
    updated_at: now,
  };
  promises.push(promise);
  previewEvents.emit("promises-changed");
  return promise;
}
