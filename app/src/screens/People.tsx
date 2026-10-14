// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// People (handbook P6): everyone your promises are with, what you owe each
// other, and a recap you can send. Bindle copies the recap; you send it.

import { useCallback, useEffect, useState } from "react";
import { Chips } from "../components/Chips";
import { useNotify } from "../components/Undo";
import { chipsForPromise } from "../lib/chips";
import { commands, errorMessage, onPromisesChanged, type PersonDetail, type PersonSummary, type Promise as BindlePromise } from "../lib/commands";
import { recapMessage } from "../lib/recap";

export function People() {
  const [people, setPeople] = useState<PersonSummary[] | null>(null);
  const [chosen, setChosen] = useState<string | null>(null);
  const [filter, setFilter] = useState("");

  const refresh = useCallback(() => {
    commands.peopleList().then(setPeople, () => setPeople([]));
  }, []);

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
        <h1 className="display">People</h1>
        <p className="lead">{people.length === 0 ? "No one yet." : "Who you owe, and who owes you."}</p>
      </header>
      {people.length === 0 ? (
        <p className="card empty">When a promise names someone, like “send the deck to Priya”, they appear here.</p>
      ) : (
        <div className="people-columns">
          <div className="people-list">
            {people.length > 6 && (
              <input className="people-filter" value={filter} onChange={(e) => setFilter(e.target.value)} placeholder="Find someone" aria-label="Find someone" />
            )}
            <ul className="card list">
              {shown.map((p) => (
                <li key={p.id}>
                  <button type="button" className="person-button" aria-current={chosen === p.id ? "true" : undefined} onClick={() => setChosen(p.id)}>
                    <span className="initial honey" aria-hidden="true">{p.name.charAt(0).toUpperCase()}</span>
                    <span className="promise-main">
                      <span className="promise-title">{p.name}</span>
                      <span className="muted small">{owes(p)}</span>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </div>
          {chosen && <Person key={chosen} id={chosen} />}
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

function Person({ id }: { id: string }) {
  const notify = useNotify();
  const [person, setPerson] = useState<PersonDetail | null>(null);
  const [renaming, setRenaming] = useState<string | null>(null);

  const refresh = useCallback(() => {
    commands.personDetail(id).then(setPerson, () => setPerson(null));
  }, [id]);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  if (!person) return <div className="person-detail" />;
  const recap = recapMessage(person, new Date());

  async function copyRecap() {
    try {
      await navigator.clipboard.writeText(recap!);
      notify("Recap copied. Paste it wherever you talk.");
    } catch {
      notify("Could not copy. Select the recap and copy it yourself.");
    }
  }

  async function rename() {
    if (renaming === null) return;
    try {
      await commands.renamePerson(id, renaming);
      setRenaming(null);
    } catch (error) {
      notify(errorMessage(error));
    }
  }

  return (
    <div className="person-detail">
      <div className="person-head">
        {renaming === null ? (
          <>
            <h2 className="person-name">{person.name}</h2>
            <button type="button" className="link-button small" onClick={() => setRenaming(person.name)}>
              Rename
            </button>
          </>
        ) : (
          <form className="rename" onSubmit={(e) => (e.preventDefault(), void rename())}>
            <input value={renaming} onChange={(e) => setRenaming(e.target.value)} aria-label="New name" autoFocus />
            <button type="submit" className="button-primary small-primary">Save</button>
            <button type="button" className="button-quiet" onClick={() => setRenaming(null)}>Cancel</button>
          </form>
        )}
      </div>

      <Side title="You'll do" empty={`Nothing you owe ${person.name}.`} promises={person.you_owe} />
      <Side title={`${person.name} will do`} empty={`Nothing ${person.name} owes you.`} promises={person.they_owe} />
      {person.kept_lately.length > 0 && <Side title="Kept lately" empty="" promises={person.kept_lately} />}

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
