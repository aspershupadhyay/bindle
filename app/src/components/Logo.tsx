// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The Bindle mark. On a dark theme it switches to the brand sheet's "on dark"
// version, whose ink outline drops away so the bag stays visible.

import dark from "../art/bindle-24-dark.png";
import light from "../art/bindle-24.png";

/** 24px art, shown at a whole-number scale. */
export function Logo({ size = 48, alt = "" }: { size?: 24 | 48 | 72 | 96; alt?: string }) {
  return (
    <>
      <img className="sprite logo-light" src={light} alt={alt} width={size} height={size} />
      <img className="sprite logo-dark" src={dark} alt={alt} width={size} height={size} />
    </>
  );
}
