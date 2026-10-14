// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A tiny stand-in for the Rust commands, used only when the views run in a
// plain browser. Nothing here is saved, and nothing is parsed: the Rust
// parser only runs in the desktop app.

import type { Promise as BindlePromise, Reading, State } from "./commands";

const promises: BindlePromise[] = [];
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
  switch (name) {
    case "app_info":
      return { version: "preview", schema_version: 2 } as T;
    case "jot_down":
      return jotDown(String(args.text)) as T;
    case "read_jot":
      return plainReading(String(args.text)) as T;
    case "list_promises":
      return promises.filter((p) => p.state === (args.state as State)) as T;
    case "count_promises":
      return promises.filter((p) => p.state === (args.state as State)).length as T;
    default:
      throw { message: `the preview does not know "${name}"` };
  }
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
  previewEvents.emit("promise-saved");
  return promise;
}
