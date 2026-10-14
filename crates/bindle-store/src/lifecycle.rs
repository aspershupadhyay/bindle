// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Sort, the lifecycle, history and Undo (handbook P5 and P13).
//!
//! Every change goes through [`change`]: it reads the promise's fields,
//! applies the edit, checks the lifecycle, writes, and records an event with
//! the fields before and after. Those two snapshots make Undo exact: undoing
//! puts back "before", but only if the promise still looks like "after".

use crate::{people, promises, StoreError};
use bindle_core::{check_sorted, check_transition, Direction, Promise, State, Strength, When};
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The columns a change can touch, exactly as stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Fields {
    title: String,
    state: String,
    direction: Option<String>,
    strength: String,
    person_id: Option<String>,
    first_step: Option<String>,
    do_on: Option<String>,
    do_on_has_time: bool,
    due: Option<String>,
    due_has_time: bool,
    closed_at: Option<String>,
}

const FIELDS: &str = "title, state, direction, strength, person_id, first_step, \
                      do_on, do_on_has_time, due, due_has_time, closed_at";

/// Event kinds that Undo can reverse.
const UNDOABLE: &str = "'sorted','edited','moved','done','reopened','let_go','later'";

/// A saved change, with the event that can undo it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    pub promise: Promise,
    pub event_id: String,
}

/// What the person chose in Sort.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Sorting {
    pub title: String,
    pub direction: Option<Direction>,
    pub strength: Strength,
    pub person: Option<String>,
    pub first_step: Option<String>,
    pub when: Option<When>,
    /// `when` is a deadline ("by Friday"), not a plan.
    pub is_deadline: bool,
}

/// Sorts a promise waiting in Sort: keep it (`Open`), put it off (`Later`) or
/// let it go (`LetGo`). Keeping it checks the "close the loop" rule.
pub fn sort(
    conn: &mut Connection,
    id: &str,
    sorting: &Sorting,
    to: State,
    now: DateTime<Utc>,
) -> Result<Change, StoreError> {
    let kind = match to {
        State::Open => {
            check_sorted(sorting.direction, sorting.strength, sorting.when)?;
            "sorted"
        }
        State::Later => "later",
        State::LetGo => "let_go",
        _ => {
            return Err(StoreError::NotAllowed(bindle_core::TransitionError {
                from: State::Unsorted,
                to,
            }))
        }
    };
    let title = sorting.title.trim().to_string();
    if title.is_empty() {
        return Err(StoreError::EmptyTitle);
    }
    change(conn, id, kind, now, |conn, fields, at| {
        fields.title = title;
        fields.state = to.as_str().into();
        fields.direction = sorting.direction.map(|d| d.as_str().into());
        fields.strength = sorting.strength.as_str().into();
        fields.person_id = match sorting.person.as_deref().map(str::trim) {
            Some(name) if !name.is_empty() => Some(people::find_or_add(conn, name, at)?),
            _ => None,
        };
        fields.first_step = sorting
            .first_step
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(Into::into);
        set_when(fields, sorting.when, sorting.is_deadline);
        if to == State::LetGo {
            fields.closed_at = Some(at.into());
        }
        Ok(())
    })
}

/// Marks a promise as kept.
pub fn mark_done(
    conn: &mut Connection,
    id: &str,
    now: DateTime<Utc>,
) -> Result<Change, StoreError> {
    close(conn, id, State::Done, "done", now)
}

/// Lets a promise go. It stays in history; nothing is deleted.
pub fn let_go(conn: &mut Connection, id: &str, now: DateTime<Utc>) -> Result<Change, StoreError> {
    close(conn, id, State::LetGo, "let_go", now)
}

/// Opens a done or let-go promise again.
pub fn reopen(conn: &mut Connection, id: &str, now: DateTime<Utc>) -> Result<Change, StoreError> {
    change(conn, id, "reopened", now, |_, fields, _| {
        fields.state = State::Open.as_str().into();
        fields.closed_at = None;
        Ok(())
    })
}

/// Moves a promise to a new day ("Move it"). A promise kept for later comes
/// back to open. Only open or later promises can move.
pub fn move_to(
    conn: &mut Connection,
    id: &str,
    when: When,
    is_deadline: bool,
    now: DateTime<Utc>,
) -> Result<Change, StoreError> {
    change(conn, id, "moved", now, |_, fields, _| {
        let from = stored_state(&fields.state)?;
        if !matches!(from, State::Open | State::Later) {
            return Err(StoreError::NotAllowed(bindle_core::TransitionError {
                from,
                to: State::Open,
            }));
        }
        fields.state = State::Open.as_str().into();
        set_when(fields, Some(when), is_deadline);
        Ok(())
    })
}

fn close(
    conn: &mut Connection,
    id: &str,
    to: State,
    kind: &'static str,
    now: DateTime<Utc>,
) -> Result<Change, StoreError> {
    change(conn, id, kind, now, |_, fields, at| {
        fields.state = to.as_str().into();
        fields.closed_at = Some(at.into());
        Ok(())
    })
}

fn set_when(fields: &mut Fields, when: Option<When>, is_deadline: bool) {
    let stored = when.map(promises::stored_when);
    let has_time = when.is_some_and(|w| w.time.is_some());
    (
        fields.do_on,
        fields.do_on_has_time,
        fields.due,
        fields.due_has_time,
    ) = if is_deadline {
        (None, false, stored, has_time)
    } else {
        (stored, has_time, None, false)
    };
}

