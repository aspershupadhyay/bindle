// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// A sample YouTube channel for the browser preview, from the v2 Creator
// design. None of it is real, and the preview never reads YouTube.

import type { CreatorAccount, CreatorComment, CreatorPost, CreatorView } from "./commands";

const hoursAgo = (hours: number) => new Date(Date.now() - hours * 3_600_000).toISOString();

export const youtube: { account: CreatorAccount; hidden: Set<string>; promised: Map<string, string> } = {
  account: { channel: null, found: null, synced_at: null, error: null },
  hidden: new Set(),
  promised: new Map(),
};

const posts: CreatorPost[] = [
  { id: "p1", title: "I let an AI drive my Android phone", url: "https://www.youtube.com/watch?v=p1", short: false, duration_seconds: 1084, published_at: hoursAgo(80), views: 2100, likes: 61, comments: 33 },
  { id: "p2", title: "Claude and Codex, talking through MCP", url: "https://www.youtube.com/watch?v=p2", short: false, duration_seconds: 1360, published_at: hoursAgo(250), views: 1400, likes: 48, comments: 20 },
  { id: "p3", title: "Next.js and Supabase in one evening", url: "https://www.youtube.com/watch?v=p3", short: false, duration_seconds: 1872, published_at: hoursAgo(420), views: 900, likes: 27, comments: 9 },
  { id: "p4", title: "Pixel art in Python, in 60 seconds", url: "https://www.youtube.com/watch?v=p4", short: true, duration_seconds: 58, published_at: hoursAgo(130), views: 800, likes: 12, comments: 15 },
];

const comments: CreatorComment[] = [
  { id: "c1", post_title: "I let an AI drive my Android phone", author: "kunal", text: "Which model did you use for the screen reading? Would love a follow-up on the accessibility tree part.", url: "https://www.youtube.com/watch?v=p1&lc=c1", likes: 14, published_at: hoursAgo(5), promise: null },
  { id: "c2", post_title: "Claude and Codex, talking through MCP", author: "neha.builds", text: "Can you share the MCP config you used? Mine keeps timing out.", url: "https://www.youtube.com/watch?v=p2&lc=c2", likes: 6, published_at: hoursAgo(30), promise: null },
  { id: "c3", post_title: "Pixel art in Python, in 60 seconds", author: "rohanmehta", text: "Collab on a pixel art series? I do the music.", url: "https://www.youtube.com/watch?v=p4&lc=c3", likes: 2, published_at: hoursAgo(52), promise: null },
];

const growth = [520, 528, 537, 549, 556, 566, 578, 589, 598, 607, 614, 642];

function monday(weeksAgo: number): string {
  const d = new Date();
  d.setDate(d.getDate() - ((d.getDay() + 6) % 7) - weeksAgo * 7);
  return d.toISOString().slice(0, 10);
}

export function sampleCreator(): CreatorView {
  const connected = Boolean(youtube.account.channel);
  return {
    account: youtube.account,
    followers_this_week: connected ? 28 : null,
    views_this_week: connected ? 3980 : null,
    growth: connected ? growth.map((followers, i) => ({ week: monday(growth.length - 1 - i), followers })) : [],
    top_posts: connected ? [...posts].sort((a, b) => b.views - a.views) : [],
    views_last_30_days: connected ? posts.reduce((sum, p) => sum + p.views, 0) : 0,
    long_posts: connected ? 7 : 0,
    short_posts: connected ? 3 : 0,
    comments: connected
      ? comments
          .filter((c) => !youtube.hidden.has(c.id))
          .map((c) => (youtube.promised.has(c.id) ? { ...c, promise: { id: youtube.promised.get(c.id) ?? "", state: "unsorted" as const } } : c))
      : [],
  };
}

export function connectYoutube(channel: string): CreatorAccount {
  const handle = channel.trim().startsWith("@") ? channel.trim() : `@${channel.trim()}`;
  youtube.account = {
    channel: "UCpreview",
    found: { id: "UCpreview", title: "Aspersh", handle, description: "Long builds and walkthroughs. Code you can follow along with.", followers: 642, views: 51234, posts: 41 },
    synced_at: new Date().toISOString(),
    error: null,
  };
  return youtube.account;
}

export function commentById(id: string): CreatorComment | undefined {
  return comments.find((c) => c.id === id);
}
