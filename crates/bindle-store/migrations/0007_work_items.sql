-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0007: things that need you at work, read from a source such as
-- GitHub (handbook P14). Each row is what the source said at the last look;
-- "Make it a promise" links one to a promise, which waits in Sort.

CREATE TABLE work_item (
  id         TEXT PRIMARY KEY,
  source     TEXT NOT NULL,
  kind       TEXT NOT NULL CHECK (kind IN ('review','mine','issue')),
  repo       TEXT NOT NULL,
  number     INTEGER NOT NULL,
  title      TEXT NOT NULL,
  author     TEXT NOT NULL,
  url        TEXT NOT NULL,
  draft      INTEGER NOT NULL DEFAULT 0,
  labels     TEXT NOT NULL DEFAULT '[]',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  seen_at    TEXT NOT NULL,
  gone_at    TEXT,
  promise_id TEXT REFERENCES promise(id)
);
CREATE INDEX work_item_source ON work_item(source, kind, gone_at);
CREATE INDEX work_item_promise ON work_item(promise_id);
