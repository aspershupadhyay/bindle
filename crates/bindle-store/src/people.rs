// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The people promises are with (handbook P5).

use crate::StoreError;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

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
    use crate::open_in_memory;

    #[test]
    fn the_same_name_in_another_case_is_the_same_person() {
        let conn = open_in_memory().expect("open");
        let first = find_or_add(&conn, "Rohan", "2026-10-14").expect("add");
        let again = find_or_add(&conn, "rohan", "2026-10-14").expect("find");
        assert_eq!(first, again);
        assert_eq!(names(&conn).expect("names"), ["Rohan"]);
    }
}
