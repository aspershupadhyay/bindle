// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A person's picture, and the picker to change it. Someone with no picture
// chosen gets one picked from their name, so the same name always looks the
// same and nobody is a bare letter. Pictures name what is drawn, never who someone is:
// anyone can pick any of them, for themselves or anyone else.

import beard from "../art/av-beard.png";
import bun from "../art/av-bun.png";
import cap from "../art/av-cap.png";
import curly from "../art/av-curly.png";
import grey from "../art/av-grey.png";
import long from "../art/av-long.png";
import scarf from "../art/av-scarf.png";
import short from "../art/av-short.png";
import turban from "../art/av-turban.png";
import { AVATARS, type Avatar as AvatarName } from "../lib/commands";

const art: Record<AvatarName, string> = { short, long, curly, bun, beard, turban, scarf, cap, grey };

/** What each picture shows, for screen readers and the picker. */
export const avatarLabels: Record<AvatarName, string> = {
  short: "Short hair",
  long: "Long hair",
  curly: "Curly hair",
  bun: "Hair in a bun",
  beard: "A beard",
  turban: "A turban",
  scarf: "A headscarf",
  cap: "A cap",
  grey: "Grey hair",
};

const TINTS = ["honey", "sage", "clay"] as const;

/** A small, stable number from a name: the same name always gives the same. */
function hashOf(name: string): number {
  let hash = 7;
  for (const c of name.trim().toLowerCase()) hash = (hash * 31 + c.charCodeAt(0)) >>> 0;
  return hash;
}

/** The picture for someone with none chosen, picked from their name. */
export function pictureFor(name: string): AvatarName {
  return AVATARS[hashOf(name) % AVATARS.length];
}

/** 16px art, so sizes are whole-number scales: 16, 32, 48 or 64. */
export function Avatar({ name, avatar, size = 32 }: { name: string; avatar: AvatarName | null; size?: 16 | 32 | 48 | 64 }) {
  const picture = avatar && art[avatar] ? avatar : pictureFor(name);
  const tint = TINTS[hashOf(name) % TINTS.length];
  return (
    <span className={`avatar-tile ${tint}`} style={{ width: size, height: size }} aria-hidden="true">
      <img className="sprite avatar" src={art[picture]} alt="" width={size} height={size} />
    </span>
  );
}

/** Two or more people side by side, overlapping a little, as in Ours. */
export function AvatarPair({ people, size = 32 }: { people: { name: string; avatar: AvatarName | null }[]; size?: 16 | 32 }) {
  return (
    <span className="avatar-pair" aria-hidden="true">
      {people.slice(0, 3).map((p) => (
        <Avatar key={p.name} name={p.name} avatar={p.avatar} size={size} />
      ))}
    </span>
  );
}

/** Pick a picture, or let Bindle pick one from the name. */
export function AvatarPicker({ name, value, onPick }: { name: string; value: AvatarName | null; onPick: (avatar: AvatarName | null) => void }) {
  return (
    <div className="avatar-picker" role="group" aria-label={`Picture for ${name}`}>
      <button type="button" className="avatar-choice" aria-pressed={value === null} title="Pick one for me" onClick={() => onPick(null)}>
        <Avatar name={name} avatar={null} size={48} />
        <span className="visually-hidden">Pick one for me</span>
      </button>
      {AVATARS.map((a) => (
        <button key={a} type="button" className="avatar-choice" aria-pressed={value === a} title={avatarLabels[a]} onClick={() => onPick(a)}>
          <Avatar name={name} avatar={a} size={48} />
          <span className="visually-hidden">{avatarLabels[a]}</span>
        </button>
      ))}
    </div>
  );
}
