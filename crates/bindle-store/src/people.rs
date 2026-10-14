// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The people promises are with (handbook P5).

use crate::promises::{read_row, COLUMNS, FROM};
use crate::StoreError;
use bindle_core::Promise;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use uuid::Uuid;

/// One person on the People screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersonSummary {
    pub id: String,
    pub name: String,
    /// Open promises you made to them.
    pub you_owe: u32,
    /// Open promises they made to you.
    pub they_owe: u32,
}

/// Everything between you and one person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersonDetail {
    pub id: String,
    pub name: String,
    pub you_owe: Vec<Promise>,
    pub they_owe: Vec<Promise>,
    /// Promises with them kept since `since`, newest first, at most ten.
    pub kept_lately: Vec<Promise>,
}

/// Everyone, with open promises first (most first), then by name.
pub fn summaries(conn: &Connection) -> Result<Vec<PersonSummary>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT person.id, person.display_name,
                count(CASE WHEN p.state = 'open' AND p.direction IN ('mine', 'self') THEN 1 END),
                count(CASE WHEN p.state = 'open' AND p.direction = 'theirs' THEN 1 END)
         FROM person LEFT JOIN promise p ON p.person_id = person.id
         WHERE person.archived_at IS NULL
         GROUP BY person.id
         ORDER BY count(CASE WHEN p.state = 'open' THEN 1 END) DESC, person.display_name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PersonSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            you_owe: row.get(2)?,
            they_owe: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// What you owe one person, what they owe you, and what was kept since `since`
