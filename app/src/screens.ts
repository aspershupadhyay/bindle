// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Every screen in the two modes (handbook P6 and P14), in sidebar order.

import type { IconName } from "./components/Icon";
import type { Mode } from "./lib/commands";

export type ScreenId =
  | "home" | "sort" | "promises" | "people" | "wrapUp"
  | "desk" | "inbox" | "code" | "board" | "creator"
  | "settings" | "ours";

export interface ScreenInfo {
  id: ScreenId;
  label: string;
  icon: IconName;
  /** When the roadmap builds this screen, e.g. "week 4". Shown until it exists. */
  week: string;
  /** The pixel scene for the screen's header, if it has one. */
  scene?: string;
}

export const screens: Record<Mode, ScreenInfo[]> = {
  life: [
    { id: "home", label: "Home", icon: "home", week: "week 4" },
    { id: "sort", label: "Sort", icon: "sort", week: "week 3" },
    { id: "promises", label: "Promises", icon: "promises", week: "week 5" },
    { id: "people", label: "People", icon: "people", week: "week 5" },
    { id: "wrapUp", label: "Wrap up", icon: "wrapUp", week: "week 6" },
  ],
  work: [
    { id: "desk", label: "Desk", icon: "desk", week: "weeks 13 to 14", scene: "station" },
    { id: "inbox", label: "Inbox", icon: "inbox", week: "weeks 11 to 12", scene: "dakghar" },
    { id: "code", label: "Code", icon: "code", week: "weeks 9 to 10", scene: "arcade" },
    { id: "board", label: "Board", icon: "board", week: "weeks 11 to 12", scene: "workshop" },
    { id: "creator", label: "Creator", icon: "creator", week: "weeks 18 to 20", scene: "studio" },
  ],
};

export const settingsScreen: ScreenInfo = { id: "settings", label: "Settings", icon: "settings", week: "week 6" };

export function findScreen(id: ScreenId): ScreenInfo {
  return [...screens.life, ...screens.work, settingsScreen].find((s) => s.id === id) ?? settingsScreen;
}
