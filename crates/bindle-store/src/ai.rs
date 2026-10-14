// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The rules for a connected AI (handbook P8).
//!
//! The person picks how far an AI may go: not at all, reading, or reading and
//! suggesting. An AI never changes or closes a promise. What it suggests
//! waits in Sort like anything typed (principle 4), at most
//! [`MAX_WAITING`] at a time. Anything marked "keep from AI", or with a
//! person marked so, is never shown to it. Every read and suggestion is
//! logged, so Settings can show what the AI saw.

use crate::promises::{self, Jot, Origin};
use crate::{settings, StoreError};
use bindle_core::Promise;
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// How far a connected AI may go. Off until the person turns it on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiAccess {
    #[default]
    Off,
    /// Read promises and people, never write.
    Read,
    /// Read, and suggest new promises that wait in Sort.
    Suggest,
}

/// The most AI suggestions that may wait in Sort at once, so an AI can never
/// bury the person's own jots.
pub const MAX_WAITING: u32 = 10;

/// Fails unless the person lets an AI read.
pub fn check_read(conn: &Connection) -> Result<(), StoreError> {
    match settings::load(conn)?.ai_access {
        AiAccess::Off => Err(StoreError::AiOff),
        AiAccess::Read | AiAccess::Suggest => Ok(()),
    }
}

/// Fails unless the person lets an AI suggest.
pub fn check_suggest(conn: &Connection) -> Result<(), StoreError> {
    match settings::load(conn)?.ai_access {
        AiAccess::Off => Err(StoreError::AiOff),
        AiAccess::Read => Err(StoreError::AiReadOnly),
        AiAccess::Suggest => Ok(()),
    }
}

/// The ids of promises an AI must not see: kept from AI themselves, or with
/// a person who is.
pub fn hidden_ids(conn: &Connection) -> Result<HashSet<String>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT p.id FROM promise p LEFT JOIN person ON person.id = p.person_id
         WHERE p.hidden_from_ai = 1 OR person.hidden_from_ai = 1",
    )?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The ids of people kept from AI.
pub fn hidden_people(conn: &Connection) -> Result<HashSet<String>, StoreError> {
    let mut stmt = conn.prepare("SELECT id FROM person WHERE hidden_from_ai = 1")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Drops the promises in `hidden` from a list.
pub fn without_hidden(list: &mut Vec<Promise>, hidden: &HashSet<String>) {
    list.retain(|p| !hidden.contains(&p.id));
}

/// Saves an AI's suggestion as a new promise waiting in Sort.
pub fn suggest(
    conn: &mut Connection,
    jot: &Jot,
    client: &str,
    reason: Option<&str>,
    now: DateTime<Utc>,
) -> Result<Promise, StoreError> {
    check_suggest(conn)?;
    let waiting: u32 = conn.query_row(
        "SELECT count(*) FROM promise WHERE state = 'unsorted' AND origin = 'ai'",
        [],
        |r| r.get(0),
    )?;
    if waiting >= MAX_WAITING {
        return Err(StoreError::TooManySuggestions(waiting));
    }
    let reason = reason.map(str::trim).filter(|r| !r.is_empty());
    promises::insert(conn, jot, Origin::Ai { client, reason }, now)
}

/// Marks a person as kept from AI, or not. Their promises go with them.
pub fn set_person_hidden(conn: &Connection, id: &str, hidden: bool) -> Result<(), StoreError> {
    let changed = conn.execute(
        "UPDATE person SET hidden_from_ai = ?2 WHERE id = ?1",
        params![id, hidden],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

/// One thing an AI read or suggested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogEntry {
    pub at: String,
    pub client: String,
    pub tool: String,
    pub detail: String,
}

/// Notes that `client` used `tool`. Entries older than 90 days are dropped.
pub fn log(
    conn: &Connection,
    client: &str,
    tool: &str,
    detail: &str,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    let at = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    let cutoff = (now - chrono::Duration::days(90)).to_rfc3339_opts(SecondsFormat::Millis, true);
    conn.execute("DELETE FROM ai_log WHERE at < ?1", [cutoff])?;
    conn.execute(
        "INSERT INTO ai_log (id, at, client, tool, detail) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![Uuid::now_v7().to_string(), at, client, tool, detail],
    )?;
    Ok(())
}

/// The latest log entries, newest first.
pub fn recent(conn: &Connection, limit: usize) -> Result<Vec<LogEntry>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT at, client, tool, detail FROM ai_log ORDER BY at DESC, id DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit as i64], |row| {
        Ok(LogEntry {
            at: row.get(0)?,
            client: row.get(1)?,
            tool: row.get(2)?,
            detail: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

#[cfg(test)]
#[path = "ai_tests.rs"]
mod tests;
