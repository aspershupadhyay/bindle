-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0009: how long people wait on you (handbook P14, Insights).
--
-- One row per answer: someone asked (an email to you, a review requested
-- by opening a pull request) and you answered. Rows come from the looks at
-- Gmail and GitHub and are never changed, so the history grows week by week.

CREATE TABLE wait_sample (
  id          TEXT PRIMARY KEY,
  kind        TEXT NOT NULL CHECK (kind IN ('email_reply','review')),
  person      TEXT NOT NULL DEFAULT '',
  asked_at    TEXT NOT NULL,
  answered_at TEXT NOT NULL,
  hours       REAL NOT NULL CHECK (hours >= 0)
);
CREATE INDEX wait_sample_kind ON wait_sample(kind, answered_at);