/// (an RFC 3339 time).
pub fn detail(conn: &Connection, id: &str, since: &str) -> Result<PersonDetail, StoreError> {
    let name: String = conn
        .query_row("SELECT display_name FROM person WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or(StoreError::NotFound)?;
    let list =
        |condition: &str, order: &str, since: Option<&str>| -> Result<Vec<Promise>, StoreError> {
            let sql = format!(
                "SELECT {COLUMNS} {FROM} WHERE p.person_id = ?1 AND {condition} ORDER BY {order}"
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = match since {
                Some(since) => stmt.query_map(params![id, since], read_row)?,
                None => stmt.query_map(params![id], read_row)?,
            };
            Ok(rows.collect::<Result<_, _>>()?)
        };
    let soonest = "coalesce(p.due, p.do_on) IS NULL, coalesce(p.due, p.do_on), p.created_at";
    Ok(PersonDetail {
        id: id.to_string(),
        name,
        you_owe: list(
            "p.state = 'open' AND p.direction IN ('mine', 'self')",
            soonest,
            None,
        )?,
        they_owe: list("p.state = 'open' AND p.direction = 'theirs'", soonest, None)?,
        kept_lately: list(
            "p.state = 'done' AND p.closed_at >= ?2",
            "p.closed_at DESC LIMIT 10",
            Some(since),
        )?,
    })
}

/// Renames a person, for example "Wifey" to "Ananya". Search follows along.
pub fn rename(conn: &Connection, id: &str, name: &str, at: &str) -> Result<(), StoreError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(StoreError::EmptyName);
    }
    let taken: Option<String> = conn
        .query_row(
            "SELECT id FROM person WHERE display_name = ?1 COLLATE NOCASE AND id != ?2 AND archived_at IS NULL",
            params![name, id],
            |r| r.get(0),
        )
        .optional()?;
    if taken.is_some() {
        return Err(StoreError::NameTaken(name.to_string()));
    }
    let changed = conn.execute(
        "UPDATE person SET display_name = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, name, at],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

/// The names of everyone in Bindle, for the parser to recognise.
/// People hidden from AI are still listed: this never leaves the device.
pub fn names(conn: &Connection) -> Result<Vec<String>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT display_name FROM person WHERE archived_at IS NULL ORDER BY display_name",
    )?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Finds a person by name, ignoring case, or adds them. Returns their id.
pub fn find_or_add(conn: &Connection, name: &str, at: &str) -> Result<String, StoreError> {
    let existing = conn
        .query_row(
            "SELECT id FROM person WHERE display_name = ?1 COLLATE NOCASE AND archived_at IS NULL",
            [name],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO person (id, display_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, name, at],
    )?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::{self, Sorting};
    use crate::open_in_memory;
    use crate::promises::{jot_down, Jot};
    use bindle_core::{Direction, State, Strength};
    use chrono::{TimeZone, Utc};

    fn kept_with(conn: &mut Connection, title: &str, person: &str, direction: Direction) -> String {
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap();
        let id = jot_down(conn, &Jot::plain(title), now).expect("jot").id;
        let sorting = Sorting {
            title: title.into(),
            direction: Some(direction),
            strength: Strength::Soft,
            person: Some(person.into()),
            first_step: None,
            when: None,
            is_deadline: false,
        };
        lifecycle::sort(conn, &id, &sorting, State::Open, now).expect("keep");
        id
    }

    #[test]
    fn people_with_open_promises_come_first_with_both_counts() {
        let mut conn = open_in_memory().expect("open");
        find_or_add(&conn, "Aarti", "x").expect("add");
        kept_with(&mut conn, "Send lesson plan", "Rohan", Direction::Mine);
        kept_with(&mut conn, "Pay invoice", "Rohan", Direction::Theirs);
        kept_with(&mut conn, "Thumbnails", "Aman", Direction::Theirs);
        let list = summaries(&conn).expect("people");
        let rows: Vec<_> = list
            .iter()
            .map(|p| (p.name.as_str(), p.you_owe, p.they_owe))
            .collect();
        assert_eq!(rows, [("Rohan", 1, 1), ("Aman", 0, 1), ("Aarti", 0, 0)]);
    }

    #[test]
    fn a_person_shows_both_sides_and_what_was_kept_lately() {
        let mut conn = open_in_memory().expect("open");
        kept_with(&mut conn, "Send lesson plan", "Rohan", Direction::Mine);
        kept_with(&mut conn, "Pay invoice", "Rohan", Direction::Theirs);
        let done = kept_with(&mut conn, "Share notes", "Rohan", Direction::Mine);
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 13, 0, 0).unwrap();
        lifecycle::mark_done(&mut conn, &done, now).expect("done");
        let id = summaries(&conn).expect("people")[0].id.clone();
        let rohan = detail(&conn, &id, "2026-10-01T00:00:00Z").expect("detail");
        assert_eq!(rohan.you_owe.len(), 1);
        assert_eq!(rohan.they_owe[0].title, "Pay invoice");
        assert_eq!(rohan.kept_lately[0].title, "Share notes");
        let later = detail(&conn, &id, "2026-10-15T00:00:00Z").expect("detail");
        assert!(later.kept_lately.is_empty());
    }

    #[test]
    fn a_rename_cannot_take_someone_elses_name() {
        let conn = open_in_memory().expect("open");
        let wifey = find_or_add(&conn, "Wifey", "x").expect("add");
        find_or_add(&conn, "Rohan", "x").expect("add");
        assert!(matches!(
            rename(&conn, &wifey, "rohan", "y"),
            Err(StoreError::NameTaken(_))
        ));
        assert!(matches!(
            rename(&conn, &wifey, "  ", "y"),
            Err(StoreError::EmptyName)
        ));
        rename(&conn, &wifey, "Ananya", "y").expect("rename");
        assert_eq!(names(&conn).expect("names"), ["Ananya", "Rohan"]);
    }

    #[test]
    fn the_same_name_in_another_case_is_the_same_person() {
        let conn = open_in_memory().expect("open");
        let first = find_or_add(&conn, "Rohan", "2026-10-14").expect("add");
        let again = find_or_add(&conn, "rohan", "2026-10-14").expect("find");
        assert_eq!(first, again);
        assert_eq!(names(&conn).expect("names"), ["Rohan"]);
    }
}
