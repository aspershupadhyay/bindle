// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Reading and writing promises (handbook P5).
//!
//! Every write also records a `promise_event`, so history is never lost.
//! Callers pass "now" in, which keeps every function easy to test.

use crate::{people, StoreError};
use bindle_core::{Direction, Mode, Money, Promise, State, Strength, When};
use chrono::{DateTime, NaiveDate, NaiveTime, SecondsFormat, Utc};
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

pub(crate) const COLUMNS: &str = "p.id, p.title, p.state, p.direction, p.strength, p.mode, p.first_step, \
                       p.do_on, p.do_on_has_time, p.due, p.due_has_time, \
                       p.amount_minor, p.currency, person.display_name, p.created_at, p.updated_at, \
                       p.closed_at";
pub(crate) const FROM: &str = "FROM promise p LEFT JOIN person ON person.id = p.person_id";

/// What Jot down saves. Everything except the text is a suggestion read from
/// it; the promise still waits in Sort for the person to confirm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jot {
    /// Exactly what was typed. Kept as the promise's source, so nothing is lost.
    pub text: String,
    pub title: String,
    pub when: Option<When>,
    /// `when` is a deadline ("by Friday"), not a plan.
    pub is_deadline: bool,
    pub amount: Option<Money>,
    pub person: Option<String>,
    pub direction: Option<Direction>,
    pub strength: Strength,
    /// The person's time zone (IANA name), so times keep their meaning.
    pub tz: Option<String>,
    /// The words that could mean another day ("kal"), when there were some.
    /// Kept in the `created` event so Sort can ask about them.
    pub unsure_words: Option<String>,
}

impl Jot {
    /// A jot where nothing was understood: the title is the text.
    pub fn plain(text: &str) -> Self {
        Self {
            text: text.to_string(),
            title: text.trim().to_string(),
            when: None,
            is_deadline: false,
            amount: None,
            person: None,
            direction: None,
            strength: Strength::Firm,
            tz: None,
            unsure_words: None,
        }
    }
}

/// Saves a jot as a new promise waiting in Sort, with its source and a
/// `created` event, all in one transaction.
///
/// An empty title is refused, so "Saved" never shows for nothing
/// (handbook principle 2).
pub fn jot_down(
    conn: &mut Connection,
    jot: &Jot,
    now: DateTime<Utc>,
) -> Result<Promise, StoreError> {
    let title = jot.title.trim();
    if title.is_empty() {
        return Err(StoreError::EmptyTitle);
    }
    let id = Uuid::now_v7().to_string();
    let source_id = Uuid::now_v7().to_string();
    let at = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    let (do_on, due) = if jot.is_deadline {
        (None, jot.when)
    } else {
        (jot.when, None)
    };

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO source (id, kind, snippet, captured_at) VALUES (?1, 'typed', ?2, ?3)",
        params![source_id, jot.text, at],
    )?;
    let person_id = match &jot.person {
        Some(name) => Some(people::find_or_add(&tx, name, &at)?),
        None => None,
    };
    tx.execute(
        "INSERT INTO promise (id, title, direction, strength, person_id, source_id,
                              do_on, do_on_has_time, due, due_has_time, tz,
                              amount_minor, currency, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14)",
        params![
            id,
            title,
            jot.direction.map(Direction::as_str),
            jot.strength.as_str(),
            person_id,
            source_id,
            do_on.map(stored_when),
            do_on.is_some_and(|w| w.time.is_some()),
            due.map(stored_when),
            due.is_some_and(|w| w.time.is_some()),
            jot.tz,
            jot.amount.as_ref().map(|m| m.minor),
            jot.amount.as_ref().map(|m| m.currency.as_str()),
            at,
        ],
    )?;
    let data = serde_json::json!({ "unsure_words": jot.unsure_words }).to_string();
    tx.execute(
        "INSERT INTO promise_event (id, promise_id, at, kind, data) VALUES (?1, ?2, ?3, 'created', ?4)",
        params![Uuid::now_v7().to_string(), id, at, data],
    )?;
    tx.commit()?;
    get(conn, &id)
}

/// Reads one promise by id.
pub fn get(conn: &Connection, id: &str) -> Result<Promise, StoreError> {
    let sql = format!("SELECT {COLUMNS} {FROM} WHERE p.id = ?1");
    Ok(conn.query_row(&sql, [id], read_row)?)
}

