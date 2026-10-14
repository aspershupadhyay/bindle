-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0001: the promise ledger (handbook P5).

CREATE TABLE person (
  id             TEXT PRIMARY KEY,
  display_name   TEXT NOT NULL,
  aliases        TEXT NOT NULL DEFAULT '[]',
  note           TEXT,
  hidden_from_ai INTEGER NOT NULL DEFAULT 0,
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL,
  archived_at    TEXT
);

CREATE TABLE source (
  id          TEXT PRIMARY KEY,
  kind        TEXT NOT NULL CHECK (kind IN ('typed','link','ai','import')),
  url         TEXT,
  label       TEXT,
  snippet     TEXT,
  captured_at TEXT NOT NULL
);

CREATE TABLE promise (
  id             TEXT PRIMARY KEY,
  title          TEXT NOT NULL,
  state          TEXT NOT NULL DEFAULT 'unsorted'
                 CHECK (state IN ('unsorted','open','later','done','let_go')),
  direction      TEXT CHECK (direction IN ('mine','theirs','self')),
  strength       TEXT NOT NULL DEFAULT 'firm' CHECK (strength IN ('firm','soft')),
  person_id      TEXT REFERENCES person(id),
  source_id      TEXT REFERENCES source(id),
  first_step     TEXT,
  do_on          TEXT,
  do_on_has_time INTEGER NOT NULL DEFAULT 0,
  due            TEXT,
  due_has_time   INTEGER NOT NULL DEFAULT 0,
  tz             TEXT,
  amount_minor   INTEGER,
  currency       TEXT,
  note           TEXT,
  repeat_rule    TEXT,
  series_id      TEXT,
  origin         TEXT NOT NULL DEFAULT 'user' CHECK (origin IN ('user','ai','import')),
  origin_client  TEXT,
  hidden_from_ai INTEGER NOT NULL DEFAULT 0,
  rev            INTEGER NOT NULL DEFAULT 1,
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL,
  closed_at      TEXT
);
CREATE INDEX promise_state_due   ON promise(state, due);
CREATE INDEX promise_state_do_on ON promise(state, do_on);
CREATE INDEX promise_person      ON promise(person_id, state);

CREATE TABLE promise_event (
  id         TEXT PRIMARY KEY,
  promise_id TEXT NOT NULL REFERENCES promise(id),
  at         TEXT NOT NULL,
  actor      TEXT NOT NULL DEFAULT 'user',
  kind       TEXT NOT NULL CHECK (kind IN (
               'created','sorted','edited','moved','done','reopened',
               'let_go','nudged','received','top3_added','top3_removed',
               'fresh_start','ai_suggested','ai_approved','ai_rejected')),
  data       TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX event_promise ON promise_event(promise_id, at);

CREATE TABLE top3 (
  day        TEXT NOT NULL,
  position   INTEGER NOT NULL CHECK (position BETWEEN 1 AND 3),
  promise_id TEXT NOT NULL REFERENCES promise(id),
  PRIMARY KEY (day, position)
);

CREATE TABLE day_log (
  day           TEXT PRIMARY KEY,
  wrapped_up_at TEXT,
  mind_load     INTEGER CHECK (mind_load BETWEEN 1 AND 5),
  note          TEXT
);

CREATE TABLE setting (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE VIRTUAL TABLE search_index USING fts5(
  ref_id UNINDEXED, ref_kind UNINDEXED, title, body,
  tokenize = 'unicode61 remove_diacritics 2'
);
