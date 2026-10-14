// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

import { describe, expect, it } from "vitest";
import type { CreatorView } from "./commands";
import { compact, creatorSummary, duration, mix, weekChange } from "./creator";

const empty: CreatorView = {
  account: { channel: "UC1", found: null, synced_at: null, error: null },
  followers_this_week: null,
  views_this_week: null,
  growth: [],
  top_posts: [],
  views_last_30_days: 0,
  long_posts: 0,
  short_posts: 0,
  comments: [],
};
const comment = (author: string, promised = false) => ({
  id: author,
  post_title: "A video",
  author,
  text: "Hi",
  url: "https://www.youtube.com/watch?v=a",
  likes: 0,
  published_at: "2026-10-14T00:00:00Z",
  promise: promised ? { id: "p", state: "unsorted" as const } : null,
});

describe("compact", () => {
  it("writes counts the way YouTube does, never rounding up", () => {
    expect(compact(642)).toBe("642");
    expect(compact(2100)).toBe("2.1K");
    expect(compact(2999)).toBe("2.9K");
    expect(compact(12_400)).toBe("12K");
    expect(compact(1_000)).toBe("1K");
    expect(compact(1_450_000)).toBe("1.4M");
  });
});

describe("duration", () => {
  it("reads like a video player", () => {
    expect(duration(58)).toBe("0:58");
    expect(duration(1084)).toBe("18:04");
    expect(duration(3720)).toBe("1:02:00");
  });
});

describe("weekChange", () => {
  it("says nothing until there is a week of history", () => {
    expect(weekChange(null, "subscribers")).toBeNull();
    expect(weekChange(28, "subscribers")).toBe("+28 this week");
    expect(weekChange(-3, "subscribers")).toBe("−3 this week");
    expect(weekChange(0, "subscribers")).toBe("No change in subscribers this week");
  });
});

describe("creatorSummary", () => {
  it("names one person waiting, counts several, and adds growth when there is some", () => {
    expect(creatorSummary(empty)).toBe("No comment is waiting on your reply.");
    expect(creatorSummary({ ...empty, comments: [comment("kunal"), comment("neha", true)] })).toBe("kunal is waiting on your reply.");
    const growth = [
      { week: "2026-09-28", followers: 500 },
      { week: "2026-10-05", followers: 600 },
    ];
    expect(creatorSummary({ ...empty, comments: [comment("a"), comment("b")], growth })).toBe("2 comments are waiting on your reply. Up 20% in subscribers over 2 weeks.");
  });
});

describe("mix", () => {
  it("adds up to 100", () => {
    expect(mix(7, 3)).toEqual({ long: 70, short: 30 });
    expect(mix(1, 2)).toEqual({ long: 33, short: 67 });
    expect(mix(0, 0)).toEqual({ long: 0, short: 0 });
  });
});
