-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0008: email that needs you (handbook P14), and the Board's
-- "Making" lane.
--
-- Each mail_item is what Bindle found at the last look: a thread waiting on
-- your reply, a question of yours waiting on someone else, or a promise you
-- wrote in an email. "Not needed" hides one in Bindle only; nothing is ever
-- changed in the mailbox.

CREATE TABLE mail_item (
  id               TEXT PRIMARY KEY,
  source           TEXT NOT NULL,
  kind             TEXT NOT NULL CHECK (kind IN ('reply','waiting','promise')),
  thread           TEXT NOT NULL,
  subject          TEXT NOT NULL,
  person           TEXT NOT NULL,
  address          TEXT NOT NULL,
  at               TEXT NOT NULL,
  snippet          TEXT NOT NULL DEFAULT '',
  url              TEXT NOT NULL,
  promise_title    TEXT,
  promise_when     TEXT,
  promise_has_time INTEGER NOT NULL DEFAULT 0,
  promise_deadline INTEGER NOT NULL DEFAULT 0,
  seen_at          TEXT NOT NULL,
  gone_at          TEXT,
  dismissed_at     TEXT,
  promise_id       TEXT REFERENCES promise(id)
);
CREATE INDEX mail_item_source ON mail_item(source, kind, gone_at);
CREATE INDEX mail_item_promise ON mail_item(promise_id);

-- An open promise being worked on now sits in the Board's Making lane.
ALTER TABLE promise ADD COLUMN board_lane TEXT CHECK (board_lane IN ('making'));
