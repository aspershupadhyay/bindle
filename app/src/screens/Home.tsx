// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Life Home (v2 design, Main; handbook P6): today's top 3, what is running
// late, what others owe you, what is coming up, and how full the week is.

import { forwardRef, useCallback, useEffect, useState } from "react";
import hills from "../art/hills.png";
import { TopThree } from "../components/home/TopThree";
import { RunningLate } from "../components/home/RunningLate";
import { Waiting } from "../components/home/Waiting";
import { WeekStrip } from "../components/home/WeekStrip";
import { JotField } from "../components/JotField";
import { PromiseRow } from "../components/PromiseRow";
import { commands, onPromisesChanged, type HomeView } from "../lib/commands";
import { dateLine, greeting } from "../lib/days";

export const Home = forwardRef<HTMLInputElement, { onOpenSort: () => void; unsorted: number }>(function Home(
  { onOpenSort, unsorted },
  jotRef,
) {
  const now = new Date();
  const [view, setView] = useState<HomeView | null>(null);

  const refresh = useCallback(() => {
    commands.homeView().then(setView, () => setView(null));
  }, []);

  useEffect(() => {
    refresh();
    // A day boundary or a change anywhere both mean Home should look again.
    const timer = setInterval(refresh, 60_000);
    const stop = onPromisesChanged(refresh);
    return () => {
      clearInterval(timer);
      stop();
    };
  }, [refresh]);

  return (
    <div className="page">
      <header className="home-header">
        <div className="home-hello">
          <div className="muted small">{dateLine(now)}</div>
          <h1 className="display">{greeting(now)}.</h1>
          {view && <Summary view={view} unsorted={unsorted} onOpenSort={onOpenSort} />}
          <JotField ref={jotRef} />
        </div>
        <figure className="scene-frame">
          <img
            className="sprite scene"
            src={hills}
            alt="Pixel art: a traveller with a bindle walking over green hills at sunset"
            width={384}
            height={160}
          />
        </figure>
      </header>

      {view && (
        <div className="home-columns">
          <div className="home-today" aria-label="Today">
            <TopThree top3={view.top3} suggestions={view.suggestions} />
            <RunningLate promises={view.running_late} />
            <Waiting waiting={view.waiting_on_others} />
            <section aria-labelledby="coming-up">
              <div className="section-head">
                <h2 id="coming-up">Coming up</h2>
                <span className="muted small">{view.coming_up.length === 0 ? "Nothing yet" : "Soonest first"}</span>
              </div>
              {view.coming_up.length === 0 ? (
                <p className="card empty">Promises you keep in Sort show up here, soonest first.</p>
              ) : (
                <ul className="card list">
                  {view.coming_up.map((p) => (
                    <PromiseRow key={p.id} promise={p} canAddToTop3={view.top3.length < 3} />
                  ))}
                </ul>
              )}
            </section>
          </div>
          <aside className="home-side" aria-label="This week">
            {view.week.length > 0 && <WeekStrip view={view} />}
          </aside>
        </div>
      )}
    </div>
  );
});

/** "3 things today, 2 people waiting on you." Plain, and never a score. */
function Summary({ view, unsorted, onOpenSort }: { view: HomeView; unsorted: number; onOpenSort: () => void }) {
  const today = view.today_count;
  const people = view.people_waiting_on_you;
  const things = today === 0 ? "Nothing planned for today" : today === 1 ? "1 thing today" : `${today} things today`;
  return (
    <p className="lead">
      {things}
      {people > 0 ? (
        <>
          , <span className="accent">{people === 1 ? "1 person" : `${people} people`} waiting on you.</span>
        </>
      ) : (
        "."
      )}
      {unsorted > 0 && (
        <>
          {" "}
          <button type="button" className="link-button" onClick={onOpenSort}>
            {unsorted} to sort
          </button>
        </>
      )}
    </p>
  );
}
