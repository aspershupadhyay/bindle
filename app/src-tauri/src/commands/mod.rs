// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The typed commands the views can call.
//!
//! Each command has a matching TypeScript wrapper in `app/src/lib/commands.ts`.
//! Keep the two in step: same names, same argument names, same shapes.
//! Sort, the lifecycle and Undo live in [`lifecycle`]; Home in [`home`]; Promises, People
//! and search in [`browse`]; Settings, first run and Wrap up in [`day`]; the AI
//! connection, export, safety copies and people's pictures in [`data`].

pub mod browse;
pub mod data;
pub mod day;
pub mod home;
pub mod lifecycle;

use crate::Db;
use bindle_core::{Promise, State};
use bindle_parse::Reading;
use bindle_store::{people, promises, StoreError};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State as Managed};

/// Sent to every window after any promise is saved or changed, so lists refresh.
pub const PROMISES_CHANGED: &str = "promises-changed";

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

pub(crate) type Reply<T> = Result<T, CommandError>;

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

/// Reads a line as it is typed, so the views can show chips for the day,
/// time, amount and person before anything is saved.
#[tauri::command]
pub fn read_jot(db: Managed<Db>, text: String) -> Reply<Reading> {
    let known = people::names(&*lock(&db)?)?;
    Ok(bindle_parse::read(
        &text,
        chrono::Local::now().naive_local(),
        &known,
    ))
}

/// Saves what was typed in Jot down, with what Bindle understood from it.
/// The reply arrives only after the write is committed, so the view can say
/// "Saved" honestly. Every window then hears [`PROMISES_CHANGED`].
#[tauri::command]
pub fn jot_down(app: AppHandle, db: Managed<Db>, text: String) -> Reply<Promise> {
    let mut conn = lock(&db)?;
    let known = people::names(&conn)?;
    let reading = bindle_parse::read(&text, chrono::Local::now().naive_local(), &known);
    let jot = jot_from(&text, reading, iana_time_zone::get_timezone().ok());
    let promise = promises::jot_down(&mut conn, &jot, chrono::Utc::now())?;
    announce(&app);
    Ok(promise)
}

/// Tells every window that promises changed. The change is already saved, so
/// a window that misses the news only shows it a little later.
pub(crate) fn announce(app: &AppHandle) {
    let _ = app.emit(PROMISES_CHANGED, ());
}

/// Turns what the parser read into what the store saves.
fn jot_from(text: &str, reading: Reading, tz: Option<String>) -> promises::Jot {
    promises::Jot {
        unsure_words: reading.unsure_words,
        text: text.to_string(),
        title: reading.title,
        when: reading.when,
        is_deadline: reading.is_deadline,
        amount: reading.amount,
        person: reading.person,
        direction: reading.direction,
        strength: reading.strength,
        tz,
    }
}

/// Opens the Jot down window, as Alt+Shift+J does.
#[tauri::command]
pub fn show_jot(app: AppHandle) {
    #[cfg(desktop)]
    crate::jot_window::show(&app);
    #[cfg(not(desktop))]
    let _ = app;
}

/// Puts the Jot down window away.
#[tauri::command]
pub fn hide_jot(app: AppHandle) {
    #[cfg(desktop)]
    crate::jot_window::hide(&app);
    #[cfg(not(desktop))]
    let _ = app;
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

pub(crate) fn lock<'a>(
    db: &'a Managed<Db>,
) -> Reply<std::sync::MutexGuard<'a, rusqlite::Connection>> {
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
    fn a_jot_keeps_the_typed_text_and_the_reading() {
        let now = chrono::NaiveDate::from_ymd_opt(2026, 10, 14)
            .and_then(|d| d.and_hms_opt(18, 30, 0))
            .expect("valid time");
        let text = "Pay electricity bill ₹2,340 by friday";
        let jot = jot_from(
            text,
            bindle_parse::read(text, now, &[]),
            Some("Asia/Kolkata".into()),
        );
        assert_eq!(jot.text, text);
        assert_eq!(jot.title, "Pay electricity bill");
        assert!(jot.is_deadline);
        assert_eq!(jot.amount.map(|m| m.minor), Some(234_000));
        assert_eq!(jot.tz.as_deref(), Some("Asia/Kolkata"));
    }

    #[test]
    fn store_errors_become_plain_messages() {
        let error = CommandError::from(StoreError::EmptyTitle);
        assert_eq!(error.message, "a promise needs a few words");
    }
}
