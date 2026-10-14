// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Reading and writing promises (handbook P5).
//!
//! Every write also records a `promise_event`, so history is never lost.
//! Callers pass "now" in, which keeps every function easy to test.

use crate::StoreError;
use bindle_core::{Direction, Mode, Promise, State, Strength};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

const COLUMNS: &str = "id, title, state, direction, strength, mode, first_step, \
                       do_on, due, created_at, updated_at";

/// Saves what the person typed as a new promise waiting in Sort.
///
/// The title is trimmed. An empty title is refused, so "Saved" never shows
/// for nothing (handbook principle 2).
pub fn jot_down(
    conn: &mut Connection,
    title: &str,
    now: DateTime<Utc>,
) -> Result<Promise, StoreError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(StoreError::EmptyTitle);
    }
    let id = Uuid::now_v7().to_string();
    let at = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO promise (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, title, at],
    )?;
    tx.execute(
        "INSERT INTO promise_event (id, promise_id, at, kind) VALUES (?1, ?2, ?3, 'created')",
        params![Uuid::now_v7().to_string(), id, at],
    )?;
    tx.commit()?;
    get(conn, &id)
}

/// Reads one promise by id.
pub fn get(conn: &Connection, id: &str) -> Result<Promise, StoreError> {
    let sql = format!("SELECT {COLUMNS} FROM promise WHERE id = ?1");
    Ok(conn.query_row(&sql, [id], read_row)?)
}

/// Every promise in `state`, oldest first.
pub fn list_by_state(conn: &Connection, state: State) -> Result<Vec<Promise>, StoreError> {
    let sql = format!("SELECT {COLUMNS} FROM promise WHERE state = ?1 ORDER BY created_at, id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([state.as_str()], read_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// How many promises are in `state`. Used for badges such as "4 new" on Sort.
pub fn count_by_state(conn: &Connection, state: State) -> Result<u32, StoreError> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM promise WHERE state = ?1",
        [state.as_str()],
        |row| row.get(0),
    )?)
}

fn read_row(row: &Row) -> rusqlite::Result<Promise> {
    Ok(Promise {
        id: row.get(0)?,
        title: row.get(1)?,
        state: word(row, 2, State::from_word)?,
        direction: optional_word(row, 3, Direction::from_word)?,
        strength: word(row, 4, Strength::from_word)?,
        mode: word(row, 5, Mode::from_word)?,
        first_step: row.get(6)?,
        do_on: day(row, 7)?,
        due: day(row, 8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

/// Reads a stored word into an enum. The schema's CHECKs make a bad word
/// unlikely, but if one slips in it is reported, never guessed.
fn word<T>(row: &Row, index: usize, parse: fn(&str) -> Option<T>) -> rusqlite::Result<T> {
    let text: String = row.get(index)?;
    parse(&text).ok_or_else(|| bad_value(index, text))
}

fn optional_word<T>(
    row: &Row,
    index: usize,
    parse: fn(&str) -> Option<T>,
) -> rusqlite::Result<Option<T>> {
    let text: Option<String> = row.get(index)?;
    text.map(|t| parse(&t).ok_or_else(|| bad_value(index, t)))
        .transpose()
}

/// Reads the date part of a stored day or day-and-time.
fn day(row: &Row, index: usize) -> rusqlite::Result<Option<NaiveDate>> {
    let text: Option<String> = row.get(index)?;
    text.map(|t| {
        let date_part = t.get(..10).unwrap_or(&t);
        NaiveDate::parse_from_str(date_part, "%Y-%m-%d").map_err(|_| bad_value(index, t))
    })
    .transpose()
}

fn bad_value(index: usize, text: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        rusqlite::types::Type::Text,
        format!("unexpected stored value {text:?}").into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 14, 18, 30, 0).unwrap()
    }

    #[test]
    fn a_jotted_down_promise_waits_in_sort() {
        let mut conn = open_in_memory().expect("open");
        let promise = jot_down(&mut conn, "  Pay electricity bill ", now()).expect("save");
        assert_eq!(promise.title, "Pay electricity bill");
        assert_eq!(promise.state, State::Unsorted);
        assert_eq!(promise.mode, Mode::Life);
        assert_eq!(count_by_state(&conn, State::Unsorted).expect("count"), 1);
    }

    #[test]
    fn jotting_down_records_a_created_event() {
        let mut conn = open_in_memory().expect("open");
        let promise = jot_down(&mut conn, "Call mom", now()).expect("save");
        let kind: String = conn
            .query_row(
                "SELECT kind FROM promise_event WHERE promise_id = ?1",
                [&promise.id],
                |row| row.get(0),
            )
            .expect("event");
        assert_eq!(kind, "created");
    }

    #[test]
    fn an_empty_jot_is_refused_and_nothing_is_saved() {
        let mut conn = open_in_memory().expect("open");
        assert!(matches!(
            jot_down(&mut conn, "   ", now()),
            Err(StoreError::EmptyTitle)
        ));
        assert_eq!(count_by_state(&conn, State::Unsorted).expect("count"), 0);
    }

    #[test]
    fn promises_are_listed_oldest_first() {
        let mut conn = open_in_memory().expect("open");
        jot_down(&mut conn, "First", now()).expect("save");
        jot_down(&mut conn, "Second", now() + chrono::Duration::minutes(1)).expect("save");
        let titles: Vec<_> = list_by_state(&conn, State::Unsorted)
            .expect("list")
            .into_iter()
            .map(|p| p.title)
            .collect();
        assert_eq!(titles, ["First", "Second"]);
    }

    #[test]
    fn stored_days_and_words_read_back_as_core_types() {
        let mut conn = open_in_memory().expect("open");
        let promise = jot_down(&mut conn, "Book Diwali tickets", now()).expect("save");
        conn.execute(
            "UPDATE promise SET state = 'open', direction = 'self', due = '2026-10-17T09:00' WHERE id = ?1",
            [&promise.id],
        )
        .expect("update");
        let read = get(&conn, &promise.id).expect("read");
        assert_eq!(read.direction, Some(Direction::SelfPromise));
        assert_eq!(read.due, NaiveDate::from_ymd_opt(2026, 10, 17));
    }
}
