// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Sort (handbook P6): everything jotted down waits here until the person
// decides what it is. One at a time, a few seconds each.

import { useCallback, useEffect, useState } from "react";
import { ScreenArt, ScreenTitle } from "../components/ScreenTitle";
import { SortCard } from "../components/SortCard";
import { commands, onPromisesChanged, type SortItem } from "../lib/commands";

export function Sort() {
  const [queue, setQueue] = useState<SortItem[] | null>(null);

  const refresh = useCallback(() => {
    commands.sortQueue().then(setQueue, () => setQueue([]));
  }, []);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  if (queue === null) return <div className="page" />;
  const [current, ...rest] = queue;
  return (
    <div className="page sort-page">
      <header className="sort-head">
        <ScreenTitle art="sort">Sort</ScreenTitle>
        <p className="lead">
          {queue.length === 0
            ? "All sorted."
            : queue.length === 1
              ? "1 to sort. A few seconds."
              : `${queue.length} to sort. A few seconds each.`}
        </p>
      </header>

      {current ? (
        <>
          <SortCard key={current.promise.id} item={current} />
          {rest.length > 0 && (
            <p className="muted small then">
              Then: {rest.slice(0, 3).map((i) => i.promise.title).join(" · ")}
              {rest.length > 3 && ` and ${rest.length - 3} more`}
            </p>
          )}
        </>
      ) : (
        <div className="card all-sorted">
          <ScreenArt name="sort" scale={3} />
          <p>Nothing waits here. Anything you jot down comes here until you decide what it is.</p>
        </div>
      )}
    </div>
  );
}
