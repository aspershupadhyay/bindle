// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Settings (handbook P6 and P9). Every change saves at once.

import { useEffect, useState } from "react";
import { AvatarPicker } from "../components/Avatar";
import { AiSection } from "../components/settings/AiSection";
import { CloudSection } from "../components/settings/CloudSection";
import { DataSection } from "../components/settings/DataSection";
import { useNotify } from "../components/Undo";
import { commands, errorMessage, type AppInfo, type Settings as SettingsData } from "../lib/commands";
import { useSettings } from "../lib/settings";
import { updateNote, useUpdates } from "../lib/updates";

const themes: { value: SettingsData["theme"]; label: string }[] = [
  { value: "system", label: "Same as system" },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
];

export function Settings({ onOpenSpace }: { onOpenSpace?: (id: string) => void }) {
  const { settings, update } = useSettings();
  const notify = useNotify();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [name, setName] = useState(settings.name ?? "");
  const updates = useUpdates();
  const updating = updates.state.kind === "checking" || updates.state.kind === "installing" || updates.state.kind === "restarting";

  useEffect(() => {
    commands.appInfo().then(setInfo, (e) => notify(errorMessage(e)));
  }, [notify]);

  const save = (patch: Partial<SettingsData>) => update(patch).catch((e) => notify(`Not saved: ${errorMessage(e)}`));
  const time = (value: string) => (value ? `${value}:00` : undefined);

  return (
    <div className="page settings-page">
      <h1 className="display">Settings</h1>

      <section className="card settings-card" aria-labelledby="you">
        <h2 id="you">You</h2>
        <label className="field">
          <span className="field-label">What Bindle calls you</span>
          <input value={name} onChange={(e) => setName(e.target.value)} onBlur={() => void save({ name: name.trim() || null })} placeholder="Your first name" />
        </label>
        <div className="field">
          <span className="field-label">Your picture</span>
          <AvatarPicker name={settings.name ?? "Me"} value={settings.avatar} onPick={(avatar) => void save({ avatar })} />
        </div>
      </section>

      <section className="card settings-card" aria-labelledby="evening">
        <h2 id="evening">Wrap up</h2>
        <label className="field">
          <span className="field-label">Time</span>
          <input type="time" value={settings.wrap_up_time.slice(0, 5)} onChange={(e) => time(e.target.value) && void save({ wrap_up_time: time(e.target.value) })} />
        </label>
        <label className="check">
          <input type="checkbox" checked={settings.wrap_up_reminder} onChange={(e) => void save({ wrap_up_reminder: e.target.checked })} />
          Remind me at this time, unless I already wrapped up
        </label>
      </section>

      <section className="card settings-card" aria-labelledby="reminders">
        <h2 id="reminders">Reminders</h2>
        <label className="check">
          <input type="checkbox" checked={settings.timed_reminders} onChange={(e) => void save({ timed_reminders: e.target.checked })} />
          At the time I give a promise
        </label>
        <div className="settings-row">
          <label className="check">
            <input type="checkbox" checked={settings.morning_note} onChange={(e) => void save({ morning_note: e.target.checked })} />
            A short note about my day, at
          </label>
          <input type="time" aria-label="Morning note time" value={settings.morning_time.slice(0, 5)} onChange={(e) => time(e.target.value) && void save({ morning_time: time(e.target.value) })} />
        </div>
        <p className="muted small">
          Bindle is quiet by default: at most three reminders a day, none between 10 PM and 7 AM, and never about being late.
        </p>
        <div>
          <button type="button" className="button-quiet" onClick={() => void commands.testNotification().then(() => notify("Sent a test reminder."))}>
            Send a test reminder
          </button>
        </div>
      </section>

      <section className="card settings-card" aria-labelledby="look">
        <h2 id="look">Look</h2>
        <div className="pills" role="group" aria-labelledby="look">
          {themes.map((t) => (
            <button key={t.value} type="button" className="pill" aria-pressed={settings.theme === t.value} onClick={() => void save({ theme: t.value })}>
              {t.label}
            </button>
          ))}
        </div>
      </section>

      <section className="card settings-card" aria-labelledby="work">
        <h2 id="work">Work</h2>
        <label className="field">
          <span className="field-label">The Board's Making lane holds</span>
          <input
            type="number"
            min={1}
            max={9}
            value={settings.making_limit}
            onChange={(e) => {
              const limit = Number(e.target.value);
              if (limit >= 1 && limit <= 9) void save({ making_limit: limit });
            }}
          />
        </label>
        <p className="muted small">When more than this are in progress, the Board says so. It never stops you. GitHub and Gmail are connected from Code and Inbox.</p>
      </section>

      <CloudSection onOpenSpace={onOpenSpace} />
      <AiSection />
      <DataSection />

      <section className="card settings-card" aria-labelledby="keys">
        <h2 id="keys">Shortcuts</h2>
        <dl className="keys">
          <dt><kbd>Alt</kbd> <kbd>Shift</kbd> <kbd>J</kbd></dt>
          <dd>Jot down, from anywhere</dd>
          <dt><kbd>Ctrl</kbd> <kbd>K</kbd></dt>
          <dd>Search every promise</dd>
          <dt><kbd>Ctrl</kbd> <kbd>Z</kbd></dt>
          <dd>Undo the last change</dd>
        </dl>
      </section>

      <section className="card settings-card about" aria-labelledby="about">
        <h2 id="about">About Bindle</h2>
        {info && (
          <dl>
            <dt>Version</dt>
            <dd>{info.version}</dd>
            <dt>Data version</dt>
            <dd>{info.schema_version}</dd>
          </dl>
        )}
        <div className="update-actions">
          {updates.state.kind === "available" ? (
            <button type="button" className="button-primary small-button" onClick={() => void updates.install()}>
              Install {updates.state.version} and restart
            </button>
          ) : (
            <button type="button" className="button-quiet small-button" disabled={updating} onClick={() => void updates.check()}>
              Check for a new version
            </button>
          )}
        </div>
        <p className="muted small" role="status" aria-live="polite">
          {updateNote(updates.state) || "Bindle looks for new versions by itself and asks before installing one."}
        </p>
        <p className="muted small">Everything stays on this device. Your promises are never sent anywhere unless you connect an AI app, and then only what you allow above. To find new versions, Bindle only asks GitHub which version is newest.</p>
      </section>
    </div>
  );
}
