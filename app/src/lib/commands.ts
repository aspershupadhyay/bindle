// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Typed wrappers for the Rust commands in src-tauri/src/commands.rs.
// Keep the two in step: same names, same argument names, same shapes.

import { invoke, isTauri } from "@tauri-apps/api/core";
import { preview } from "./preview";

export type State = "unsorted" | "open" | "later" | "done" | "let_go";
export type Direction = "mine" | "theirs" | "self";
export type Strength = "firm" | "soft";
export type Mode = "life" | "work";

/** A promise, as `bindle_core::Promise` sends it. Days are `YYYY-MM-DD`. */
export interface Promise {
  id: string;
  title: string;
  state: State;
  direction: Direction | null;
  strength: Strength;
  mode: Mode;
  first_step: string | null;
  do_on: string | null;
  due: string | null;
  created_at: string;
  updated_at: string;
}

export interface AppInfo {
  version: string;
  schema_version: number;
}

/** What a failed command sends back. The message is safe to show. */
export interface CommandError {
  message: string;
}

/**
 * Calls a Rust command. Outside the desktop app (a plain browser running
 * `npm run dev`) the in-memory preview answers instead, so the views can be
 * worked on and screenshotted without building Rust.
 */
function call<T>(name: string, args?: Record<string, unknown>): globalThis.Promise<T> {
  return isTauri() ? invoke<T>(name, args) : preview<T>(name, args);
}

export const commands = {
  appInfo: () => call<AppInfo>("app_info"),
  jotDown: (title: string) => call<Promise>("jot_down", { title }),
  listPromises: (state: State) => call<Promise[]>("list_promises", { state }),
  countPromises: (state: State) => call<number>("count_promises", { state }),
};

/** Turns anything a command throws into words for the person. */
export function errorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String((error as CommandError).message);
  }
  return String(error);
}
