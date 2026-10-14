-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0011: Creator (handbook P14). What a creator source such as
-- YouTube shows: the channel's numbers each day, its recent posts, and
-- comments waiting on a reply. "Make it a promise" links a comment to a
-- promise, which waits in Sort. Nothing here is ever sent back.

CREATE TABLE creator_snapshot (
  source    TEXT NOT NULL,
  day       TEXT NOT NULL,            -- YYYY-MM-DD, UTC
  followers INTEGER,                  -- NULL when the channel hides it
  views     INTEGER NOT NULL,
  posts     INTEGER NOT NULL,
  PRIMARY KEY (source, day)
);

CREATE TABLE creator_post (
  id               TEXT PRIMARY KEY,
  source           TEXT NOT NULL,
  title            TEXT NOT NULL,
  url              TEXT NOT NULL,
  short            INTEGER NOT NULL DEFAULT 0,
  duration_seconds INTEGER NOT NULL,
  published_at     TEXT NOT NULL,
  views            INTEGER NOT NULL,
  likes            INTEGER NOT NULL,
  comments         INTEGER NOT NULL,
  seen_at          TEXT NOT NULL
);
CREATE INDEX creator_post_source ON creator_post(source, published_at);

CREATE TABLE creator_comment (
  id           TEXT PRIMARY KEY,
  source       TEXT NOT NULL,
  post_id      TEXT NOT NULL,
  post_title   TEXT NOT NULL,
  author       TEXT NOT NULL,
  text         TEXT NOT NULL,
  url          TEXT NOT NULL,
  likes        INTEGER NOT NULL DEFAULT 0,
  published_at TEXT NOT NULL,
  seen_at      TEXT NOT NULL,
  gone_at      TEXT,
  hidden_at    TEXT,
  promise_id   TEXT REFERENCES promise(id)
);
CREATE INDEX creator_comment_source ON creator_comment(source, gone_at, hidden_at);
CREATE INDEX creator_comment_promise ON creator_comment(promise_id);
