// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import type { CodeView, WorkItem } from "./commands";
import { age, codeSummary, countWord, lookedAgo, promiseNote, shortName } from "./work";

const now = new Date("2026-10-14T12:00:00Z");

function item(id: string, author: string, created_at: string, extra: Partial<WorkItem> = {}): WorkItem {
  return {
    id,
    kind: "review",
    repo: "kunal-m/latch",
    number: 42,
    title: "Permission prompts",
    author,
    url: "https://github.com/kunal-m/latch/pull/42",
    draft: false,
    labels: [],
    created_at,
    updated_at: created_at,
    gone: false,
    promise: null,
    ...extra,
  };
}

function view(patch: Partial<CodeView>): CodeView {
  return {
    account: { login: "me", synced_at: null, error: null },
    reviews: [],
    mine: [],
    issues: [],
    finished: [],
    reviews_older_than_a_day: 0,
    ...patch,
  };
}

describe("age", () => {
  it("is short and plain", () => {
    expect(age("2026-10-14T11:59:40Z", now)).toBe("just now");
    expect(age("2026-10-14T11:20:00Z", now)).toBe("40 min");
    expect(age("2026-10-14T06:00:00Z", now)).toBe("6 h");
    expect(age("2026-10-13T11:00:00Z", now)).toBe("1 day");
    expect(age("2026-10-12T10:00:00Z", now)).toBe("2 days");
    expect(age("2026-09-14T12:00:00Z", now)).toBe("4 weeks");
    expect(age("2026-10-15T12:00:00Z", now)).toBe("just now");
  });
});

describe("lookedAgo", () => {
  it("never says just now ago", () => {
    expect(lookedAgo(null, now)).toBe("Not looked yet");
    expect(lookedAgo("2026-10-14T11:59:50Z", now)).toBe("Looked just now");
    expect(lookedAgo("2026-10-14T11:55:00Z", now)).toBe("Looked 5 min ago");
  });
});

describe("codeSummary", () => {
  it("names who waits and where to start", () => {
    const summary = codeSummary(
      view({
        reviews: [item("a", "kunal-m", "2026-10-12T10:00:00Z"), item("b", "neha-k", "2026-10-14T06:00:00Z")],
        reviews_older_than_a_day: 1,
      }),
      now,
    );
    expect(summary).toBe("Two people are waiting on your reviews. Start with latch #42, asked 2 days ago.");
  });

  it("speaks of one person by name", () => {
    expect(codeSummary(view({ reviews: [item("a", "kunal-m", "2026-10-14T10:00:00Z")] }), now)).toBe(
      "kunal-m is waiting on your review. Nothing else needs you on GitHub.",
    );
  });

  it("falls back to issues and pull requests, kindly", () => {
    expect(codeSummary(view({ issues: [item("i", "x", "2026-10-14T10:00:00Z", { kind: "issue" })] }), now)).toBe(
      "Nobody is waiting on a review from you. One issue is assigned to you.",
    );
    expect(codeSummary(view({ mine: [item("m", "me", "2026-10-14T10:00:00Z"), item("n", "me", "2026-10-14T10:00:00Z")] }), now)).toBe(
      "Nobody is waiting on a review from you. Two of your pull requests are waiting on others.",
    );
  });
});

describe("small words", () => {
  it("counts in words up to ten", () => {
    expect(countWord(3)).toBe("Three");
    expect(countWord(12)).toBe("12");
  });

  it("shortens a repository", () => {
    expect(shortName({ repo: "kunal-m/latch", number: 42 })).toBe("latch #42");
  });

  it("says where a linked promise stands", () => {
    const base = item("a", "k", "2026-10-14T10:00:00Z");
    expect(promiseNote(base)).toBeNull();
    expect(promiseNote({ ...base, promise: { id: "p", state: "unsorted" } })).toBe("Waiting in Sort");
    expect(promiseNote({ ...base, promise: { id: "p", state: "open" } })).toBe("A promise");
    expect(promiseNote({ ...base, promise: { id: "p", state: "let_go" } })).toBeNull();
  });
});
