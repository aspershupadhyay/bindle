// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Settings. For now it only says which build is running; the rest comes in week 6.

import { useEffect, useState } from "react";
import { commands, errorMessage, type AppInfo } from "../lib/commands";

export function Settings() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    commands.appInfo().then(setInfo, (e) => setError(errorMessage(e)));
  }, []);

  return (
    <div className="page">
      <h1 className="display">Settings</h1>
      <section className="card about" aria-label="About this build">
        <h2>About Bindle</h2>
        {error && <p className="error-text">{error}</p>}
        {info && (
          <dl>
            <dt>Version</dt>
            <dd>{info.version}</dd>
            <dt>Data version</dt>
            <dd>{info.schema_version}</dd>
          </dl>
        )}
        <p className="muted small">Everything stays on this device. Nothing is sent anywhere.</p>
      </section>
    </div>
  );
}
