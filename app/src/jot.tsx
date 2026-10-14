// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The small Jot down window that Alt+Shift+J opens over any app.
// Enter saves, shows "Saved", then the window steps away. Escape or clicking
// elsewhere puts it away without saving.

import { StrictMode, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource/inter/400.css";
import "@fontsource/inter/500.css";
import "@fontsource/inter/600.css";
import "@fontsource/jetbrains-mono/400.css";
import "./tokens.css";
import "./app.css";
import logo from "./art/bindle-24.png";
import { JotField } from "./components/JotField";
import { commands, onJotOpened } from "./lib/commands";

/** How long "Saved" stays on screen before the window goes. */
const SAVED_PAUSE_MS = 700;

function JotWindow() {
  const input = useRef<HTMLInputElement>(null);
  const [opened, setOpened] = useState(0);

  useEffect(
    () =>
      onJotOpened(() => {
        setOpened((n) => n + 1);
        input.current?.focus();
      }),
    [],
  );

  useEffect(() => {
    input.current?.focus();
    // Clicking another app puts the window away, like a menu.
    const away = () => void commands.hideJot();
    window.addEventListener("blur", away);
    return () => window.removeEventListener("blur", away);
  }, []);

  return (
    <div className="jot-window">
      <div className="jot-window-head">
        <img className="sprite" src={logo} alt="" width={24} height={24} />
        <span>Jot down</span>
        <span className="muted small jot-window-tip">Enter to save · Esc to close</span>
      </div>
      <JotField
        ref={input}
        resetKey={opened}
        onEscape={() => void commands.hideJot()}
        onSaved={() => setTimeout(() => void commands.hideJot(), SAVED_PAUSE_MS)}
      />
    </div>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <JotWindow />
  </StrictMode>,
);
