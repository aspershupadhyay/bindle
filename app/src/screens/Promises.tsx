// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Promises (handbook P6): every promise, by state, and search across all of
// them. Ctrl+K or "/" anywhere comes here with the search box ready.

import { useCallback, useEffect, useRef, useState } from "react";
import { BrowseRow } from "../components/BrowseRow";
import { Icon } from "../components/Icon";
import { commands, errorMessage, onPromisesChanged, type Promise as BindlePromise, type State, type StateCounts } from "../lib/commands";

const tabs: { state: State; label: string; empty: string }[] = [
  { state: "open", label: "Open", empty: "Nothing open. Promises you keep in Sort show up here." },
  { state: "later", label: "Later", empty: "Nothing kept for later." },
  { state: "done", label: "Done", empty: "Nothing done yet. It will come." },
  { state: "let_go", label: "Let go", empty: "Nothing let go. Letting go is allowed, and it stays in history." },
];

interface PromisesProps {
  /** Changes whenever the search box should take focus (Ctrl+K). */
  focusSearch: number;
  onOpenSort: () => void;
}

export function Promises({ focusSearch, onOpenSort }: PromisesProps) {
  const [tab, setTab] = useState<State>("open");
  const [counts, setCounts] = useState<StateCounts | null>(null);
  const [list, setList] = useState<BindlePromise[]>([]);
  const [query, setQuery] = useState("");
  const [found, setFound] = useState<BindlePromise[] | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const searchBox = useRef<HTMLInputElement>(null);
  const latest = useRef(0);

  const refresh = useCallback(() => {
    const failed = (error: unknown) => setProblem(`Could not load promises: ${errorMessage(error)}`);
    commands.stateCounts().then(setCounts, failed);
    commands.promisesIn(tab).then((l) => (setList(l), setProblem(null)), failed);
  }, [tab]);

  useEffect(() => {
    refresh();
    return onPromisesChanged(refresh);
  }, [refresh]);

  useEffect(() => searchBox.current?.focus(), [focusSearch]);

  // Search as you type. Late answers for older text are ignored.
  const runSearch = useCallback(() => {
    const ask = ++latest.current;
    if (query.trim() === "") {
      setFound(null);
      return;
    }
    commands.searchPromises(query).then(
      (results) => ask === latest.current && setFound(results),
      (error) => ask === latest.current && (setFound([]), setProblem(`Search failed: ${errorMessage(error)}`)),
    );
  }, [query]);

  useEffect(() => {
    runSearch();
    return onPromisesChanged(runSearch);
  }, [runSearch]);

  const current = tabs.find((t) => t.state === tab)!;
  return (
    <div className="page">
      <header className="sort-head">
        <h1 className="display">Promises</h1>
        <label className="search-field">
          <Icon name="search" size={18} />
          <span className="visually-hidden">Search every promise</span>
          <input
            ref={searchBox}
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Escape" && setQuery("")}
            placeholder="Search every promise, person or first step"
            autoComplete="off"
            spellCheck={false}
          />
          <span className="key-hint">Ctrl K</span>
        </label>
      </header>

      {problem && <p className="error-text" role="alert">{problem}</p>}

      {found !== null ? (
        <section aria-labelledby="results">
          <div className="section-head">
            <h2 id="results">{found.length === 0 ? "Nothing found" : found.length === 1 ? "1 found" : `${found.length} found`}</h2>
            <button type="button" className="link-button small" onClick={() => setQuery("")}>
              Clear
            </button>
          </div>
          {found.length > 0 && (
            <ul className="card list">
              {found.map((p) => (
                <BrowseRow key={p.id} promise={p} showState onOpenSort={onOpenSort} />
              ))}
            </ul>
          )}
        </section>
      ) : (
        <section aria-label={current.label}>
          <div className="pills tabs" role="tablist" aria-label="Promises by state">
            {tabs.map((t) => (
              <button key={t.state} type="button" role="tab" className="pill" aria-selected={tab === t.state} aria-pressed={tab === t.state} onClick={() => setTab(t.state)}>
                {t.label}
                {counts && <span className="tab-count">{counts[t.state]}</span>}
              </button>
            ))}
          </div>
          {list.length === 0 ? (
            <p className="card empty">{current.empty}</p>
          ) : (
            <ul className="card list">
              {list.map((p) => (
                <BrowseRow key={p.id} promise={p} onOpenSort={onOpenSort} />
              ))}
            </ul>
          )}
        </section>
      )}
    </div>
  );
}
