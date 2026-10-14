// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { useCallback, useEffect, useRef, useState } from "react";
import { Sidebar } from "./components/Sidebar";
import { commands, type Mode, type Promise as BindlePromise } from "./lib/commands";
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

  useEffect(refresh, [refresh]);

  const jotDown = useCallback(() => {
    setMode("life");
    setCurrent("home");
    // Wait a frame so Home is on screen before focusing its field.
    requestAnimationFrame(() => jotRef.current?.focus());
  }, []);

  useEffect(() => {
    // In-app shortcut. The system-wide one comes with the Jot down window in week 2.
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
          <Home ref={jotRef} unsorted={unsorted} onSaved={refresh} />
        ) : current === "settings" ? (
          <Settings />
        ) : (
          <Soon screen={findScreen(current)} />
        )}
      </main>
    </div>
  );
}
