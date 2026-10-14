// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The typed commands the views can call.
//!
//! Each command has a matching TypeScript wrapper in `app/src/lib/commands.ts`.
//! Keep the two in step: same names, same argument names, same shapes.

use crate::Db;
use bindle_core::{Promise, State};
use bindle_store::{promises, StoreError};
use serde::Serialize;
use tauri::State as Managed;

/// What the views receive when a command fails. The message is plain words,
/// safe to show as is.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct CommandError {
    pub message: String,
}

impl From<StoreError> for CommandError {
    fn from(error: StoreError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

type Reply<T> = Result<T, CommandError>;

/// Facts about this build, shown in Settings and used by the first screen.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct AppInfo {
    pub version: &'static str,
    pub schema_version: usize,
}

/// Returns the app version and the schema version it expects.
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        schema_version: bindle_store::latest_version(),
    }
}

/// Saves what was typed in Jot down. The reply arrives only after the write is
/// committed, so the view can say "Saved" honestly.
#[tauri::command]
pub fn jot_down(db: Managed<Db>, title: String) -> Reply<Promise> {
    let mut conn = lock(&db)?;
    Ok(promises::jot_down(&mut conn, &title, chrono::Utc::now())?)
}

/// Lists every promise in one state, oldest first.
#[tauri::command]
pub fn list_promises(db: Managed<Db>, state: State) -> Reply<Vec<Promise>> {
    let conn = lock(&db)?;
    Ok(promises::list_by_state(&conn, state)?)
}

/// Counts the promises in one state.
#[tauri::command]
pub fn count_promises(db: Managed<Db>, state: State) -> Reply<u32> {
    let conn = lock(&db)?;
    Ok(promises::count_by_state(&conn, state)?)
}

fn lock<'a>(db: &'a Managed<Db>) -> Reply<std::sync::MutexGuard<'a, rusqlite::Connection>> {
    db.0.lock().map_err(|_| CommandError {
        message: "the database is busy after an earlier error; please restart Bindle".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_reports_the_latest_schema() {
        assert_eq!(app_info().schema_version, bindle_store::latest_version());
    }

    #[test]
    fn store_errors_become_plain_messages() {
        let error = CommandError::from(StoreError::EmptyTitle);
        assert_eq!(error.message, "a promise needs a few words");
    }
}
