// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The guided tour: a few short stops over the real screens, each pointing at
// the thing it explains. Esc or "Skip" ends it; Settings starts it again.

import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import type { Mode } from "../lib/commands";
import type { ScreenId } from "../screens";

export interface TourStop {
  /** The `data-tour` value of what to point at. */
  target: string;
  /** Where to be first, if anywhere. */
  mode?: Mode;
  screen?: ScreenId;
  title: string;
  body: string;
}

export const TOUR: TourStop[] = [
  {
    target: "jot",
    mode: "life",
    screen: "home",
    title: "Say it the way you’d say it",
    body: "Press Alt+Shift+J anywhere on your computer and type “send the deck to Priya by Friday” or “Rohan will pay 500 kal”. Bindle reads the day, the person, the money and who owes whom, in English or Hinglish.",
  },
  {
    target: "catch",
    mode: "life",
    screen: "home",
    title: "Promises hide in chats",
    body: "Paste a WhatsApp, Slack or email thread and Bindle finds every promise in it, yours and theirs. Nothing leaves your computer.",
  },
  {
    target: "top3",
    mode: "life",
    screen: "home",
    title: "Three things today",
    body: "Pick up to three. Everything else can wait, and Bindle says so instead of piling it on.",
  },
  {
    target: "waiting",
    mode: "life",
    screen: "home",
    title: "What others owe you",
    body: "Money lent, files promised, replies due. Nudge copies a friendly reminder for you to send; Bindle never messages anyone for you.",
  },
  {
    target: "nav-sort",
    mode: "life",
    title: "Sort, a few seconds each",
    body: "Anything you jot waits here until you say who does it and when. Bindle asks when a word like “kal” could mean two days.",
  },
  {
    target: "nav-people",
    mode: "life",
    title: "Both sides, per person",
    body: "What you owe Priya, what she owes you, what you kept lately, and a recap you can paste into your chat.",
  },
  {
    target: "space-ours",
    mode: "life",
    title: "Ours: a shared bag",
    body: "Share a space with someone you live or work with. Locked on your device before it is sent, and no scores, ever.",
  },
  {
    target: "mode",
    mode: "work",
    screen: "desk",
    title: "Work, in one desk",
    body: "Reviews people wait on, emails that need a reply, comments on your videos, and how long people wait on you. Read only: Bindle never posts or sends.",
  },
  {
    target: "nav-wrapUp",
    mode: "life",
    screen: "home",
    title: "Close the day in a minute",
    body: "Each evening, see what you kept and decide about the rest, so your head can rest.",
  },
  {
    target: "start",
    title: "Now make it yours",
    body: "This was Aarav’s made-up week. Start your own Bindle when you’re ready; you can come back to this sample from Settings.",
  },
];

interface TourProps {
  onGo: (mode: Mode, screen?: ScreenId) => void;
  onEnd: () => void;
}

const GAP = 12;

export function Tour({ onGo, onEnd }: TourProps) {
  const [index, setIndex] = useState(0);
  const [box, setBox] = useState<DOMRect | null>(null);
  const card = useRef<HTMLDivElement>(null);
  const stop = TOUR[index];

  useEffect(() => {
    if (stop.mode) onGo(stop.mode, stop.screen);
  }, [stop, onGo]);

  // Finds the target once it has drawn, and follows it on resize and scroll.
  const measure = useCallback(() => {
    const el = document.querySelector(`[data-tour="${stop.target}"]`);
    if (el) {
      el.scrollIntoView({ block: "nearest" });
      setBox(el.getBoundingClientRect());
    } else {
      setBox(null);
    }
  }, [stop]);

  useLayoutEffect(() => {
    let tries = 0;
    const timer = setInterval(() => {
      measure();
      if (++tries > 10) clearInterval(timer);
    }, 80);
    window.addEventListener("resize", measure);
    window.addEventListener("scroll", measure, true);
    return () => {
      clearInterval(timer);
      window.removeEventListener("resize", measure);
      window.removeEventListener("scroll", measure, true);
    };
  }, [measure]);

  useEffect(() => {
    card.current?.querySelector<HTMLButtonElement>(".tour-next")?.focus();
  }, [index]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") onEnd();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onEnd]);

  const last = index === TOUR.length - 1;
  const place = placement(box);
  return (
    <div className="tour" aria-live="polite">
      {box && (
        <div
          className="tour-spot"
          style={{ top: box.top - 6, left: box.left - 6, width: box.width + 12, height: box.height + 12 }}
          aria-hidden="true"
        />
      )}
      <div ref={card} className="tour-card" role="dialog" aria-labelledby="tour-title" aria-describedby="tour-body" style={place}>
        <div className="tour-step">
          {index + 1} of {TOUR.length}
        </div>
        <h2 id="tour-title" className="tour-title">
          {stop.title}
        </h2>
        <p id="tour-body" className="tour-body">
          {stop.body}
        </p>
        <div className="tour-actions">
          <button type="button" className="link-button small" onClick={onEnd}>
            Skip the tour
          </button>
          <span className="tour-moves">
            {index > 0 && (
              <button type="button" className="button-quiet small-button" onClick={() => setIndex(index - 1)}>
                Back
              </button>
            )}
            <button type="button" className="button-primary small-primary tour-next" onClick={() => (last ? onEnd() : setIndex(index + 1))}>
              {last ? "Done" : "Next"}
            </button>
          </span>
        </div>
      </div>
    </div>
  );
}

/** Beside the target when it fits, else below or above it; centred without one. */
function placement(box: DOMRect | null): React.CSSProperties {
  const width = Math.min(340, window.innerWidth - 32);
  if (!box) return { top: "50%", left: "50%", width, transform: "translate(-50%, -50%)" };
  const room = window.innerWidth - box.right;
  const clampTop = (top: number) => Math.max(16, Math.min(top, window.innerHeight - 260));
  if (room > width + GAP * 2) return { top: clampTop(box.top), left: box.right + GAP * 2, width };
  const left = Math.max(16, Math.min(box.left, window.innerWidth - width - 16));
  if (window.innerHeight - box.bottom > 240) return { top: box.bottom + GAP * 2, left, width };
  return { top: Math.max(16, box.top - 240 - GAP), left, width };
}
