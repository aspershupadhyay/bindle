// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Stands in for a screen that is not built yet, and says when it will be.

import type { ScreenInfo } from "../screens";

const scenes = import.meta.glob<string>("../art/*.png", { eager: true, import: "default" });

export function Soon({ screen }: { screen: ScreenInfo }) {
  const scene = screen.scene ? scenes[`../art/${screen.scene}.png`] : undefined;
  return (
    <div className="page">
      <header className="soon">
        {scene && (
          <figure className="scene-frame">
            <img className="sprite scene" src={scene} alt="" width={384} height={160} />
          </figure>
        )}
        <h1 className="display">{screen.label}</h1>
        <p className="lead">On its way. This screen is built in {screen.week} of the plan.</p>
      </header>
    </div>
  );
}
