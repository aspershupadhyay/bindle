-- SPDX-License-Identifier: Apache-2.0
-- Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
-- Migration 0005: remember which notifications were sent (handbook P9), so
-- none is sent twice and the daily limit holds across restarts.

CREATE TABLE notification_log (
  key     TEXT PRIMARY KEY,
  day     TEXT NOT NULL,
  sent_at TEXT NOT NULL
);
CREATE INDEX notification_log_day ON notification_log(day);
