// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Safety copies: whole copies of the database, made once a day and before a
//! restore, kept in a folder next to it. The newest [`COPIES_KEPT`] stay.
//!
//! A copy is a plain SQLite file, so it opens in any SQLite tool too.

use crate::StoreError;
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{Connection, DatabaseName, OpenFlags};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// How many safety copies are kept. Older ones are removed.
pub const COPIES_KEPT: usize = 14;

const PREFIX: &str = "bindle-";
const STAMP: &str = "%Y%m%d-%H%M%S";

/// One safety copy on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SafetyCopy {
    /// The file name, which is also how a copy is picked for restore.
    pub file_name: String,
    /// When it was made, RFC 3339 in UTC.
    pub made_at: String,
    pub bytes: u64,
}

/// Copies the database into `dir` now, then keeps only the newest copies.
pub fn make_copy(
    conn: &Connection,
    dir: &Path,
    now: DateTime<Utc>,
) -> Result<SafetyCopy, StoreError> {
    std::fs::create_dir_all(dir)?;
    let file_name = format!("{PREFIX}{}.db", now.format(STAMP));
    let path = dir.join(&file_name);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    conn.execute("VACUUM INTO ?1", [path.to_string_lossy()])?;
    prune(dir, COPIES_KEPT)?;
    list(dir)?
        .into_iter()
        .find(|c| c.file_name == file_name)
        .ok_or(StoreError::NotFound)
}

/// Makes a copy unless one was made in the last day. Returns it if it did.
pub fn daily_copy(
    conn: &Connection,
    dir: &Path,
    now: DateTime<Utc>,
) -> Result<Option<SafetyCopy>, StoreError> {
    let day_ago = now - chrono::Duration::hours(24);
    let newest = list(dir)?
        .into_iter()
        .next()
        .and_then(|c| made_at(&c.file_name));
    if newest.is_some_and(|at| at > day_ago) {
        return Ok(None);
    }
    make_copy(conn, dir, now).map(Some)
}

/// Every safety copy in `dir`, newest first. A missing folder means none.
pub fn list(dir: &Path) -> Result<Vec<SafetyCopy>, StoreError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut copies = Vec::new();
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let Some(at) = made_at(&file_name) else {
            continue;
        };
        copies.push(SafetyCopy {
            made_at: at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            bytes: entry.metadata()?.len(),
            file_name,
        });
    }
    copies.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    Ok(copies)
}

/// Puts a safety copy back in place of the current data. The current data is
/// copied first, so a restore can itself be undone by restoring that copy.
pub fn restore(
    conn: &mut Connection,
    dir: &Path,
    file_name: &str,
    now: DateTime<Utc>,
) -> Result<SafetyCopy, StoreError> {
    let path = copy_path(dir, file_name)?;
    check_is_bindle(&path)?;
    let before = make_copy(conn, dir, now)?;
    conn.restore(
        DatabaseName::Main,
        &path,
        None::<fn(rusqlite::backup::Progress)>,
    )?;
    crate::migrate(conn)?;
    Ok(before)
}

/// The path of a listed copy. Only names [`list`] gives are accepted, so a
/// restore can never read a file outside `dir`.
fn copy_path(dir: &Path, file_name: &str) -> Result<PathBuf, StoreError> {
    if made_at(file_name).is_none() || file_name.contains(['/', '\\']) {
        return Err(StoreError::NotFound);
    }
    let path = dir.join(file_name);
    if !path.is_file() {
        return Err(StoreError::NotFound);
    }
    Ok(path)
}

/// Refuses a file that is not a Bindle database, or is from a newer Bindle.
fn check_is_bindle(path: &Path) -> Result<(), StoreError> {
    let copy = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let version: usize = copy.pragma_query_value(None, "user_version", |r| r.get(0))?;
    let has_promises: bool = copy.query_row(
        "SELECT count(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'promise'",
        [],
        |r| r.get(0),
    )?;
    if !has_promises || version > crate::latest_version() {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

fn made_at(file_name: &str) -> Option<DateTime<Utc>> {
    let stamp = file_name.strip_prefix(PREFIX)?.strip_suffix(".db")?;
    NaiveDateTime::parse_from_str(stamp, STAMP)
        .ok()
        .map(|t| t.and_utc())
}

fn prune(dir: &Path, keep: usize) -> Result<(), StoreError> {
    for old in list(dir)?.into_iter().skip(keep) {
        std::fs::remove_file(dir.join(old.file_name))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "backup_tests.rs"]
mod tests;
