// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// New versions. Bindle looks for one shortly after it opens and every few
// hours after that. When a release is newer, a bar offers to install it;
// installing downloads it, checks its signature and restarts Bindle. Nothing
// is installed without the person saying so.

import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { errorMessage, inDesktopApp, onPhone } from "./commands";

/** Where people can always download Bindle by hand. */
export const DOWNLOAD_PAGE = "https://aspershupadhyay.github.io/bindle/";

/** The first look waits until Bindle has settled. */
const FIRST_LOOK_MS = 10_000;
/** Then it looks again every six hours. */
const LOOK_EVERY_MS = 6 * 60 * 60 * 1000;

export type UpdateState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "none" }
  | { kind: "available"; version: string; notes: string | null }
  | { kind: "installing"; version: string; done: number; total: number | null }
  | { kind: "restarting"; version: string }
  /** `version` is set when installing failed, and unset when looking failed. */
  | { kind: "error"; message: string; version?: string };

/** The words for an update state, as Settings shows them. */
export function updateNote(state: UpdateState): string {
  switch (state.kind) {
    case "idle":
      return "";
    case "checking":
      return "Looking for a new version…";
    case "none":
      return "You have the newest version.";
    case "available":
      return `Bindle ${state.version} is ready to install.`;
    case "installing":
      return `Getting Bindle ${state.version}… ${percent(state.done, state.total)}`.trim();
    case "restarting":
      return `Bindle ${state.version} is installed. Restarting…`;
    case "error":
      return `Could not update: ${state.message}. You can always download Bindle from ${DOWNLOAD_PAGE}`;
  }
}

/** "42%" when the size is known, otherwise nothing. */
export function percent(done: number, total: number | null): string {
  if (!total || total <= 0) return "";
  return `${Math.min(100, Math.floor((done / total) * 100))}%`;
}

interface Updates {
  state: UpdateState;
  /** True once the person chose Later for the version on offer. */
  dismissed: boolean;
  check: (quiet?: boolean) => Promise<void>;
  install: () => Promise<void>;
  later: () => void;
}

const UpdatesContext = createContext<Updates>({
  state: { kind: "idle" },
  dismissed: false,
  check: async () => {},
  install: async () => {},
  later: () => {},
});

export function useUpdates() {
  return useContext(UpdatesContext);
}

type Found = Awaited<ReturnType<typeof import("@tauri-apps/plugin-updater").check>>;

export function UpdatesProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<UpdateState>({ kind: "idle" });
  const [dismissed, setDismissed] = useState(false);
  const found = useRef<Found>(null);
  const busy = useRef(false);

  // `quiet` checks (the automatic ones) never show "newest" or errors: no
  // network, or no release yet, is not worth a word.
  const check = useCallback(async (quiet = false) => {
    if (!inDesktopApp() || onPhone() || busy.current) return;
    busy.current = true;
    if (!quiet) setState({ kind: "checking" });
    try {
      const { check: lookForUpdate } = await import("@tauri-apps/plugin-updater");
      const update = await lookForUpdate();
      if (update) {
        if (found.current?.version !== update.version) setDismissed(false);
        found.current = update;
        setState({ kind: "available", version: update.version, notes: update.body ?? null });
      } else if (!quiet) {
        setState({ kind: "none" });
      }
    } catch (error) {
      if (!quiet) setState({ kind: "error", message: errorMessage(error) });
    } finally {
      busy.current = false;
    }
  }, []);

  const install = useCallback(async () => {
    const update = found.current;
    if (!update || busy.current) return;
    busy.current = true;
    const version = update.version;
    setDismissed(false);
    let done = 0;
    let total: number | null = null;
    setState({ kind: "installing", version, done, total });
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? null;
        if (event.event === "Progress") done += event.data.chunkLength;
        setState({ kind: "installing", version, done, total });
      });
      setState({ kind: "restarting", version });
      // On Windows the installer has already closed Bindle and will open it again.
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (error) {
      setState({ kind: "error", message: errorMessage(error), version });
    } finally {
      busy.current = false;
    }
  }, []);

  const later = useCallback(() => setDismissed(true), []);

  useEffect(() => {
    if (!inDesktopApp() || onPhone()) return;
    const first = setTimeout(() => void check(true), FIRST_LOOK_MS);
    const every = setInterval(() => void check(true), LOOK_EVERY_MS);
    return () => {
      clearTimeout(first);
      clearInterval(every);
    };
  }, [check]);

  const value = useMemo(() => ({ state, dismissed, check, install, later }), [state, dismissed, check, install, later]);
  return <UpdatesContext.Provider value={value}>{children}</UpdatesContext.Provider>;
}
