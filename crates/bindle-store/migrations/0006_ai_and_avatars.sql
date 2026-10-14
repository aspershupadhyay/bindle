-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0006: a picture and pronouns for each person, and a log of what
-- a connected AI read or suggested (handbook P8), shown in Settings.

ALTER TABLE person ADD COLUMN avatar TEXT;
ALTER TABLE person ADD COLUMN pronouns TEXT;

CREATE TABLE ai_log (
  id     TEXT PRIMARY KEY,
  at     TEXT NOT NULL,
  client TEXT NOT NULL,
  tool   TEXT NOT NULL,
  detail TEXT NOT NULL DEFAULT ''
);
CREATE INDEX ai_log_at ON ai_log(at);
