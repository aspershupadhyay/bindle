// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import type { Chip } from "../lib/chips";

export function Chips({ chips, label }: { chips: Chip[]; label: string }) {
  if (chips.length === 0) return null;
  return (
    <ul className="chips" aria-label={label}>
      {chips.map((chip) => (
        <li key={chip.key} className={`chip ${chip.tone}`} title={chip.hint}>
          {chip.label}
          {chip.hint && <span className="visually-hidden">. {chip.hint}</span>}
        </li>
      ))}
    </ul>
  );
}
