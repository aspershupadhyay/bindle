// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// How each Work source is doing, for the sidebar: connected or not, when it
// last looked, and whether it needs reconnecting.

import { useEffect, useState } from "react";
import { commands, onCreatorChanged, onMailChanged, onWorkChanged } from "./commands";
import type { ScreenId } from "../screens";
import { age, mailName } from "./work";

export interface SourceLine {
  name: string;
  screen: ScreenId;
  /** "ok", "error" or "off" (not connected). */
  state: "ok" | "error" | "off";
  /** "2 min ago", "reconnect" or "connect". */
  note: string;
}

/** One line for a source, from its account. */
export function sourceLine(
  name: string,
  screen: ScreenId,
  account: { synced_at: string | null; error: string | null } | null,
  connected: boolean,
  now: Date,
): SourceLine {
  if (!connected || !account) return { name, screen, state: "off", note: "connect" };
  if (account.error) return { name, screen, state: "error", note: "reconnect" };
  const when = account.synced_at ? age(account.synced_at, now) : null;
  return { name, screen, state: "ok", note: when ? (when === "just now" ? "just now" : `${when} ago`) : "looking…" };
}

export function useSources(): SourceLine[] {
  const [lines, setLines] = useState<SourceLine[]>([]);
  useEffect(() => {
    const load = async () => {
      const now = new Date();
      const [code, inbox, creator] = await Promise.all([
        commands.codeView().catch(() => null),
        commands.inboxView().catch(() => null),
        commands.creatorView().catch(() => null),
      ]);
      setLines([
        sourceLine(inbox?.account.address ? mailName(inbox.account) : "Email", "inbox", inbox?.account ?? null, Boolean(inbox?.account.address), now),
        sourceLine("GitHub", "code", code?.account ?? null, Boolean(code?.account.login), now),
        sourceLine("YouTube", "creator", creator?.account ?? null, Boolean(creator?.account.channel), now),
      ]);
    };
    void load();
    const timer = setInterval(() => void load(), 60_000);
    const stops = [onWorkChanged(() => void load()), onMailChanged(() => void load()), onCreatorChanged(() => void load())];
    return () => {
      clearInterval(timer);
      stops.forEach((stop) => stop());
    };
  }, []);
  return lines;
}
