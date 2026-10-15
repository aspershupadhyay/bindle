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
  /** When it was done or let go. */
  closed_at: string | null;
}

/** A promise waiting in Sort, with what was typed and words to check. */
export interface SortItem {
  promise: Promise;
  typed: string | null;
  unsure_words: string | null;
  /** The AI that suggested it ("Claude"), when it was not typed. */
  suggested_by: string | null;
  /** Why the AI suggested it, in its words. */
  reason: string | null;
}

/** What the person chose in Sort. */
export interface Sorting {
  title: string;
  direction: Direction | null;
  strength: Strength;
  person: string | null;
  first_step: string | null;
  when: When | null;
  is_deadline: boolean;
}

/** A saved change, with the event that can undo it. */
export interface Change {
  promise: Promise;
  event_id: string;
}

/** One line of a promise's history. */
export interface HistoryEntry {
  id: string;
  at: string;
  kind: string;
  actor: string;
  undone: boolean;
}

/** Everything Life Home shows, as `bindle_store::home::HomeView` sends it. */
export interface HomeView {
  top3: Promise[];
  suggestions: Promise[];
  running_late: Promise[];
  waiting_on_others: { promise: Promise; last_nudged: string | null }[];
  coming_up: Promise[];
  week: DayLoad[];
  day_capacity: number;
  this_week: { kept: number; moved: number; let_go: number };
  today_count: number;
  people_waiting_on_you: number;
  wrapped_up: boolean;
}

export interface DayLoad {
  day: string;
  count: number;
  full: boolean;
}

export interface StateCounts {
  unsorted: number;
  open: number;
  later: number;
  done: number;
  let_go: number;
}

/** The pictures a person can have. Each names what is drawn, not who someone is. */
export const AVATARS = ["short", "long", "curly", "bun", "beard", "turban", "scarf", "cap", "grey"] as const;
export type Avatar = (typeof AVATARS)[number];

export interface PersonSummary {
  id: string;
  name: string;
  avatar: Avatar | null;
  you_owe: number;
  they_owe: number;
}

export interface PersonDetail {
  id: string;
  name: string;
  avatar: Avatar | null;
  pronouns: string | null;
  hidden_from_ai: boolean;
  you_owe: Promise[];
  they_owe: Promise[];
  kept_lately: Promise[];
}

/** The person's settings, as `bindle_store::settings::Settings` sends them. Times are `HH:MM:SS`. */
export interface Settings {
  name: string | null;
  onboarded: boolean;
  wrap_up_time: string;
  wrap_up_reminder: boolean;
  morning_time: string;
  morning_note: boolean;
  timed_reminders: boolean;
  theme: "system" | "light" | "dark";
  /** How far a connected AI may go. */
  ai_access: AiAccess;
  /** The person's own picture. */
  avatar: Avatar | null;
  /** Life or Work, as last chosen. Jot down saves promises in this mode. */
  mode: Mode;
  /** How many things the Board's Making lane holds before it says so. */
  making_limit: number;
}

export type MailKind = "reply" | "waiting" | "promise";

/** A promise Bindle noticed in an email you wrote. */
export interface NoticedPromise {
  title: string;
  when: When | null;
  is_deadline: boolean;
}

/** An email that needs you, as `bindle_store::mail::MailItem` sends it. */
export interface MailItem {
  id: string;
  kind: MailKind;
  subject: string;
  person: string;
  address: string;
  at: string;
  snippet: string;
  url: string;
  noticed: NoticedPromise | null;
  gone: boolean;
  promise: { id: string; state: State } | null;
}

/** The mailbox connection. The app password is never sent to the views. */
export interface MailAccount {
  address: string | null;
  synced_at: string | null;
  error: string | null;
  /** "gmail", "icloud", … or "other"; missing means Gmail. */
  provider?: string | null;
  host?: string | null;
  port?: number | null;
}

/** A YouTube channel's public numbers. */
export interface Channel {
  id: string;
  title: string;
  handle: string | null;
  description: string;
  /** null when the channel hides its subscriber count. */
  followers: number | null;
  views: number;
  posts: number;
}

/** The YouTube connection. The API key is never sent to the views. */
export interface CreatorAccount {
  channel: string | null;
  found: Channel | null;
  synced_at: string | null;
  error: string | null;
}

export interface CreatorPost {
  id: string;
  title: string;
  url: string;
  short: boolean;
  duration_seconds: number;
  published_at: string;
  views: number;
  likes: number;
  comments: number;
}

export interface CreatorComment {
  id: string;
  post_title: string;
  author: string;
  text: string;
  url: string;
  likes: number;
  published_at: string;
  promise: { id: string; state: State } | null;
}

