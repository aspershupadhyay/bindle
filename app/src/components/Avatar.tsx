// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A person's picture, or their first letter when they have none, and the
// picker to change it. Pictures name what is drawn, never who someone is:
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

/** 16px art, so sizes are whole-number scales: 16, 32, 48 or 64. */
export function Avatar({ name, avatar, size = 32 }: { name: string; avatar: AvatarName | null; size?: 16 | 32 | 48 | 64 }) {
  if (avatar && art[avatar]) {
    return <img className="sprite avatar" src={art[avatar]} alt="" width={size} height={size} />;
  }
  return (
    <span className="initial honey" style={{ width: size, height: size, fontSize: size * 0.42 }} aria-hidden="true">
      {name.trim().charAt(0).toUpperCase() || "?"}
    </span>
  );
}

/** Pick a picture, or the first letter. */
export function AvatarPicker({ name, value, onPick }: { name: string; value: AvatarName | null; onPick: (avatar: AvatarName | null) => void }) {
  return (
    <div className="avatar-picker" role="group" aria-label={`Picture for ${name}`}>
      <button type="button" className="avatar-choice" aria-pressed={value === null} title="First letter" onClick={() => onPick(null)}>
        <Avatar name={name} avatar={null} size={48} />
        <span className="visually-hidden">First letter</span>
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
