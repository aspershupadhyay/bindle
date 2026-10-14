// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// Sample GitHub work for the browser preview, from the v2 Code design. None
// of it is real, and the preview never talks to GitHub.

import type { CodeView, GithubAccount, WorkItem, WorkKind } from "./commands";

const hoursAgo = (hours: number) => new Date(Date.now() - hours * 3_600_000).toISOString();

function sample(id: string, kind: WorkKind, repo: string, number: number, title: string, author: string, hours: number, extra: Partial<WorkItem> = {}): WorkItem {
  return {
    id,
    kind,
    repo,
    number,
    title,
    author,
    url: `https://github.com/${repo}/${kind === "issue" ? "issues" : "pull"}/${number}`,
    draft: false,
    labels: [],
    created_at: hoursAgo(hours),
    updated_at: hoursAgo(hours / 2),
    gone: false,
    promise: null,
    ...extra,
  };
}

export const sampleWork: WorkItem[] = [
  sample("r1", "review", "kunal-m/latch", 42, "Permission prompts for the share sheet", "kunal-m", 50),
  sample("r2", "review", "neha-k/gymbros", 88, "Exercise animation loader", "neha-k", 6),
  sample("m1", "mine", "aspershupadhyay/codex-claude-bridge", 21, "Per-user login cache for Codex sessions", "aspershupadhyay", 26),
  sample("m2", "mine", "aspershupadhyay/bindle", 3, "Hybrid logical clock for sync", "aspershupadhyay", 3, { draft: true }),
  sample("i1", "issue", "aspershupadhyay/codex-claude-bridge", 17, "Token refresh fails on Windows", "rhea-s", 9, { labels: ["bug"] }),
  sample("i2", "issue", "kunal-m/latch", 39, "Share sheet drops the caption on Android 15", "kunal-m", 48, { labels: ["bug"] }),
  sample("i3", "issue", "aspershupadhyay/unimitra", 12, "MCA Sem 1 notes page returns 404", "student-22", 96),
];

export function sampleCodeView(account: GithubAccount): CodeView {
  if (!account.login) return { account, reviews: [], mine: [], issues: [], finished: [], reviews_older_than_a_day: 0 };
  const of = (kind: WorkKind) => sampleWork.filter((i) => i.kind === kind && !i.gone);
  return {
    account,
    reviews: of("review"),
    mine: of("mine"),
    issues: of("issue"),
    finished: sampleWork.filter((i) => i.gone && i.promise),
    reviews_older_than_a_day: of("review").filter((i) => Date.now() - Date.parse(i.created_at) > 86_400_000).length,
  };
}
