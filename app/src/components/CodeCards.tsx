// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The Code screen's side cards (v2 design): a check that keeps failing,
// sixteen weeks of contributions, pull requests merged each week, and your
// repositories. Read only; "Open" goes to GitHub.

import { useState } from "react";
import { commands, errorMessage, type CodeCards, type Repo } from "../lib/commands";
import { useNotify } from "./Undo";

export function FailingCard({ cards, onPromise }: { cards: CodeCards; onPromise: () => void }) {
  const notify = useNotify();
  const failing = cards.failing;
  if (!failing) return null;
  return (
    <section className="card failing-card" aria-labelledby="failing">
      <div className="failing-head">
        <span className="failing-bug" aria-hidden="true">
          ✕
        </span>
        <div>
          <h2 id="failing">A check keeps failing</h2>
          <div className="small failing-where">
            {failing.workflow} · {failing.repo.split("/").pop()}
          </div>
        </div>
      </div>
      <p>
        It failed {failing.failed} of the last {failing.of} runs. Worth a look before it hides a real problem.
      </p>
      <div className="update-actions">
        <button type="button" className="button-dark small-primary" onClick={() => commands.openOnGithub(failing.url).catch((e) => notify(errorMessage(e)))}>
          Open logs
        </button>
        <button type="button" className="button-quiet small-button" onClick={onPromise}>
          Make it a promise
        </button>
      </div>
    </section>
  );
}

/** Sixteen weeks of contributions, a column per week, Monday at the top. */
export function CommitHeat({ cards }: { cards: CodeCards }) {
  const days = cards.days.slice(-16 * 7);
  if (days.length === 0) return null;
  const top = Math.max(1, ...days.map(([, n]) => n));
  const level = (n: number) => (n === 0 ? 0 : Math.min(4, Math.ceil((n / top) * 4)));
  // Pad the first week so every column starts on a Monday.
  const first = new Date(`${days[0][0]}T00:00:00`);
  const pad = (first.getDay() + 6) % 7;
  const cells: ([string, number] | null)[] = [...Array(pad).fill(null), ...days];
  const weeks: ([string, number] | null)[][] = [];
  for (let i = 0; i < cells.length; i += 7) weeks.push(cells.slice(i, i + 7));
  const total = days.reduce((sum, [, n]) => sum + n, 0);
  const month = (iso: string) => new Date(`${iso}T00:00:00`).toLocaleDateString("en-GB", { month: "long" });
  return (
    <section className="card desk-card" aria-labelledby="commits">
      <div className="section-head">
        <h2 id="commits">16 weeks of commits</h2>
        {cards.streak > 1 && <span className="muted small">{cards.streak} day streak</span>}
      </div>
      <div className="heat" role="img" aria-label={`${total} contributions in 16 weeks${cards.streak > 1 ? `, ${cards.streak} days in a row` : ""}`}>
        {weeks.map((week, w) => (
          <div key={w} className="heat-week">
            {week.map((day, d) =>
              day ? (
                <span key={d} className={`heat-cell l${level(day[1])}`} title={`${day[0]}: ${day[1]} ${day[1] === 1 ? "contribution" : "contributions"}`} />
              ) : (
                <span key={d} className="heat-cell blank" />
              ),
            )}
          </div>
        ))}
      </div>
      <div className="heat-foot muted small">
        <span>{month(days[0][0])}</span>
        <span>{month(days[days.length - 1][0])}</span>
      </div>
    </section>
  );
}

/** Pull requests merged each week for eight weeks; this week stands out. */
export function MergedWeeks({ cards }: { cards: CodeCards }) {
  const weeks = cards.merged_weeks;
  const [table, setTable] = useState(false);
  if (weeks.length === 0) return null;
  const top = Math.max(1, ...weeks);
  const label = (back: number) => (back === 0 ? "This week" : back === 1 ? "Last week" : `${back} weeks ago`);
  return (
    <section className="card desk-card" aria-labelledby="merged">
      <div className="section-head">
        <h2 id="merged">Merged per week</h2>
        <button type="button" className="link-button small" aria-pressed={table} onClick={() => setTable(!table)}>
          {table ? "Chart" : "Table"}
        </button>
      </div>
      {table ? (
        <table className="numbers">
          <tbody>
            {weeks.map((n, i) => (
              <tr key={i}>
                <th scope="row">{label(weeks.length - 1 - i)}</th>
                <td>{n}</td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <div className="merged-bars" role="img" aria-label={`Merged per week, oldest first: ${weeks.join(", ")}`}>
          {weeks.map((n, i) => (
            <div key={i} className="merged-col" title={`${label(weeks.length - 1 - i)}: ${n} merged`}>
              <span className={`merged-bar ${i === weeks.length - 1 ? "now" : ""}`} style={{ height: `${Math.max(4, (n / top) * 100)}%` }} />
              <span className="merged-label">{i === weeks.length - 1 ? "Now" : `−${weeks.length - 1 - i}`}</span>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

const checkWords: Record<string, { label: string; tone: string }> = {
  passing: { label: "Passing", tone: "sage" },
  failing: { label: "Failing", tone: "clay" },
  flaky: { label: "Flaky", tone: "honey" },
};

export function Repos({ repos }: { repos: Repo[] }) {
  const notify = useNotify();
  if (repos.length === 0) return null;
  return (
    <section aria-labelledby="repos">
      <div className="section-head">
        <h2 id="repos">Your repos</h2>
        <span className="muted small">Most active first</span>
      </div>
      <ul className="card list">
        {repos.map((repo) => (
          <li key={repo.full_name} className="work-row">
            <div className="work-main">
              <button type="button" className="link-quiet work-title" onClick={() => commands.openOnGithub(repo.url).catch((e) => notify(errorMessage(e)))}>
                {repo.full_name.split("/").pop()}
              </button>
              <span className="muted small">
                {[repo.language, repo.private ? "private" : null, `★ ${repo.stars}`, `${repo.open_prs} ${repo.open_prs === 1 ? "PR" : "PRs"}`, `${repo.open_issues} ${repo.open_issues === 1 ? "issue" : "issues"}`]
                  .filter(Boolean)
                  .join(" · ")}
              </span>
            </div>
            {repo.checks && <span className={`tag ${checkWords[repo.checks].tone}`}>{checkWords[repo.checks].label}</span>}
          </li>
        ))}
      </ul>
    </section>
  );
}