/** Everything the Creator screen shows. */
export interface CreatorView {
  account: CreatorAccount;
  followers_this_week: number | null;
  views_this_week: number | null;
  growth: { week: string; followers: number }[];
  top_posts: CreatorPost[];
  views_last_30_days: number;
  long_posts: number;
  short_posts: number;
  comments: CreatorComment[];
}

/** A mail service Bindle knows how to read. */
export interface MailProvider {
  id: string;
  name: string;
  host: string;
  port: number;
  password_page: string;
  webmail: string;
  note: string;
}

export interface InboxView {
  account: MailAccount;
  reply: MailItem[];
  waiting: MailItem[];
  noticed: MailItem[];
  finished: MailItem[];
  hidden: number;
}

/** A promise on the Board, with where it came from ("GitHub owner/repo#42", "Gmail"). */
export interface BoardCard {
  promise: Promise;
  from: string | null;
}

/** How long people wait on you, in hours. */
export interface WaitTrend {
  /** Median over the last 7 days. */
  recent: number | null;
  /** Median over the four weeks before. */
  before: number | null;
  /** Median of each of the last 8 weeks, oldest first. */
  weekly: (number | null)[];
}

/** Something that needs you now, on the Desk. */
export interface DeskItem {
  id: string;
  source: "github" | "gmail";
  kind: "review" | "issue" | "reply";
  title: string;
  person: string;
  reference: string;
  at: string;
  url: string;
  promise: { id: string; state: State } | null;
}

export interface DeskView {
  needs_you: DeskItem[];
  email_reply: WaitTrend;
  review_wait: WaitTrend;
  shipped: { this_week: number; weekly: number[]; from_code: number; from_inbox: number; other: number };
  week: { day: string; open: number; done: number }[];
  github: GithubAccount;
  gmail: MailAccount;
  reviews_older_than_a_day: number;
  replies_owed: number;
}

/** Bindle Cloud on this device. Keys never reach the views. */
export interface CloudAccount {
  relay: string | null;
  name: string | null;
  account: string | null;
  device: string | null;
  kit_saved_at: string | null;
}

/** A shared space on this device. */
export interface Space {
  id: string;
  name: string;
  relay: string | null;
  promises: number;
  synced_at: string | null;
  /** Changes waiting to go up. */
  waiting: number;
}

export interface CloudStatus {
  account: CloudAccount;
  spaces: Space[];
}

export interface Member {
  account: string;
  name: string;
  joined_at: string;
}

/** A promise in Ours. `added_by` and `done_by` are null when it was you. */
export interface SharedPromise {
  promise: Promise;
  added_by: string | null;
  done_by: string | null;
}

export interface OursView {
  space: Space;
  members: Member[];
  me: string | null;
  open: SharedPromise[];
  kept_lately: SharedPromise[];
  kept_this_month: number;
}

export interface BoardView {
  ideas: BoardCard[];
  next: BoardCard[];
  making: BoardCard[];
  waiting: BoardCard[];
  shipped: BoardCard[];
}

export type WorkKind = "review" | "mine" | "issue";

/** Something GitHub says needs you, as `bindle_store::work::WorkItem` sends it. */
export interface WorkItem {
  id: string;
  kind: WorkKind;
  /** "owner/name". */
  repo: string;
  number: number;
  title: string;
  author: string;
  url: string;
  draft: boolean;
  labels: string[];
  created_at: string;
  updated_at: string;
  /** GitHub no longer lists it: reviewed, merged, closed or reassigned. */
  gone: boolean;
  /** The promise made from it, if there is one. */
  promise: { id: string; state: State } | null;
}

/** The GitHub connection. The token is never sent to the views. */
export interface GithubAccount {
  login: string | null;
  synced_at: string | null;
  error: string | null;
}

/** Everything the Code screen shows. */
export interface CodeView {
  account: GithubAccount;
  reviews: WorkItem[];
  mine: WorkItem[];
  issues: WorkItem[];
  finished: WorkItem[];
  reviews_older_than_a_day: number;
}

export type AiAccess = "off" | "read" | "suggest";

/** One thing a connected AI read or suggested. */
export interface AiLogEntry {
  at: string;
  client: string;
  tool: string;
  detail: string;
}

/** How to connect an AI app to Bindle. */
export interface AiConnection {
  command: string;
  args: string[];
  config: string;
  log: AiLogEntry[];
}

/** One safety copy of the database. */
export interface SafetyCopy {
  file_name: string;
  made_at: string;
  bytes: number;
}

