// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A tiny stand-in for the Rust commands, used only when the views run in a
// plain browser. Nothing here is saved, nothing is parsed, and the lifecycle
// rules are not checked: the real ones live in Rust. It exists so screens can
// be tried and screenshotted without building the desktop app.

import { sampleCodeView, sampleWork } from "./preview-work";
import type { Change, GithubAccount, HomeView, Settings, Promise as BindlePromise, Reading, SortItem, Sorting, State, When } from "./commands";

const promises: BindlePromise[] = [];
let settings: Settings = {
  name: null,
  onboarded: false,
  wrap_up_time: "21:00:00",
  wrap_up_reminder: false,
  morning_time: "08:30:00",
  morning_note: false,
  timed_reminders: true,
  theme: "system",
  ai_access: "off",
  avatar: null,
  mode: "life",
};
let github: GithubAccount = { login: null, synced_at: null, error: null };
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
        .map((promise): SortItem => ({ promise, typed: promise.title, unsure_words: null, suggested_by: null, reason: null })) as T;
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
    case "home_view":
      return homeView() as T;
    case "search_promises": {
      const words = String(args.query).toLowerCase().split(/\s+/).filter(Boolean);
      return (words.length === 0 ? [] : promises.filter((p) => words.every((w) => p.title.toLowerCase().includes(w)))) as T;
    }
    case "state_counts": {
      const count = (state: State) => promises.filter((p) => p.state === state).length;
      return { unsorted: count("unsorted"), open: count("open"), later: count("later"), done: count("done"), let_go: count("let_go") } as T;
    }
    case "promises_in":
      return promises.filter((p) => p.state === (args.state as State)) as T;
    case "people_list":
      return [] as T;
    case "person_detail":
    case "rename_person":
      throw { message: "People works in the desktop app only" };
    case "get_settings":
      return settings as T;
    case "save_settings":
      settings = { ...(args.settings as Settings) };
      return settings as T;
    case "test_notification":
      return undefined as T;
    case "wrap_up_view":
      return {
        day: new Date().toISOString().slice(0, 10),
        kept_today: promises.filter((p) => p.state === "done"),
        still_open: promises.filter((p) => p.state === "open"),
        tomorrow: [],
        tomorrow_top3: [],
        wrapped_up_at: null,
        mind_load: null,
      } as T;
    case "finish_wrap_up":
      return undefined as T;
    case "ai_connection":
      return { command: "bindle", args: ["--mcp", "--db", "bindle.db"], config: "{}", log: [] } as T;
    case "safety_copies":
      return [] as T;
    case "export_data":
    case "make_safety_copy":
    case "restore_safety_copy":
    case "set_person_look":
    case "set_person_hidden":
      throw { message: "this works in the desktop app only" };
    case "add_to_top3":
    case "remove_from_top3":
    case "nudge":
      throw { message: "the top 3 and nudges work in the desktop app only" };
    case "code_view":
      return sampleCodeView(github) as T;
    case "reviews_owed":
      return (github.login ? sampleWork.filter((i) => i.kind === "review").length : 0) as T;
    case "github_token_page":
      return "https://github.com/settings/tokens/new?description=Bindle&scopes=repo" as T;
    case "github_connect":
    case "github_refresh":
      if (name === "github_connect" && !String(args.token).trim()) throw { message: "paste the token from GitHub first" };
      github = { login: "aspershupadhyay", synced_at: new Date().toISOString(), error: null };
      previewEvents.emit("work-changed");
      return github as T;
    case "github_disconnect":
      github = { login: null, synced_at: null, error: null };
      previewEvents.emit("work-changed");
      return undefined as T;
    case "make_promise": {
      const item = sampleWork.find((i) => i.id === args.itemId);
      if (!item) throw { message: "that promise is not here any more" };
      const promise = { ...jotDown(`Review ${item.repo.split("/")[1]} #${item.number}: ${item.title}`), mode: "work" as const, person: item.author };
      promises[promises.length - 1] = promise;
      item.promise = { id: promise.id, state: "unsorted" };
      previewEvents.emit("work-changed");
      return promise as T;
    }
    case "open_on_github":
      window.open(String(args.url), "_blank", "noopener");
      return undefined as T;
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
    closed_at: null,
  };
  promises.push(promise);
  previewEvents.emit("promises-changed");
  return promise;
}

/** A rough Home: open promises go to Coming up or Waiting on others. */
function homeView(): HomeView {
  const open = promises.filter((p) => p.state === "open");
  return {
    top3: [],
    suggestions: [],
    running_late: [],
    waiting_on_others: open.filter((p) => p.direction === "theirs").map((promise) => ({ promise, last_nudged: null })),
    coming_up: open.filter((p) => p.direction !== "theirs"),
    week: [],
    day_capacity: 5,
    this_week: { kept: 0, moved: 0, let_go: 0 },
    today_count: 0,
    people_waiting_on_you: 0,
    wrapped_up: false,
  };
}
