-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0002: Mine and Ours spaces, and the sync log (handbook P15).

CREATE TABLE space (
  id         TEXT PRIMARY KEY,
  kind       TEXT NOT NULL CHECK (kind IN ('mine','ours')),
  name       TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE member (
  space_id     TEXT NOT NULL REFERENCES space(id),
  account_id   TEXT NOT NULL,
  display_name TEXT NOT NULL,
  avatar       TEXT,
  role         TEXT NOT NULL CHECK (role IN ('owner','member')),
  joined_at    TEXT NOT NULL,
  PRIMARY KEY (space_id, account_id)
);

ALTER TABLE promise ADD COLUMN space_id TEXT REFERENCES space(id);
ALTER TABLE promise ADD COLUMN created_by TEXT;
ALTER TABLE promise ADD COLUMN done_by TEXT;
ALTER TABLE promise ADD COLUMN mode TEXT NOT NULL DEFAULT 'life' CHECK (mode IN ('life','work'));

CREATE TABLE sync_op (
  id         TEXT PRIMARY KEY,
  space_id   TEXT NOT NULL,
  hlc        TEXT NOT NULL,
  device_id  TEXT NOT NULL,
  account_id TEXT NOT NULL,
  record     TEXT NOT NULL,
  field      TEXT NOT NULL,
  value      TEXT,
  pushed_at  TEXT
);
CREATE INDEX sync_op_pending ON sync_op(pushed_at) WHERE pushed_at IS NULL;
