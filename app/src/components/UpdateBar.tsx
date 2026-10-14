// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The bar at the top of every screen when a new version of Bindle is ready,
// while it installs, and if installing fails. Errors from looking for one
// show only in Settings.

import { updateNote, useUpdates } from "../lib/updates";

export function UpdateBar() {
  const { state, dismissed, install, later } = useUpdates();

  if (state.kind === "available" && !dismissed) {
    return (
      <section className="update-bar" aria-label="New version">
        <p>
          <strong>Bindle {state.version} is here.</strong> Install it now and Bindle restarts in a few seconds. Your promises stay as they are.
        </p>
        <div className="update-actions">
          <button type="button" className="button-primary small-button" onClick={() => void install()}>
            Install and restart
          </button>
          <button type="button" className="button-quiet small-button" onClick={later}>
            Later
          </button>
        </div>
      </section>
    );
  }

  if (state.kind === "installing" || state.kind === "restarting" || (state.kind === "error" && state.version && !dismissed)) {
    return (
      <section className={`update-bar ${state.kind === "error" ? "error" : ""}`} aria-label="New version" role="status">
        <p>{updateNote(state)}</p>
        {state.kind === "error" && (
          <div className="update-actions">
            <button type="button" className="button-quiet small-button" onClick={later}>
              Close
            </button>
          </div>
        )}
      </section>
    );
  }

  return null;
}
