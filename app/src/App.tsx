// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { useCallback, useEffect, useRef, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { commands, inDesktopApp, onPromiseSaved, type Mode, type Promise as BindlePromise } from "./lib/commands";
import { findScreen, screens, type ScreenId } from "./screens";
import { Home } from "./screens/Home";
import { Settings } from "./screens/Settings";
import { Soon } from "./screens/Soon";

export function App() {
  const [mode, setMode] = useState<Mode>("life");
  const [current, setCurrent] = useState<ScreenId>("home");
  const [unsorted, setUnsorted] = useState<BindlePromise[]>([]);
  const jotRef = useRef<HTMLInputElement>(null);

  const refresh = useCallback(() => {
    commands.listPromises("unsorted").then(setUnsorted, () => setUnsorted([]));
  }, []);

  useEffect(() => {
    refresh();
    // Saves from the Jot down window, or from Home, refresh every list.
    return onPromiseSaved(refresh);
  }, [refresh]);

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
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [jotDown]);

  function switchMode(next: Mode) {
    setMode(next);
    setCurrent(screens[next][0].id);
  }

  return (
    <div className="shell">
      <Sidebar
        mode={mode}
        current={current}
        unsorted={unsorted.length}
        onMode={switchMode}
        onScreen={setCurrent}
        onJotDown={jotDown}
      />
      <main className="main">
        {current === "home" ? (
          <Home ref={jotRef} unsorted={unsorted} />
        ) : current === "settings" ? (
          <Settings />
        ) : (
          <Soon screen={findScreen(current)} />
        )}
      </main>
    </div>
  );
}
