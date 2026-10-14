// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Settings, AI connection (handbook P8): how far a connected AI may go, how
// to connect one, and what connected ones read or suggested lately.

import { useCallback, useEffect, useState } from "react";
import { commands, errorMessage, onPromisesChanged, type AiAccess, type AiConnection } from "../../lib/commands";
import { useSettings } from "../../lib/settings";
import { useNotify } from "../Undo";

const levels: { value: AiAccess; label: string; note: string }[] = [
  { value: "off", label: "Off", note: "No AI can read or suggest anything." },
  { value: "read", label: "Read only", note: "A connected AI can read Home, promises and people. It cannot add anything." },
  {
    value: "suggest",
    label: "Read and suggest",
    note: "It can also suggest promises. Each one waits in Sort until you keep it or let it go, at most 10 at a time.",
  },
];

const toolWords: Record<string, string> = {
  read_home: "Read Home",
  list_promises: "Listed promises",
  search_promises: "Searched",
  read_people: "Read people",
  suggest_promise: "Suggested",
};

export function AiSection() {
  const { settings, update } = useSettings();
  const notify = useNotify();
  const [connection, setConnection] = useState<AiConnection | null>(null);

  const refresh = useCallback(() => {
    commands.aiConnection().then(setConnection, (e) => notify(errorMessage(e)));
  }, [notify]);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  async function copyConfig() {
    try {
      await navigator.clipboard.writeText(connection!.config);
      notify("Copied. Paste it into your AI app's MCP settings.");
    } catch {
      notify("Could not copy. Select the text and copy it yourself.");
    }
  }

  const level = levels.find((l) => l.value === settings.ai_access) ?? levels[0];
  return (
    <section className="card settings-card" aria-labelledby="ai">
      <h2 id="ai">AI connection</h2>
      <div className="pills" role="group" aria-labelledby="ai">
        {levels.map((l) => (
          <button key={l.value} type="button" className="pill" aria-pressed={settings.ai_access === l.value} onClick={() => void update({ ai_access: l.value }).catch((e) => notify(errorMessage(e)))}>
            {l.label}
          </button>
        ))}
      </div>
      <p className="muted small">
        {level.note} An AI never changes, finishes or removes a promise, and never sends anything. People you keep from AI stay hidden.
      </p>
      {settings.ai_access !== "off" && connection && (
        <>
          <p className="small">
            To connect Claude Desktop or another app that speaks MCP, add this to its MCP settings, then restart it:
          </p>
          <pre className="config-block">{connection.config}</pre>
          <div>
            <button type="button" className="button-quiet" onClick={() => void copyConfig()}>Copy</button>
          </div>
        </>
      )}
      {connection && connection.log.length > 0 && (
        <>
          <h3 className="small">Lately</h3>
          <ul className="ai-log">
            {connection.log.slice(0, 10).map((entry, i) => (
              <li key={`${entry.at}-${i}`}>
                <time dateTime={entry.at}>{when(entry.at)}</time>
                <span>
                  {entry.client}: {toolWords[entry.tool] ?? entry.tool}
                  {entry.detail && <span className="muted"> “{entry.detail}”</span>}
                </span>
              </li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}

function when(at: string): string {
  const date = new Date(at);
  const day = date.toDateString() === new Date().toDateString() ? "Today" : date.toLocaleDateString(undefined, { day: "numeric", month: "short" });
  return `${day} ${date.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" })}`;
}
