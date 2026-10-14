-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0010: syncing Ours through Bindle Cloud (handbook P15).
--
-- sync_field remembers, for every shared field, the value and stamp last
-- agreed on. A local change is a field whose value differs from it; a
-- remote change wins only if its stamp is later. sync_outbox holds sealed-
-- to-be batches until the relay has them, so nothing is lost offline.

CREATE TABLE sync_field (
  space_id TEXT NOT NULL,
  record   TEXT NOT NULL,
  field    TEXT NOT NULL,
  value    TEXT,
  millis   INTEGER NOT NULL,
  counter  INTEGER NOT NULL,
  device   TEXT NOT NULL,
  PRIMARY KEY (space_id, record, field)
);

CREATE TABLE sync_outbox (
  id       INTEGER PRIMARY KEY AUTOINCREMENT,
  space_id TEXT NOT NULL,
  batch    TEXT NOT NULL
);

CREATE TABLE sync_cursor (
  space_id TEXT PRIMARY KEY,
  pulled   INTEGER NOT NULL DEFAULT 0,
  synced_at TEXT
);

ALTER TABLE space ADD COLUMN relay TEXT;
