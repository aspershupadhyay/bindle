// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Changing how a person shows: their name, picture and pronouns, and whether
// a connected AI may see them. Each part saves on its own.

import { useState } from "react";
import { commands, errorMessage, type PersonDetail } from "../lib/commands";
import { AvatarPicker } from "./Avatar";
import { useNotify } from "./Undo";

export function PersonEdit({ person, onDone }: { person: PersonDetail; onDone: () => void }) {
  const notify = useNotify();
  const [name, setName] = useState(person.name);
  const [pronouns, setPronouns] = useState(person.pronouns ?? "");

  const tryTo = async (work: () => globalThis.Promise<void>, saved: string) => {
    try {
      await work();
      notify(saved);
    } catch (error) {
      notify(errorMessage(error));
    }
  };

  const rename = () =>
    name.trim() !== person.name && void tryTo(() => commands.renamePerson(person.id, name), "Renamed.");
  const savePronouns = () =>
    pronouns.trim() !== (person.pronouns ?? "") &&
    void tryTo(() => commands.setPersonLook(person.id, person.avatar, pronouns.trim() || null), "Saved.");

  return (
    <section className="card person-look" aria-label={`Edit ${person.name}`}>
      <form className="settings-row" onSubmit={(e) => (e.preventDefault(), rename())}>
        <label className="field">
          <span className="field-label">Name</span>
          <input value={name} onChange={(e) => setName(e.target.value)} onBlur={rename} />
        </label>
        <label className="field">
          <span className="field-label">Pronouns <span className="muted">Optional, in their words</span></span>
          <input value={pronouns} onChange={(e) => setPronouns(e.target.value)} onBlur={savePronouns} placeholder="Like they/them" />
        </label>
      </form>
      <div className="field">
        <span className="field-label">Picture <span className="muted">Pick whichever feels like them</span></span>
        <AvatarPicker
          name={person.name}
          value={person.avatar}
          onPick={(avatar) => void tryTo(() => commands.setPersonLook(person.id, avatar, person.pronouns), "Picture changed.")}
        />
      </div>
      <label className="check">
        <input
          type="checkbox"
          checked={person.hidden_from_ai}
          onChange={(e) => void tryTo(() => commands.setPersonHidden(person.id, e.target.checked), e.target.checked ? "Kept from AI." : "AI can see them again.")}
        />
        Keep {person.name} from AI. A connected AI never sees them or their promises.
      </label>
      <div>
        <button type="button" className="button-quiet" onClick={onDone}>Done</button>
      </div>
    </section>
  );
}
