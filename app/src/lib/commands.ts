// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Typed wrappers for the Rust commands in src-tauri/src/commands.rs.
// Keep the two in step: same names, same argument names, same shapes.

import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { preview, previewEvents } from "./preview";

export type State = "unsorted" | "open" | "later" | "done" | "let_go";
export type Direction = "mine" | "theirs" | "self";
export type Strength = "firm" | "soft";
export type Mode = "life" | "work";

/** A local day (`YYYY-MM-DD`) with an optional time (`HH:MM:SS`). */
export interface When {
  day: string;
  time: string | null;
}

/** Money in minor units: 234000 with "INR" is ₹2,340. */
export interface Money {
  minor: number;
  currency: string;
}

/** What Bindle understood from a line, as `bindle_parse::Reading` sends it. */
export interface Reading {
  title: string;
  when: When | null;
  is_deadline: boolean;
  needs_check: boolean;
  unsure_words: string | null;
  amount: Money | null;
  person: string | null;
  direction: Direction | null;
  strength: Strength;
}

/** A promise, as `bindle_core::Promise` sends it. */
export interface Promise {
  id: string;
  title: string;
  state: State;
  direction: Direction | null;
  strength: Strength;
  mode: Mode;
  first_step: string | null;
  do_on: When | null;
  due: When | null;
  amount: Money | null;
  person: string | null;
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
  jotDown: (text: string) => call<Promise>("jot_down", { text }),
  readJot: (text: string) => call<Reading>("read_jot", { text }),
  showJot: () => call<void>("show_jot"),
  hideJot: () => call<void>("hide_jot"),
  listPromises: (state: State) => call<Promise[]>("list_promises", { state }),
  countPromises: (state: State) => call<number>("count_promises", { state }),
};

/** True inside the desktop app, false in a plain browser preview. */
export const inDesktopApp = isTauri;

/**
 * Runs `handler` whenever a promise is saved, in any window.
 * Returns a function that stops listening.
 */
export function onPromiseSaved(handler: () => void): () => void {
  if (!isTauri()) return previewEvents.on("promise-saved", handler);
  const stop = listen("promise-saved", handler);
  return () => void stop.then((unlisten) => unlisten());
}

/** Runs `handler` each time the Jot down window is opened. */
export function onJotOpened(handler: () => void): () => void {
  if (!isTauri()) return () => {};
  const stop = listen("jot-opened", handler);
  return () => void stop.then((unlisten) => unlisten());
}

/** Turns anything a command throws into words for the person. */
export function errorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String((error as CommandError).message);
  }
  return String(error);
}
