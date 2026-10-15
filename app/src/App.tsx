// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { useCallback, useEffect, useRef, useState } from "react";
import { SampleBar } from "./components/SampleBar";
import { Sidebar } from "./components/Sidebar";
import { Tour } from "./components/Tour";
import { UndoProvider } from "./components/Undo";
import { UpdateBar } from "./components/UpdateBar";
import { commands, inDesktopApp, onCreatorChanged, onMailChanged, onPromisesChanged, onWorkChanged, onPhone, type Mode, type SampleStatus, type Space } from "./lib/commands";
import { findScreen, screens, type ScreenId } from "./screens";
import { Board } from "./screens/Board";
import { Code } from "./screens/Code";
import { Desk } from "./screens/Desk";
import { Home } from "./screens/Home";
import { Inbox } from "./screens/Inbox";
import { Creator } from "./screens/Creator";
import { Ours } from "./screens/Ours";
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
  const [repliesOwed, setRepliesOwed] = useState(0);
  const [commentsWaiting, setCommentsWaiting] = useState(0);
  const [spaces, setSpaces] = useState<Space[]>([]);
  const [spaceId, setSpaceId] = useState<string | null>(null);
  const [focusSearch, setFocusSearch] = useState(0);
  const jotRef = useRef<HTMLInputElement>(null);
  const [sample, setSample] = useState<SampleStatus | null>(null);
  const [touring, setTouring] = useState(false);

  // The sample Bindle shows the first time; its tour starts once by itself.
  useEffect(() => {
    commands.sampleStatus().then((status) => {
      setSample(status);
      if (status.showing && !seenTour()) setTouring(true);
    }, () => setSample(null));
  }, []);

  const endTour = useCallback(() => {
    setTouring(false);
    rememberTour();
  }, []);

  const goTo = useCallback((next: Mode, screen?: ScreenId) => {
    setMode(next);
    setCurrent((now) => screen ?? (screens[next].some((s) => s.id === now) ? now : screens[next][0].id));
  }, []);

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

  useEffect(() => {
    const count = () => commands.commentsWaiting().then(setCommentsWaiting, () => setCommentsWaiting(0));
    count();
    const stopCreator = onCreatorChanged(count);
    const stopPromises = onPromisesChanged(count);
    return () => {
      stopCreator();
      stopPromises();
    };
  }, []);

  useEffect(() => {
    const count = () => commands.repliesOwed().then(setRepliesOwed, () => setRepliesOwed(0));
    count();
    return onMailChanged(count);
  }, []);

  useEffect(() => {
    const load = () => commands.cloudStatus().then((s) => setSpaces(s.spaces), () => setSpaces([]));
    load();
    return onPromisesChanged(load);
  }, []);

  const openSpace = useCallback((id: string) => {
    setSpaceId(id);
    setCurrent("ours");
    commands.cloudStatus().then((s) => setSpaces(s.spaces), () => {});
  }, []);

  const openSort = useCallback(() => {
    setMode("life");
    setCurrent("sort");
  }, []);

  const jotDown = useCallback(() => {
    if (inDesktopApp() && !onPhone()) {
      void commands.showJot();
      return;
    }
    // In a browser preview or on a phone there is no second window: use the field on Home.
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
            repliesOwed={repliesOwed}
            commentsWaiting={commentsWaiting}
            spaces={spaces}
            currentSpace={spaceId}
            onSpace={openSpace}
            onMode={switchMode}
            onScreen={setCurrent}
            onJotDown={jotDown}
          />
          <main className="main" id="main" tabIndex={-1}>
            {sample?.showing && <SampleBar status={sample} onTour={() => setTouring(true)} />}
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
            ) : current === "desk" ? (
              <Desk onScreen={setCurrent} onOpenSort={openSort} />
            ) : current === "code" ? (
              <Code onOpenSort={openSort} />
            ) : current === "inbox" ? (
              <Inbox onOpenSort={openSort} />
            ) : current === "board" ? (
              <Board />
            ) : current === "creator" ? (
              <Creator onOpenSort={openSort} />
            ) : current === "settings" ? (
              <Settings onOpenSpace={openSpace} onTour={() => setTouring(true)} />
            ) : current === "ours" && spaceId ? (
              <Ours
                spaceId={spaceId}
                onLeft={() => {
                  setSpaceId(null);
                  setCurrent(screens[mode][0].id);
                  commands.cloudStatus().then((s) => setSpaces(s.spaces), () => {});
                }}
              />
            ) : (
              <Soon screen={findScreen(current)} />
            )}
          </main>
          {touring && <Tour onGo={goTo} onEnd={endTour} />}
        </div>
      </UndoProvider>
    </UpdatesProvider>
  );
}

const TOUR_SEEN = "bindle.tour.seen";

function seenTour(): boolean {
  try {
    return localStorage.getItem(TOUR_SEEN) === "1";
  } catch {
    return false;
  }
}

function rememberTour() {
  try {
    localStorage.setItem(TOUR_SEEN, "1");
  } catch {
    // Without storage the tour may show again next time; that is all.
  }
}
