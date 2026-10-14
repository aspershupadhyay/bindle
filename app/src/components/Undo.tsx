// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Undo, everywhere (handbook P13). After a change, a quiet bar says what
// happened and offers Undo for a few seconds. Ctrl+Z (Cmd+Z on a Mac) undoes
// the latest change at any time, unless you are typing in a field.

import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { commands, errorMessage, type Change } from "../lib/commands";

/** How long the bar stays, in milliseconds. */
const SHOW_FOR_MS = 8000;

type Bar = { text: string; eventId?: string; tone: "plain" | "error" };

const OfferUndo = createContext<(text: string, change: Change) => void>(() => {});

/** Call with words like "Kept" and the change, to show the bar with Undo. */
export function useOfferUndo() {
  return useContext(OfferUndo);
}

export function UndoProvider({ children }: { children: ReactNode }) {
  const [bar, setBar] = useState<Bar | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);

  const show = useCallback((next: Bar) => {
    clearTimeout(timer.current);
    setBar(next);
    timer.current = setTimeout(() => setBar(null), SHOW_FOR_MS);
  }, []);

  const offer = useCallback((text: string, change: Change) => show({ text, eventId: change.event_id, tone: "plain" }), [show]);

  const run = useCallback(
    async (undo: () => globalThis.Promise<Change | null>) => {
      try {
        const change = await undo();
        show({ text: change ? "Undone" : "Nothing to undo", tone: "plain" });
      } catch (error) {
        show({ text: `Could not undo: ${errorMessage(error)}`, tone: "error" });
      }
    },
    [show],
  );

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      const typing = event.target instanceof HTMLElement && event.target.closest("input, textarea, [contenteditable]");
      if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.key.toLowerCase() === "z" && !typing) {
        event.preventDefault();
        void run(commands.undoLast);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [run]);

  useEffect(() => () => clearTimeout(timer.current), []);

  return (
    <OfferUndo.Provider value={offer}>
      {children}
      <div className="undo-bar-slot" role="status" aria-live="polite">
        {bar && (
          <div className={`undo-bar ${bar.tone}`}>
            <span>{bar.text}</span>
            {bar.eventId && (
              <button type="button" onClick={() => void run(() => commands.undo(bar.eventId!))}>
                Undo
              </button>
            )}
          </div>
        )}
      </div>
    </OfferUndo.Provider>
  );
}
