-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0003: history can record "later" and "undone" (handbook P5, Undo).
-- SQLite cannot change a CHECK, so the table is rebuilt and every row copied.

CREATE TABLE promise_event_new (
  id         TEXT PRIMARY KEY,
  promise_id TEXT NOT NULL REFERENCES promise(id),
  at         TEXT NOT NULL,
  actor      TEXT NOT NULL DEFAULT 'user',
  kind       TEXT NOT NULL CHECK (kind IN (
               'created','sorted','edited','moved','done','reopened',
               'let_go','later','undone','nudged','received','top3_added',
               'top3_removed','fresh_start','ai_suggested','ai_approved',
               'ai_rejected')),
  data       TEXT NOT NULL DEFAULT '{}'
);

INSERT INTO promise_event_new (id, promise_id, at, actor, kind, data)
  SELECT id, promise_id, at, actor, kind, data FROM promise_event ORDER BY rowid;

DROP TABLE promise_event;
ALTER TABLE promise_event_new RENAME TO promise_event;
CREATE INDEX event_promise ON promise_event(promise_id, at);