/// Every promise in `state`, oldest first.
pub fn list_by_state(conn: &Connection, state: State) -> Result<Vec<Promise>, StoreError> {
    let sql = format!("SELECT {COLUMNS} {FROM} WHERE p.state = ?1 ORDER BY p.created_at, p.id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([state.as_str()], read_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// A promise waiting in Sort, with what was typed and any words to check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SortItem {
    pub promise: Promise,
    /// Exactly what was typed in Jot down.
    pub typed: Option<String>,
    /// Words that could mean two things, such as "kal", to ask about.
    pub unsure_words: Option<String>,
}

/// Everything waiting in Sort, oldest first.
pub fn sort_queue(conn: &Connection) -> Result<Vec<SortItem>, StoreError> {
    let sql = format!(
        "SELECT {COLUMNS}, source.snippet,
                (SELECT json_extract(e.data, '$.unsure_words') FROM promise_event e
                 WHERE e.promise_id = p.id AND e.kind = 'created')
         {FROM} LEFT JOIN source ON source.id = p.source_id
         WHERE p.state = 'unsorted' ORDER BY p.created_at, p.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(SortItem {
            promise: read_row(row)?,
            typed: row.get(17)?,
            unsure_words: row.get(18)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// How many promises are in each state, for the tabs on the Promises screen.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct StateCounts {
    pub unsorted: u32,
    pub open: u32,
    pub later: u32,
    pub done: u32,
    pub let_go: u32,
}

/// Counts promises in every state in one pass.
pub fn state_counts(conn: &Connection) -> Result<StateCounts, StoreError> {
    let mut counts = StateCounts::default();
    let mut stmt = conn.prepare("SELECT state, COUNT(*) FROM promise GROUP BY state")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
    })?;
    for row in rows {
        let (state, n) = row?;
        match State::from_word(&state) {
            Some(State::Unsorted) => counts.unsorted = n,
            Some(State::Open) => counts.open = n,
            Some(State::Later) => counts.later = n,
            Some(State::Done) => counts.done = n,
            Some(State::LetGo) => counts.let_go = n,
            None => {}
        }
    }
    Ok(counts)
}

/// One state's promises for the Promises screen, at most `limit`: open ones
/// soonest first (no day last), done and let go newest first, others oldest first.
pub fn list_for_screen(
    conn: &Connection,
    state: State,
    limit: usize,
) -> Result<Vec<Promise>, StoreError> {
    let order = match state {
        State::Open => "coalesce(p.due, p.do_on) IS NULL, coalesce(p.due, p.do_on), p.created_at",
        State::Done | State::LetGo => "p.closed_at DESC",
        State::Unsorted | State::Later => "p.created_at",
    };
    let sql = format!("SELECT {COLUMNS} {FROM} WHERE p.state = ?1 ORDER BY {order}, p.id LIMIT ?2");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params![state.as_str(), limit as i64], read_row)?;
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

/// A day as stored: `2026-10-14`, or `2026-10-14T20:30` with a time.
pub(crate) fn stored_when(when: When) -> String {
    match when.time {
        Some(time) => format!("{}T{}", when.day.format("%Y-%m-%d"), time.format("%H:%M")),
        None => when.day.format("%Y-%m-%d").to_string(),
    }
}

pub(crate) fn read_row(row: &Row) -> rusqlite::Result<Promise> {
    let amount = match (
        row.get::<_, Option<i64>>(11)?,
        row.get::<_, Option<String>>(12)?,
    ) {
        (Some(minor), Some(currency)) => Some(Money { minor, currency }),
        _ => None,
    };
    Ok(Promise {
        id: row.get(0)?,
        title: row.get(1)?,
        state: word(row, 2, State::from_word)?,
        direction: optional_word(row, 3, Direction::from_word)?,
        strength: word(row, 4, Strength::from_word)?,
        mode: word(row, 5, Mode::from_word)?,
        first_step: row.get(6)?,
        do_on: when(row, 7, 8)?,
        due: when(row, 9, 10)?,
        amount,
        person: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
        closed_at: row.get(16)?,
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

/// Reads a stored day, and its time when the `has_time` column says so.
fn when(row: &Row, index: usize, has_time_index: usize) -> rusqlite::Result<Option<When>> {
    let Some(text) = row.get::<_, Option<String>>(index)? else {
        return Ok(None);
    };
    let has_time: bool = row.get(has_time_index)?;
    let day = text
        .get(..10)
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| bad_value(index, text.clone()))?;
    let time = match (has_time, text.get(11..16)) {
        (true, Some(t)) => Some(
            NaiveTime::parse_from_str(t, "%H:%M").map_err(|_| bad_value(index, text.clone()))?,
        ),
        _ => None,
    };
    Ok(Some(When { day, time }))
}

fn bad_value(index: usize, text: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        rusqlite::types::Type::Text,
        format!("unexpected stored value {text:?}").into(),
    )
}

#[cfg(test)]
#[path = "promises_tests.rs"]
mod tests;
