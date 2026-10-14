// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The person's settings, shared by every screen, with the theme applied to
// the page. Changes save at once and reach every window.

import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from "react";
import { commands, onSettingsChanged, type Settings } from "./commands";

interface SettingsValue {
  settings: Settings;
  /** Saves a change and returns once it is stored. */
  update: (patch: Partial<Settings>) => globalThis.Promise<void>;
}

const SettingsContext = createContext<SettingsValue | null>(null);

export function useSettings(): SettingsValue {
  const value = useContext(SettingsContext);
  if (!value) throw new Error("useSettings needs a SettingsProvider");
  return value;
}

/** Sets `data-theme` on the page; "system" follows the system. */
export function applyTheme(theme: Settings["theme"]) {
  if (theme === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = theme;
}

/** Loads settings, then shows `children`; nothing shows before they arrive. */
export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings | null>(null);

  useEffect(() => {
    commands.getSettings().then(setSettings);
    return onSettingsChanged(setSettings);
  }, []);

  useEffect(() => {
    if (settings) applyTheme(settings.theme);
  }, [settings]);

  const update = useCallback(
    async (patch: Partial<Settings>) => {
      if (!settings) return;
      setSettings(await commands.saveSettings({ ...settings, ...patch }));
    },
    [settings],
  );

  if (!settings) return null;
  return <SettingsContext.Provider value={{ settings, update }}>{children}</SettingsContext.Provider>;
}
