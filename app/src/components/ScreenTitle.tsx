// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A screen's title with its pixel icon, shown at 2x (art/README.md).

import type { ReactNode } from "react";
import people from "../art/people.png";
import promises from "../art/promises.png";
import sort from "../art/sort.png";
import wrapUp from "../art/wrap-up.png";

const art = { sort, promises, people, wrapUp } as const;

/** The pixel icon for a screen, at a whole-number scale. */
export function ScreenArt({ name, scale = 2 }: { name: keyof typeof art; scale?: 1 | 2 | 3 | 4 }) {
  return <img className="sprite" src={art[name]} alt="" width={32 * scale} height={32 * scale} />;
}

export function ScreenTitle({ art: name, children }: { art: keyof typeof art; children: ReactNode }) {
  return (
    <div className="screen-title">
      <ScreenArt name={name} />
      <h1 className="display">{children}</h1>
    </div>
  );
}
