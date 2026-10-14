// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// People's pictures by name, for places that only know a promise's person.

import { useEffect, useState } from "react";
import { commands, onPromisesChanged, type Avatar } from "./commands";

/** Each person's picture, keyed by their name in lower case. */
export function usePictures(): Map<string, Avatar> {
  const [pictures, setPictures] = useState<Map<string, Avatar>>(new Map());
  useEffect(() => {
    const load = () =>
      commands.peopleList().then(
        (people) => setPictures(new Map(people.flatMap((p) => (p.avatar ? [[p.name.toLowerCase(), p.avatar] as const] : [])))),
        () => setPictures(new Map()),
      );
    load();
    return onPromisesChanged(load);
  }, []);
  return pictures;
}
