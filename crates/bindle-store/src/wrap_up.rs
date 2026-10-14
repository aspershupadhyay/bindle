// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Wrap up (handbook P6 and P13): about one minute in the evening to close the
//! day. See what was kept, decide about what is still open, choose tomorrow's
//! top 3, and say how full your head feels. Plus the records notifications need.

use crate::promises::{read_row, COLUMNS, FROM};
use crate::StoreError;
use bindle_core::notify::TimedPromise;
use bindle_core::{Direction, Promise};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WrapUpView {
    pub day: NaiveDate,
    /// Done today, in the order they were done.
    pub kept_today: Vec<Promise>,
    /// Your open promises for today or earlier, oldest day first.
    pub still_open: Vec<Promise>,
    /// Your open promises for tomorrow, to pick tomorrow's top 3 from.
    pub tomorrow: Vec<Promise>,
    /// Ids already in tomorrow's top 3.
    pub tomorrow_top3: Vec<String>,
    pub wrapped_up_at: Option<String>,
    pub mind_load: Option<u8>,
}

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn day_text(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

/// Reads Wrap up for `day`, which runs from `starts` to `ends` (UTC).
pub fn view(
    conn: &Connection,
    day: NaiveDate,
    starts: DateTime<Utc>,
    ends: DateTime<Utc>,
) -> Result<WrapUpView, StoreError> {
    let tomorrow = day.succ_opt().ok_or(StoreError::NotFound)?;
    let list = |condition: &str,
                order: &str,
                values: &[&dyn rusqlite::ToSql]|
     -> Result<Vec<Promise>, StoreError> {
        let sql = format!("SELECT {COLUMNS} {FROM} WHERE {condition} ORDER BY {order}");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(values, read_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    };
    let day_of = "substr(coalesce(p.due, p.do_on), 1, 10)";
    let yours = "(p.direction IS NULL OR p.direction != 'theirs')";
    let (log_at, mind): (Option<String>, Option<u8>) = conn
        .query_row(
            "SELECT wrapped_up_at, mind_load FROM day_log WHERE day = ?1",
            [day_text(day)],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((None, None));
    let mut top3 = conn.prepare("SELECT promise_id FROM top3 WHERE day = ?1 ORDER BY position")?;
    let tomorrow_top3 = top3
        .query_map([day_text(tomorrow)], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(WrapUpView {
        day,
        kept_today: list(
            "p.state = 'done' AND p.closed_at >= ?1 AND p.closed_at < ?2",
            "p.closed_at",
            &[&stamp(starts), &stamp(ends)],
        )?,
        still_open: list(
            &format!("p.state = 'open' AND {yours} AND {day_of} <= ?1"),
            &format!("{day_of}, p.created_at"),
            &[&day_text(day)],
        )?,
        tomorrow: list(
            &format!("p.state = 'open' AND {yours} AND {day_of} = ?1"),
            "p.created_at",
            &[&day_text(tomorrow)],
        )?,
        tomorrow_top3,
        wrapped_up_at: log_at,
        mind_load: mind,
    })
}

/// Marks the day as wrapped up. How full the head felt (1 to 5) stays private:
/// it is never shown on Home as a score.
pub fn finish(
    conn: &Connection,
    day: NaiveDate,
    mind_load: Option<u8>,
    note: Option<&str>,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    let mind_load = mind_load.filter(|m| (1..=5).contains(m));
    let note = note.map(str::trim).filter(|n| !n.is_empty());
    conn.execute(
        "INSERT INTO day_log (day, wrapped_up_at, mind_load, note) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(day) DO UPDATE SET wrapped_up_at = excluded.wrapped_up_at,
                                        mind_load = excluded.mind_load, note = excluded.note",
        params![day_text(day), stamp(now), mind_load, note],
    )?;
    Ok(())
}

/// True once the day has been wrapped up.
pub fn is_wrapped_up(conn: &Connection, day: NaiveDate) -> Result<bool, StoreError> {
    Ok(conn
        .query_row(
            "SELECT wrapped_up_at IS NOT NULL FROM day_log WHERE day = ?1",
            [day_text(day)],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(false))
}

/// Your promises with a time on `day`, for reminders: open ones, and ones
/// still in Sort, since a time typed in Jot down is a time the person expects.
pub fn timed_on(conn: &Connection, day: NaiveDate) -> Result<Vec<TimedPromise>, StoreError> {
    let sql = format!(
        "SELECT {COLUMNS} {FROM} WHERE p.state IN ('open', 'unsorted')
           AND ((p.do_on_has_time = 1 AND substr(p.do_on, 1, 10) = ?1)
             OR (p.due_has_time = 1 AND substr(p.due, 1, 10) = ?1))"
    );
    let mut stmt = conn.prepare(&sql)?;
    let promises: Vec<Promise> = stmt
        .query_map([day_text(day)], read_row)?
        .collect::<Result<_, _>>()?;
    Ok(promises
        .into_iter()
        .filter(|p| p.direction != Some(Direction::Theirs))
        .filter_map(|p| {
            let when = p.due.or(p.do_on)?;
            Some(TimedPromise {
                id: p.id,
                title: p.title,
                at: when.day.and_time(when.time?),
            })
        })
        .collect())
}

/// Keys of notifications already sent on `day`.
pub fn sent_on(conn: &Connection, day: NaiveDate) -> Result<Vec<String>, StoreError> {
    let mut stmt = conn.prepare("SELECT key FROM notification_log WHERE day = ?1")?;
    let keys = stmt
        .query_map([day_text(day)], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(keys)
}

/// Records a sent notification. Recording the same key twice is harmless.
pub fn record_sent(
    conn: &Connection,
    key: &str,
    day: NaiveDate,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR IGNORE INTO notification_log (key, day, sent_at) VALUES (?1, ?2, ?3)",
        params![key, day_text(day), stamp(now)],
    )?;
    Ok(())
}

#[cfg(test)]
#[path = "wrap_up_tests.rs"]
mod tests;
