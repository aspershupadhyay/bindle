// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Opens Bindle's SQLite database and keeps its schema up to date.
//!
//! Each migration runs once, inside a transaction, and bumps SQLite's
//! `user_version`. A failed migration rolls back and leaves the file untouched
//! (handbook P10, Data checks).

pub mod ai;
pub mod backup;
pub mod board;
pub mod cloud;
pub mod creator;
pub mod export;
pub mod home;
pub mod insights;
pub mod lifecycle;
pub mod mail;
pub mod people;
pub mod promises;
pub mod search;
pub mod settings;
pub mod work;
pub mod wrap_up;

use rusqlite::Connection;
use std::path::Path;
use thiserror::Error;

/// Every migration, in order. The position in this list is its version number.
const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("../migrations/0001_init.sql")),
    (
        "0002_spaces_and_sync",
        include_str!("../migrations/0002_spaces_and_sync.sql"),
    ),
    (
        "0003_later_and_undo",
        include_str!("../migrations/0003_later_and_undo.sql"),
    ),
    ("0004_search", include_str!("../migrations/0004_search.sql")),
    (
        "0005_notifications",
        include_str!("../migrations/0005_notifications.sql"),
    ),
    (
        "0006_ai_and_avatars",
        include_str!("../migrations/0006_ai_and_avatars.sql"),
    ),
    (
        "0007_work_items",
        include_str!("../migrations/0007_work_items.sql"),
    ),
    (
        "0008_mail_and_board",
        include_str!("../migrations/0008_mail_and_board.sql"),
    ),
    ("0009_waits", include_str!("../migrations/0009_waits.sql")),
    (
        "0010_cloud_sync",
        include_str!("../migrations/0010_cloud_sync.sql"),
    ),
    (
        "0011_creator",
        include_str!("../migrations/0011_creator.sql"),
    ),
];

/// Everything that can go wrong in the store.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("a promise needs a few words")]
    EmptyTitle,
    #[error("that promise is not here any more")]
    NotFound,
    #[error("{0}")]
    NotAllowed(#[from] bindle_core::TransitionError),
    #[error("{0}")]
    NotSorted(#[from] bindle_core::SortError),
    #[error("it changed again since, so Undo would lose that change")]
    ChangedSince,
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("today's top 3 is full. Take one out first.")]
    TopThreeFull,
    #[error("only open promises can go in the top 3")]
    NotForTopThree,
    #[error("only an open promise someone owes you can be nudged")]
    NotNudgeable,
    #[error("a person needs a name")]
    EmptyName,
    #[error("there is already someone called {0}")]
    NameTaken(String),
    #[error("Bindle's AI connection is off. Turn it on in Settings.")]
    AiOff,
    #[error("the AI connection can read but not suggest. Change that in Settings.")]
    AiReadOnly,
    #[error("{0} suggestions are already waiting in Sort. Sort those first.")]
    TooManySuggestions(u32),
    #[error("that shared space is not on this device")]
    NoSuchSpace,
    #[error("only an open promise can be in Making")]
    NotForMaking,
    #[error("that is already a promise")]
    AlreadyPromised,
    #[error("{0} still has open promises. Finish or let go of them first.")]
    StillOpen(String),
    #[error("that is not one of the pictures")]
    UnknownAvatar,
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("migration {name} failed: {source}")]
    Migration {
        name: &'static str,
        source: rusqlite::Error,
    },
}

/// Opens (or creates) the database at `path` and applies any new migrations.
pub fn open(path: &Path) -> Result<Connection, StoreError> {
    let conn = Connection::open(path)?;
    prepare(conn)
}

/// Opens a throwaway in-memory database. Used by tests.
pub fn open_in_memory() -> Result<Connection, StoreError> {
    prepare(Connection::open_in_memory()?)
}

fn prepare(mut conn: Connection) -> Result<Connection, StoreError> {
    // WAL lets the app and the AI connection read while the other writes.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    migrate(&mut conn)?;
    Ok(conn)
}

/// Applies every migration newer than the database's current version.
pub fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let current: usize = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (index, (name, sql)) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)
            .map_err(|source| StoreError::Migration { name, source })?;
        tx.pragma_update(None, "user_version", (index + 1) as i64)?;
        tx.commit()?;
    }
    Ok(())
}

/// The schema version this build expects.
pub fn latest_version() -> usize {
    MIGRATIONS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_database_reaches_the_latest_version() {
        let conn = open_in_memory().expect("open");
        let version: usize = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("version");
        assert_eq!(version, latest_version());
    }

    #[test]
    fn running_migrations_twice_changes_nothing() {
        let mut conn = open_in_memory().expect("open");
        migrate(&mut conn).expect("second run is a no-op");
    }

    #[test]
    fn history_written_before_migration_3_survives_it() {
        let mut conn = Connection::open_in_memory().expect("open");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("pragma");
        for (name, sql) in &MIGRATIONS[..2] {
            conn.execute_batch(sql)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        }
        conn.pragma_update(None, "user_version", 2)
            .expect("version");
        conn.execute_batch(
            "INSERT INTO promise (id, title, created_at, updated_at) VALUES ('p1', 'Call mom', 'x', 'x');
             INSERT INTO promise_event (id, promise_id, at, kind, data)
               VALUES ('e1', 'p1', '2026-10-14', 'created', '{\"unsure_words\":\"kal\"}');",
        )
        .expect("old rows");
        migrate(&mut conn).expect("migrate");
        let data: String = conn
            .query_row("SELECT data FROM promise_event WHERE id = 'e1'", [], |r| {
                r.get(0)
            })
            .expect("kept");
        assert_eq!(data, r#"{"unsure_words":"kal"}"#);
        conn.execute(
            "INSERT INTO promise_event (id, promise_id, at, kind) VALUES ('e2', 'p1', 'x', 'undone')",
            [],
        )
        .expect("undone is now allowed");
    }

    #[test]
    fn an_unknown_state_is_rejected_by_the_schema() {
        let conn = open_in_memory().expect("open");
        let result = conn.execute(
            "INSERT INTO promise (id, title, state, created_at, updated_at)
             VALUES ('p1', 'Pay electricity bill', 'overdue', '2026-10-14', '2026-10-14')",
            [],
        );
        assert!(result.is_err(), "'overdue' is not a Bindle word");
    }

    #[test]
    fn promises_can_live_in_a_shared_space() {
        let conn = open_in_memory().expect("open");
        conn.execute_batch(
            "INSERT INTO space (id, kind, name, created_at)
               VALUES ('s1', 'ours', 'Upadhyay Home', '2026-10-14');
             INSERT INTO promise (id, title, space_id, created_at, updated_at)
               VALUES ('p1', 'Book Diwali train tickets', 's1', '2026-10-14', '2026-10-14');",
        )
        .expect("insert");
        let mode: String = conn
            .query_row("SELECT mode FROM promise WHERE id = 'p1'", [], |row| {
                row.get(0)
            })
            .expect("read");
        assert_eq!(mode, "life");
    }
}