/// Applies one change in a transaction and records it with before and after.
fn change(
    conn: &mut Connection,
    id: &str,
    kind: &str,
    now: DateTime<Utc>,
    edit: impl FnOnce(&Connection, &mut Fields, &str) -> Result<(), StoreError>,
) -> Result<Change, StoreError> {
    let at = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    let tx = conn.transaction()?;
    let before = read_fields(&tx, id)?;
    let mut after = before.clone();
    edit(&tx, &mut after, &at)?;
    check_transition(stored_state(&before.state)?, stored_state(&after.state)?)?;
    write_fields(&tx, id, &after, &at)?;
    let data = serde_json::json!({ "before": before, "after": after });
    let event_id = record(&tx, id, kind, &at, &data)?;
    tx.commit()?;
    Ok(Change {
        promise: promises::get(conn, id)?,
        event_id,
    })
}

/// Undoes one change. Refused when the promise changed again since, so Undo
/// never throws away something newer.
pub fn undo(
    conn: &mut Connection,
    event_id: &str,
    now: DateTime<Utc>,
) -> Result<Change, StoreError> {
    let at = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    let tx = conn.transaction()?;
    let sql = format!(
        "SELECT promise_id, data FROM promise_event e WHERE id = ?1 AND kind IN ({UNDOABLE})
           AND NOT EXISTS (SELECT 1 FROM promise_event u
                           WHERE u.kind = 'undone' AND json_extract(u.data, '$.undoes') = e.id)"
    );
    let (promise_id, data): (String, String) = tx
        .query_row(&sql, [event_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()?
        .ok_or(StoreError::NothingToUndo)?;
    #[derive(Deserialize)]
    struct Snapshots {
        before: Fields,
        after: Fields,
    }
    let snapshots: Snapshots =
        serde_json::from_str(&data).map_err(|_| StoreError::NothingToUndo)?;
    if read_fields(&tx, &promise_id)? != snapshots.after {
        return Err(StoreError::ChangedSince);
    }
    write_fields(&tx, &promise_id, &snapshots.before, &at)?;
    let data = serde_json::json!({ "undoes": event_id, "before": snapshots.after, "after": snapshots.before });
    let undo_id = record(&tx, &promise_id, "undone", &at, &data)?;
    tx.commit()?;
    Ok(Change {
        promise: promises::get(conn, &promise_id)?,
        event_id: undo_id,
    })
}

/// Undoes the most recent change that has not been undone yet, if any.
pub fn undo_last(conn: &mut Connection, now: DateTime<Utc>) -> Result<Option<Change>, StoreError> {
    let sql = format!(
        "SELECT id FROM promise_event e WHERE kind IN ({UNDOABLE})
           AND NOT EXISTS (SELECT 1 FROM promise_event u
                           WHERE u.kind = 'undone' AND json_extract(u.data, '$.undoes') = e.id)
         ORDER BY rowid DESC LIMIT 1"
    );
    let last: Option<String> = conn.query_row(&sql, [], |row| row.get(0)).optional()?;
    last.map(|id| undo(conn, &id, now)).transpose()
}

/// One line of a promise's history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryEntry {
    pub id: String,
    pub at: String,
    pub kind: String,
    pub actor: String,
    /// A later "undone" event reversed this one.
    pub undone: bool,
}

/// Everything that happened to a promise, oldest first.
pub fn history(conn: &Connection, promise_id: &str) -> Result<Vec<HistoryEntry>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, at, kind, actor,
                EXISTS (SELECT 1 FROM promise_event u
                        WHERE u.kind = 'undone' AND json_extract(u.data, '$.undoes') = e.id)
         FROM promise_event e WHERE promise_id = ?1 ORDER BY rowid",
    )?;
    let rows = stmt.query_map([promise_id], |row| {
        Ok(HistoryEntry {
            id: row.get(0)?,
            at: row.get(1)?,
            kind: row.get(2)?,
            actor: row.get(3)?,
            undone: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn read_fields(conn: &Connection, id: &str) -> Result<Fields, StoreError> {
    let sql = format!("SELECT {FIELDS} FROM promise WHERE id = ?1");
    conn.query_row(&sql, [id], |row| {
        Ok(Fields {
            title: row.get(0)?,
            state: row.get(1)?,
            direction: row.get(2)?,
            strength: row.get(3)?,
            person_id: row.get(4)?,
            first_step: row.get(5)?,
            do_on: row.get(6)?,
            do_on_has_time: row.get(7)?,
            due: row.get(8)?,
            due_has_time: row.get(9)?,
            closed_at: row.get(10)?,
        })
    })
    .optional()?
    .ok_or(StoreError::NotFound)
}

fn write_fields(conn: &Connection, id: &str, f: &Fields, at: &str) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE promise SET title = ?2, state = ?3, direction = ?4, strength = ?5, person_id = ?6,
                first_step = ?7, do_on = ?8, do_on_has_time = ?9, due = ?10, due_has_time = ?11,
                closed_at = ?12, rev = rev + 1, updated_at = ?13
         WHERE id = ?1",
        params![
            id,
            f.title,
            f.state,
            f.direction,
            f.strength,
            f.person_id,
            f.first_step,
            f.do_on,
            f.do_on_has_time,
            f.due,
            f.due_has_time,
            f.closed_at,
            at
        ],
    )?;
    Ok(())
}

fn record(
    conn: &Connection,
    promise_id: &str,
    kind: &str,
    at: &str,
    data: &serde_json::Value,
) -> Result<String, StoreError> {
    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO promise_event (id, promise_id, at, kind, data) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, promise_id, at, kind, data.to_string()],
    )?;
    Ok(id)
}

fn stored_state(word: &str) -> Result<State, StoreError> {
    State::from_word(word).ok_or(StoreError::NotFound)
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod tests;
