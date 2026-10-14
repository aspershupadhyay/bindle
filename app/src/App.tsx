// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { useCallback, useEffect, useRef, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { UndoProvider } from "./components/Undo";
import { UpdateBar } from "./components/UpdateBar";
import { commands, inDesktopApp, onPromisesChanged, onWorkChanged, type Mode } from "./lib/commands";
import { findScreen, screens, type ScreenId } from "./screens";
import { Code } from "./screens/Code";
import { Home } from "./screens/Home";
import { useSettings } from "./lib/settings";
import { UpdatesProvider } from "./lib/updates";
import { FirstRun } from "./screens/FirstRun";
import { People } from "./screens/People";
import { Promises } from "./screens/Promises";
import { Settings } from "./screens/Settings";
import { Sort } from "./screens/Sort";
import { WrapUp } from "./screens/WrapUp";
import { Soon } from "./screens/Soon";

export function App() {
  const { settings, update } = useSettings();
  // Bindle opens in the mode last chosen, on that mode's first screen.
  const [mode, setMode] = useState<Mode>(settings.mode);
  const [current, setCurrent] = useState<ScreenId>(screens[settings.mode][0].id);
  const [unsorted, setUnsorted] = useState(0);
  const [reviewsOwed, setReviewsOwed] = useState(0);
  const [focusSearch, setFocusSearch] = useState(0);
  const jotRef = useRef<HTMLInputElement>(null);

  const refresh = useCallback(() => {
    commands.countPromises("unsorted").then(setUnsorted, () => setUnsorted(0));
  }, []);

  useEffect(() => {
    refresh();
    // Changes from any window, or any screen, refresh the badge.
    return onPromisesChanged(refresh);
  }, [refresh]);

  useEffect(() => {
    const count = () => commands.reviewsOwed().then(setReviewsOwed, () => setReviewsOwed(0));
    count();
    return onWorkChanged(count);
  }, []);

  const jotDown = useCallback(() => {
    if (inDesktopApp()) {
      void commands.showJot();
      return;
    }
    // In a browser preview there is no second window: use the field on Home.
    setMode("life");
    setCurrent("home");
    requestAnimationFrame(() => jotRef.current?.focus());
  }, []);

  useEffect(() => {
    // Alt+Shift+J is claimed system-wide when the desktop allows it. Where it
    // is not (some Wayland sessions), this still catches it inside Bindle.
    function onKey(event: KeyboardEvent) {
      if (event.altKey && event.shiftKey && event.code === "KeyJ") {
        event.preventDefault();
        jotDown();
      }
      // Ctrl+K (Cmd+K), or "/" outside a text field, opens search.
      const typing = event.target instanceof HTMLElement && event.target.closest("input, textarea, [contenteditable]");
      if (((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") || (event.key === "/" && !typing)) {
        event.preventDefault();
        setMode("life");
        setCurrent("promises");
        setFocusSearch((n) => n + 1);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [jotDown]);

  function switchMode(next: Mode) {
    setMode(next);
    setCurrent(screens[next][0].id);
  }

  // The mode is saved whenever it changes, so Jot down (in its own window)
  // files promises there, and Bindle opens in it next time.
  useEffect(() => {
    if (mode !== settings.mode) void update({ mode }).catch(() => {});
  }, [mode, settings.mode, update]);

  if (!settings.onboarded) {
    return (
      <UndoProvider>
        <FirstRun />
      </UndoProvider>
    );
  }

  return (
    <UpdatesProvider>
      <UndoProvider>
        <div className="shell">
          <a className="skip-link" href="#main">Skip to the page</a>
          <Sidebar
            mode={mode}
            current={current}
            unsorted={unsorted}
            reviewsOwed={reviewsOwed}
            onMode={switchMode}
            onScreen={setCurrent}
            onJotDown={jotDown}
          />
          <main className="main" id="main" tabIndex={-1}>
            <UpdateBar />
            {current === "home" ? (
              <Home
                ref={jotRef}
                unsorted={unsorted}
                onOpenSort={() => setCurrent("sort")}
                onWrapUp={() => setCurrent("wrapUp")}
              />
            ) : current === "sort" ? (
              <Sort />
            ) : current === "promises" ? (
              <Promises focusSearch={focusSearch} onOpenSort={() => setCurrent("sort")} />
            ) : current === "people" ? (
              <People />
            ) : current === "wrapUp" ? (
              <WrapUp onDone={() => setCurrent("home")} />
            ) : current === "code" ? (
              <Code
                onOpenSort={() => {
                  setMode("life");
                  setCurrent("sort");
                }}
              />
            ) : current === "settings" ? (
              <Settings />
            ) : (
              <Soon screen={findScreen(current)} />
            )}
          </main>
        </div>
      </UndoProvider>
    </UpdatesProvider>
  );
}