export interface WrapUpView {
  day: string;
  kept_today: Promise[];
  still_open: Promise[];
  tomorrow: Promise[];
  tomorrow_top3: string[];
  wrapped_up_at: string | null;
  mind_load: number | null;
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

/** A promise found in a pasted chat. */
export interface Caught {
  speaker: string | null;
  said_at: string | null;
  said: string;
  reading: Reading;
}

/** Everything found in one pasted chat. */
export interface Catch {
  speakers: string[];
  me: string | null;
  found: Caught[];
}

/** Whether the sample Bindle is showing, and whether the person has their own. */
export interface SampleStatus {
  showing: boolean;
  has_own: boolean;
}

export const commands = {
  appInfo: () => call<AppInfo>("app_info"),
  sampleStatus: () => call<SampleStatus>("sample_status"),
  sampleLeave: () => call<void>("sample_leave"),
  sampleOpen: () => call<void>("sample_open"),
  jotDown: (text: string) => call<Promise>("jot_down", { text }),
  readJot: (text: string) => call<Reading>("read_jot", { text }),
  catchChat: (text: string, me: string | null) => call<Catch>("catch_chat", { text, me }),
  catchKeep: (chosen: { said: string; reading: Reading }[]) => call<Promise[]>("catch_keep", { chosen }),
  showJot: () => call<void>("show_jot"),
  sortQueue: () => call<SortItem[]>("sort_queue"),
  sortPromise: (id: string, sorting: Sorting, to: "open" | "later" | "let_go") =>
    call<Change>("sort_promise", { id, sorting, to }),
  markDone: (id: string) => call<Change>("mark_done", { id }),
  reopen: (id: string) => call<Change>("reopen", { id }),
  letGo: (id: string) => call<Change>("let_go", { id }),
  movePromise: (id: string, when: When, isDeadline: boolean) =>
    call<Change>("move_promise", { id, when, isDeadline }),
  undo: (eventId: string) => call<Change>("undo", { eventId }),
  undoLast: () => call<Change | null>("undo_last"),
  history: (id: string) => call<HistoryEntry[]>("history", { id }),
  homeView: () => call<HomeView>("home_view"),
  addToTop3: (id: string, day?: string) => call<void>("add_to_top3", { id, day }),
  removeFromTop3: (id: string, day?: string) => call<void>("remove_from_top3", { id, day }),
  nudge: (id: string) => call<void>("nudge", { id }),
  searchPromises: (query: string) => call<Promise[]>("search_promises", { query }),
  stateCounts: () => call<StateCounts>("state_counts"),
  promisesIn: (state: State) => call<Promise[]>("promises_in", { state }),
  peopleList: () => call<PersonSummary[]>("people_list"),
  personDetail: (id: string) => call<PersonDetail>("person_detail", { id }),
  renamePerson: (id: string, name: string) => call<void>("rename_person", { id, name }),
  archivePerson: (id: string) => call<void>("archive_person", { id }),
  unarchivePerson: (id: string) => call<void>("unarchive_person", { id }),
  archivedPeople: () => call<PersonSummary[]>("archived_people"),
  getSettings: () => call<Settings>("get_settings"),
  saveSettings: (settings: Settings) => call<Settings>("save_settings", { settings }),
  testNotification: () => call<void>("test_notification"),
  wrapUpView: () => call<WrapUpView>("wrap_up_view"),
  finishWrapUp: (mindLoad: number | null, note: string | null) => call<void>("finish_wrap_up", { mindLoad, note }),
  hideJot: () => call<void>("hide_jot"),
  aiConnection: () => call<AiConnection>("ai_connection"),
  exportData: (format: "json" | "csv") => call<string>("export_data", { format }),
  safetyCopies: () => call<SafetyCopy[]>("safety_copies"),
  makeSafetyCopy: () => call<SafetyCopy>("make_safety_copy"),
  restoreSafetyCopy: (fileName: string) => call<SafetyCopy>("restore_safety_copy", { fileName }),
  setPersonLook: (id: string, avatar: Avatar | null, pronouns: string | null) =>
    call<void>("set_person_look", { id, avatar, pronouns }),
  setPersonHidden: (id: string, hidden: boolean) => call<void>("set_person_hidden", { id, hidden }),
  codeView: () => call<CodeView>("code_view"),
  reviewsOwed: () => call<number>("reviews_owed"),
  githubTokenPage: () => call<string>("github_token_page"),
  githubConnect: (token: string) => call<GithubAccount>("github_connect", { token }),
  githubDisconnect: () => call<void>("github_disconnect"),
  githubRefresh: () => call<GithubAccount>("github_refresh"),
  makePromise: (itemId: string) => call<Promise>("make_promise", { itemId }),
  openOnGithub: (url: string) => call<void>("open_on_github", { url }),
  inboxView: () => call<InboxView>("inbox_view"),
  repliesOwed: () => call<number>("replies_owed"),
  creatorView: () => call<CreatorView>("creator_view"),
  commentsWaiting: () => call<number>("comments_waiting"),
  youtubeKeyPage: () => call<string>("youtube_key_page"),
  youtubeConnect: (channel: string, key: string) => call<CreatorAccount>("youtube_connect", { channel, key }),
  youtubeDisconnect: () => call<void>("youtube_disconnect"),
  youtubeRefresh: () => call<CreatorAccount>("youtube_refresh"),
  commentMakePromise: (commentId: string) => call<Promise>("comment_make_promise", { commentId }),
  commentHide: (commentId: string) => call<void>("comment_hide", { commentId }),
  commentUnhide: (commentId: string) => call<void>("comment_unhide", { commentId }),
  openOnYoutube: (url: string) => call<void>("open_on_youtube", { url }),
  mailProviders: () => call<MailProvider[]>("mail_providers"),
  gmailConnect: (address: string, password: string, provider: string, host?: string, port?: number) =>
    call<MailAccount>("gmail_connect", { address, password, provider, host: host ?? null, port: port ?? null }),
  gmailDisconnect: () => call<void>("gmail_disconnect"),
  gmailRefresh: () => call<MailAccount>("gmail_refresh"),
  mailMakePromise: (itemId: string) => call<Promise>("mail_make_promise", { itemId }),
  mailDismiss: (itemId: string) => call<void>("mail_dismiss", { itemId }),
  mailRestore: (itemId: string) => call<void>("mail_restore", { itemId }),
  openInGmail: (url: string) => call<void>("open_in_gmail", { url }),
  boardView: () => call<BoardView>("board_view"),
  deskView: () => call<DeskView>("desk_view"),
  cloudStatus: () => call<CloudStatus>("cloud_status"),
  cloudStart: (relay: string, name: string) => call<string>("cloud_start", { relay, name }),
  cloudRecoveryKit: () => call<string>("cloud_recovery_kit"),
  cloudKitSaved: () => call<void>("cloud_kit_saved"),
  cloudRestore: (relay: string, kit: string) => call<CloudStatus>("cloud_restore", { relay, kit }),
  cloudSync: () => call<CloudStatus>("cloud_sync"),
  oursCreate: (name: string) => call<Space>("ours_create", { name }),
  oursInvite: (spaceId: string) => call<string>("ours_invite", { spaceId }),
  oursJoin: (link: string) => call<Space>("ours_join", { link }),
  oursShare: (promiseId: string, spaceId: string | null) => call<void>("ours_share", { promiseId, spaceId }),
  oursJot: (spaceId: string, text: string) => call<Promise>("ours_jot", { spaceId, text }),
  oursView: (spaceId: string) => call<OursView>("ours_view", { spaceId }),
  oursLeave: (spaceId: string) => call<void>("ours_leave", { spaceId }),
  setMaking: (id: string, making: boolean) => call<void>("set_making", { id, making }),
  listPromises: (state: State) => call<Promise[]>("list_promises", { state }),
  countPromises: (state: State) => call<number>("count_promises", { state }),
};

/** True inside the desktop app, false in a plain browser preview. */
export const inDesktopApp = isTauri;

/** True in the phone app (Android or iOS): no second window, no global shortcut, updates come from the store. */
export function onPhone(): boolean {
  return typeof navigator !== "undefined" && /Android|iPhone|iPad|iPod/i.test(navigator.userAgent);
}

/**
 * Runs `handler` whenever any promise is saved or changed, in any window.
 * Returns a function that stops listening.
 */
export function onPromisesChanged(handler: () => void): () => void {
  if (!isTauri()) return previewEvents.on("promises-changed", handler);
  const stop = listen("promises-changed", handler);
  return () => void stop.then((unlisten) => unlisten());
}

/** Runs `handler` after each look at GitHub, or when it is connected or disconnected. */
export function onWorkChanged(handler: () => void): () => void {
  if (!isTauri()) return previewEvents.on("work-changed", handler);
  const stop = listen("work-changed", handler);
  return () => void stop.then((unlisten) => unlisten());
}

/** Runs `handler` after each look at YouTube, or when it is connected or disconnected. */
export function onCreatorChanged(handler: () => void): () => void {
  if (!isTauri()) return previewEvents.on("creator-changed", handler);
  const stop = listen("creator-changed", handler);
  return () => void stop.then((unlisten) => unlisten());
}

/** Runs `handler` after each look at Gmail, or when it is connected or disconnected. */
export function onMailChanged(handler: () => void): () => void {
  if (!isTauri()) return previewEvents.on("mail-changed", handler);
  const stop = listen("mail-changed", handler);
  return () => void stop.then((unlisten) => unlisten());
}

/** Runs `handler` with the new settings whenever they change, in any window. */
export function onSettingsChanged(handler: (settings: Settings) => void): () => void {
  if (!isTauri()) return previewEvents.on("settings-changed", () => {});
  const stop = listen<Settings>("settings-changed", (event) => handler(event.payload));
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
