// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The band across the top while the sample Bindle shows: whose week it is,
// the tour, and the way to your own Bindle. Your own data is never mixed in.

import { useState } from "react";
import { commands, errorMessage, type SampleStatus } from "../lib/commands";
import { useNotify } from "./Undo";

export function SampleBar({ status, onTour }: { status: SampleStatus; onTour: () => void }) {
  const notify = useNotify();
  const [busy, setBusy] = useState(false);
  async function leave() {
    setBusy(true);
    try {
      await commands.sampleLeave();
      // Everything on screen came from the sample: start clean.
      window.location.reload();
    } catch (error) {
      notify(errorMessage(error));
      setBusy(false);
    }
  }
  return (
    <div className="sample-bar" role="region" aria-label="Sample data">
      <span className="sample-tag">Sample data</span>
      <span className="sample-words">
        This is Aarav’s made-up week, so you can see how Bindle works. Try anything: nothing here is real, and nothing is sent.
      </span>
      <span className="sample-actions">
        <button type="button" className="button-quiet small-button" onClick={onTour}>
          Take the tour
        </button>
        <button type="button" className="button-primary small-primary" data-tour="start" disabled={busy} onClick={() => void leave()}>
          {status.has_own ? "Back to my Bindle" : "Start my own Bindle"}
        </button>
      </span>
    </div>
  );
}
