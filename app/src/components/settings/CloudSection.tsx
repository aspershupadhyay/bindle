// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Settings, Bindle Cloud (handbook P15): start with a recovery kit, bring
// Bindle back on a new device, and make or join an Ours space. Everything
// shared is sealed on this device first; the server cannot read it.

import { useCallback, useEffect, useState, type FormEvent } from "react";
import { commands, errorMessage, onPromisesChanged, type CloudStatus } from "../../lib/commands";
import { useSettings } from "../../lib/settings";
import { lookedAgo } from "../../lib/work";
import { useNotify } from "../Undo";

export function CloudSection({ onOpenSpace }: { onOpenSpace?: (id: string) => void }) {
  const { settings } = useSettings();
  const notify = useNotify();
  const [status, setStatus] = useState<CloudStatus | null>(null);
  const [kit, setKit] = useState<string | null>(null);
  const [relay, setRelay] = useState("");
  const [name, setName] = useState(settings.name ?? "");
  const [restoring, setRestoring] = useState(false);
  const [typedKit, setTypedKit] = useState("");
  const [spaceName, setSpaceName] = useState("");
  const [link, setLink] = useState("");
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  const refresh = useCallback(() => {
    commands.cloudStatus().then(setStatus, (e) => notify(errorMessage(e)));
  }, [notify]);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  async function run<T>(work: () => globalThis.Promise<T>, done?: (value: T) => void) {
    setBusy(true);
    setProblem(null);
    try {
      const value = await work();
      done?.(value);
      refresh();
    } catch (error) {
      setProblem(errorMessage(error));
    } finally {
      setBusy(false);
    }
  }

  const start = (e: FormEvent) => {
    e.preventDefault();
    void run(() => commands.cloudStart(relay, name), setKit);
  };
  const restore = (e: FormEvent) => {
    e.preventDefault();
    void run(() => commands.cloudRestore(relay, typedKit), () => notify("Bindle Cloud is back, with your shared spaces."));
  };

  if (!status) return null;
  const started = Boolean(status.account.account);

  return (
    <section className="card settings-card" aria-labelledby="cloud">
      <h2 id="cloud">Bindle Cloud</h2>
      {kit ? (
        <div className="kit">
          <p>
            <strong>Your recovery kit.</strong> Write it down or print it, and keep it somewhere safe. It is the only way to bring your shared spaces
            back on a new device. Bindle cannot show it to anyone else, and no one can reset it.
          </p>
          <p className="kit-code" aria-label="Recovery kit">
            {kit}
          </p>
          <div className="update-actions">
            <button type="button" className="button-quiet small-button" onClick={() => void navigator.clipboard?.writeText(kit).then(() => notify("Copied"))}>
              Copy
            </button>
            <button type="button" className="button-primary small-button" onClick={() => void run(() => commands.cloudKitSaved(), () => setKit(null))}>
              I saved it
            </button>
          </div>
        </div>
      ) : !started ? (
        <>
          <p className="muted small">
            Share an Ours space with someone you live or work with. What you share is locked on this device before it is sent, so the server keeps
            only sealed boxes it cannot open. Your Mine stays on this device.
          </p>
          <form className="cloud-form" onSubmit={restoring ? restore : start}>
            <label className="field">
              <span className="field-label">Bindle Cloud address</span>
              <input value={relay} onChange={(e) => setRelay(e.target.value)} placeholder="https://cloud.example.com" inputMode="url" />
            </label>
            {restoring ? (
              <label className="field">
                <span className="field-label">Your recovery kit</span>
                <textarea value={typedKit} onChange={(e) => setTypedKit(e.target.value)} rows={3} placeholder="7KQ2-M9XD-…" spellCheck={false} />
              </label>
            ) : (
              <label className="field">
                <span className="field-label">The name others see</span>
                <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Your first name" />
              </label>
            )}
            <div className="update-actions">
              <button type="submit" className="button-primary small-button" disabled={busy || !relay.trim() || (restoring ? !typedKit.trim() : !name.trim())}>
                {restoring ? "Bring it back" : "Start Bindle Cloud"}
              </button>
              <button type="button" className="link-button small" onClick={() => setRestoring(!restoring)}>
                {restoring ? "Start fresh instead" : "I have a recovery kit"}
              </button>
            </div>
          </form>
        </>
      ) : (
        <>
          <p className="muted small">
            You are <strong>{status.account.name}</strong> on {status.account.relay}.{" "}
            {status.account.kit_saved_at ? (
              "Recovery kit saved."
            ) : (
              <button type="button" className="link-button small" onClick={() => void run(() => commands.cloudRecoveryKit(), setKit)}>
                Your recovery kit is not saved yet. Show it
              </button>
            )}
          </p>
          {status.spaces.length > 0 && (
            <ul className="copies">
              {status.spaces.map((space) => (
                <li key={space.id} className="settings-row">
                  <button type="button" className="link-button" onClick={() => onOpenSpace?.(space.id)}>
                    {space.name}
                  </button>
                  <span className="muted small">
                    {space.promises} {space.promises === 1 ? "promise" : "promises"} · {space.waiting > 0 ? `${space.waiting} to send · ` : ""}
                    {lookedAgo(space.synced_at, new Date()).replace("Looked", "synced").replace("Not looked yet", "not synced yet")}
                  </span>
                </li>
              ))}
            </ul>
          )}
          <form
            className="cloud-form inline"
            onSubmit={(e) => {
              e.preventDefault();
              void run(() => commands.oursCreate(spaceName), (space) => {
                setSpaceName("");
                notify(`Made ${space.name}. Invite someone from its page.`);
                onOpenSpace?.(space.id);
              });
            }}
          >
            <label className="field">
              <span className="field-label">Make a shared space</span>
              <input value={spaceName} onChange={(e) => setSpaceName(e.target.value)} placeholder="Upadhyay Home" />
            </label>
            <button type="submit" className="button-quiet small-button" disabled={busy}>
              Make it
            </button>
          </form>
          <form
            className="cloud-form inline"
            onSubmit={(e) => {
              e.preventDefault();
              void run(() => commands.oursJoin(link), (space) => {
                setLink("");
                notify(`You joined ${space.name}.`);
                onOpenSpace?.(space.id);
              });
            }}
          >
            <label className="field">
              <span className="field-label">Join with an invite link</span>
              <input value={link} onChange={(e) => setLink(e.target.value)} placeholder="https://aspershupadhyay.github.io/bindle/join/#…" spellCheck={false} />
            </label>
            <button type="submit" className="button-quiet small-button" disabled={busy || !link.trim()}>
              Join
            </button>
          </form>
        </>
      )}
      {problem && (
        <p className="error-text small" role="alert">
          {problem}
        </p>
      )}
    </section>
  );
}
