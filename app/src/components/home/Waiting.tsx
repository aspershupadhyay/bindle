// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// "Waiting on others": what people owe you, oldest first. Nudge copies a
// friendly message for the person to send themselves. Bindle never sends it
// (handbook principle 6).

import type { Avatar, HomeView, Promise as BindlePromise } from "../../lib/commands";
import { commands, errorMessage } from "../../lib/commands";
import { nudgeMessage, waitingWords, withPerson } from "../../lib/home";
import { usePictures } from "../../lib/pictures";
import { Avatar as AvatarPicture } from "../Avatar";
import { useNotify } from "../Undo";

export function Waiting({ waiting }: { waiting: HomeView["waiting_on_others"] }) {
  const pictures = usePictures();
  if (waiting.length === 0) return null;
  return (
    <section aria-labelledby="waiting">
      <div className="section-head">
        <h2 id="waiting">Waiting on others</h2>
        <span className="muted small">{waiting.length}, oldest first</span>
      </div>
      <ul className="card list">
        {waiting.map((w) => (
          <WaitingRow key={w.promise.id} promise={w.promise} lastNudged={w.last_nudged} picture={pictures.get((w.promise.person ?? "").toLowerCase()) ?? null} />
        ))}
      </ul>
    </section>
  );
}

function WaitingRow({ promise, lastNudged, picture }: { promise: BindlePromise; lastNudged: string | null; picture: Avatar | null }) {
  const notify = useNotify();

  async function nudge() {
    const message = nudgeMessage(promise);
    let copied = false;
    try {
      await navigator.clipboard.writeText(message);
      copied = true;
    } catch {
      // Some systems refuse the clipboard; the nudge is still worth recording.
    }
    try {
      await commands.nudge(promise.id);
      notify(copied ? "Nudge copied. Paste it wherever you talk." : `Nudge noted. Say: “${message}”`);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  return (
    <li className="waiting-row">
      <AvatarPicture name={promise.person ?? promise.title} avatar={picture} size={32} />
      <span className="promise-main">
        <span className="promise-title">{withPerson(promise)}</span>
        <span className="muted small">{waitingWords(promise, lastNudged, new Date())}</span>
      </span>
      <button type="button" className="button-quiet small-button" onClick={() => void nudge()}>
        Nudge
      </button>
    </li>
  );
}
