// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A tiny stand-in for the Rust commands, used only when the views run in a
// plain browser. Nothing here is saved. It mirrors the rules in bindle-store
// closely enough to try the screens, and no further.

import type { Promise as BindlePromise, State } from "./commands";

const promises: BindlePromise[] = [];

export async function preview<T>(name: string, args: Record<string, unknown> = {}): globalThis.Promise<T> {
  switch (name) {
    case "app_info":
      return { version: "preview", schema_version: 2 } as T;
    case "jot_down":
      return jotDown(String(args.title)) as T;
    case "list_promises":
      return promises.filter((p) => p.state === (args.state as State)) as T;
    case "count_promises":
      return promises.filter((p) => p.state === (args.state as State)).length as T;
    default:
      throw { message: `the preview does not know "${name}"` };
  }
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
    created_at: now,
    updated_at: now,
  };
  promises.push(promise);
  return promise;
}
