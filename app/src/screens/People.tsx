// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// People (handbook P6): everyone your promises are with, what you owe each
// other, and a recap you can send. Bindle copies the recap; you send it.

import { useCallback, useEffect, useState } from "react";
import { Avatar } from "../components/Avatar";
import { Chips } from "../components/Chips";
import { PersonEdit } from "../components/PersonEdit";
import { ScreenArt, ScreenTitle } from "../components/ScreenTitle";
import { useNotify } from "../components/Undo";
import { chipsForPromise } from "../lib/chips";
import { commands, errorMessage, onPromisesChanged, type PersonDetail, type PersonSummary, type Promise as BindlePromise } from "../lib/commands";
import { recapMessage } from "../lib/recap";

export function People() {
  const notify = useNotify();
  const [people, setPeople] = useState<PersonSummary[] | null>(null);
  const [archived, setArchived] = useState<PersonSummary[]>([]);
  const [chosen, setChosen] = useState<string | null>(null);
  const [filter, setFilter] = useState("");

  const refresh = useCallback(() => {
    commands.peopleList().then(setPeople, () => setPeople([]));
    commands.archivedPeople().then(setArchived, () => setArchived([]));
  }, []);

  async function bringBack(person: PersonSummary) {
    try {
      await commands.unarchivePerson(person.id);
      setChosen(person.id);
      notify(`${person.name} is back on People.`);
    } catch (error) {
      notify(errorMessage(error));
    }
    refresh();
  }

  function archivedNow(name: string) {
    setChosen(null);
    notify(`${name} archived. They come back when a promise names them, or from Archived.`);
    refresh();
  }

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  // Open the first person once the list arrives, so the page is never blank.
  useEffect(() => {
    if (chosen === null && people && people.length > 0) setChosen(people[0].id);
  }, [people, chosen]);

  if (people === null) return <div className="page" />;
  const shown = people.filter((p) => p.name.toLowerCase().includes(filter.trim().toLowerCase()));
  return (
    <div className="page">
      <header className="sort-head">
        <ScreenTitle art="people">People</ScreenTitle>
        <p className="lead">{people.length > 0 ? "Who you owe, and who owes you." : archived.length > 0 ? "Everyone here is archived." : "No one yet."}</p>
      </header>
      {people.length === 0 && archived.length === 0 ? (
        <div className="card all-sorted">
          <ScreenArt name="people" scale={3} />
          <p>When a promise names someone, like “send the deck to Priya”, they appear here.</p>
        </div>
      ) : (
        <div className="people-columns">
          <div className="people-list">
            {people.length > 6 && (
              <input className="people-filter" value={filter} onChange={(e) => setFilter(e.target.value)} placeholder="Find someone" aria-label="Find someone" />
            )}
            {shown.length > 0 && (
              <ul className="card list">
                {shown.map((p) => (
                  <li key={p.id}>
                    <button type="button" className="person-button" aria-current={chosen === p.id ? "true" : undefined} onClick={() => setChosen(p.id)}>
                      <Avatar name={p.name} avatar={p.avatar} size={32} />
                      <span className="promise-main">
                        <span className="promise-title">{p.name}</span>
                        <span className="muted small">{owes(p)}</span>
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
            {archived.length > 0 && (
              <details className="people-archived">
                <summary className="muted small">Archived ({archived.length})</summary>
                <ul className="card list">
                  {archived.map((p) => (
                    <li key={p.id} className="archived-row">
                      <Avatar name={p.name} avatar={p.avatar} size={32} />
                      <span className="promise-title">{p.name}</span>
                      <button type="button" className="link-button small" onClick={() => void bringBack(p)} aria-label={`Bring back ${p.name}`}>
                        Bring back
                      </button>
                    </li>
                  ))}
                </ul>
              </details>
            )}
          </div>
          {chosen && people.some((p) => p.id === chosen) && <Person key={chosen} id={chosen} onArchived={archivedNow} />}
        </div>
      )}
    </div>
  );
}

function owes(p: PersonSummary): string {
  if (p.you_owe === 0 && p.they_owe === 0) return "Nothing open";
  const parts: string[] = [];
  if (p.you_owe > 0) parts.push(`You owe ${p.you_owe}`);
  if (p.they_owe > 0) parts.push(`They owe ${p.they_owe}`);
  return parts.join(" · ");
}

function Person({ id, onArchived }: { id: string; onArchived: (name: string) => void }) {
  const notify = useNotify();
  const [person, setPerson] = useState<PersonDetail | null>(null);
  const [editing, setEditing] = useState(false);

  const refresh = useCallback(() => {
    commands.personDetail(id).then(setPerson, () => setPerson(null));
  }, [id]);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  if (!person) return <div className="person-detail" />;
  const recap = recapMessage(person, new Date());
  const nothingOpen = person.you_owe.length === 0 && person.they_owe.length === 0;

  async function archive() {
    try {
      await commands.archivePerson(id);
      onArchived(person!.name);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  async function copyRecap() {
    try {
      await navigator.clipboard.writeText(recap!);
      notify("Recap copied. Paste it wherever you talk.");
    } catch {
      notify("Could not copy. Select the recap and copy it yourself.");
    }
  }

  return (
    <div className="person-detail">
      <div className="person-head">
        <Avatar name={person.name} avatar={person.avatar} size={64} />
        <div>
          <h2 className="person-name">{person.name}</h2>
          {person.pronouns && <div className="muted small">{person.pronouns}</div>}
        </div>
        <button type="button" className="link-button small" aria-expanded={editing} onClick={() => setEditing(!editing)}>
          {editing ? "Close" : "Edit"}
        </button>
      </div>
      {editing && <PersonEdit person={person} onDone={() => setEditing(false)} />}
      {person.hidden_from_ai && <p className="muted small">Kept from AI: a connected AI never sees {person.name} or these promises.</p>}

      <Side title="You'll do" empty={`Nothing you owe ${person.name}.`} promises={person.you_owe} />
      <Side title={`${person.name} will do`} empty={`Nothing ${person.name} owes you.`} promises={person.they_owe} />
      {person.kept_lately.length > 0 && <Side title="Kept lately" empty="" promises={person.kept_lately} />}

      {nothingOpen && (
        <p className="muted small person-archive">
          Nothing open with {person.name}.{" "}
          <button type="button" className="link-button" onClick={() => void archive()}>
            Archive {person.name}
          </button>
        </p>
      )}

      {recap && (
        <section className="card recap" aria-labelledby="recap">
          <div className="section-head">
            <h3 id="recap">Send recap</h3>
            <button type="button" className="button-primary small-primary" onClick={() => void copyRecap()}>
              Copy recap
            </button>
          </div>
          <pre className="recap-text">{recap}</pre>
          <p className="muted small">Bindle never sends messages for you. Copy it, then paste it wherever you talk.</p>
        </section>
      )}
    </div>
  );
}

function Side({ title, empty, promises }: { title: string; empty: string; promises: BindlePromise[] }) {
  return (
    <section>
      <div className="section-head">
        <h3>{title}</h3>
        <span className="muted small">{promises.length || ""}</span>
      </div>
      {promises.length === 0 ? (
        <p className="card empty">{empty}</p>
      ) : (
        <ul className="card list">
          {promises.map((p) => (
            <li key={p.id}>
              <span className={p.state === "done" ? "promise-title done" : "promise-title"}>{p.title}</span>
              <Chips chips={chipsForPromise(p, new Date()).filter((c) => c.key !== "person" && c.key !== "direction")} label="Details" />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
