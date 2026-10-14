// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for Settings, first run and Wrap up (handbook P6 and P9).

use super::{announce, lock, Reply};
use crate::Db;
use bindle_core::notify::Notice;
use bindle_store::settings::{self, Settings};
use bindle_store::wrap_up::{self, WrapUpView};
use chrono::{Local, NaiveDate, TimeZone, Utc};
use tauri::{AppHandle, Emitter, State as Managed};

/// Sent to every window when settings change, so the theme and name follow.
pub const SETTINGS_CHANGED: &str = "settings-changed";

/// The person's settings, or the defaults on first run.
#[tauri::command]
pub fn get_settings(db: Managed<Db>) -> Reply<Settings> {
    let conn = lock(&db)?;
    Ok(settings::load(&conn)?)
}

/// Saves the settings and tells every window.
#[tauri::command]
pub fn save_settings(app: AppHandle, db: Managed<Db>, settings: Settings) -> Reply<Settings> {
    let saved = settings::save(&*lock(&db)?, &settings)?;
    let _ = app.emit(SETTINGS_CHANGED, &saved);
    Ok(saved)
}

/// Shows one notification now, so the person can see what they look like.
#[tauri::command]
pub fn test_notification(app: AppHandle) {
    crate::reminders::show(
        &app,
        &Notice {
            key: "test".into(),
            title: "Bindle".into(),
            body: "This is how a reminder looks. At most three a day.".into(),
        },
    );
}

/// Local midnight at the start of `day`, as UTC. Falls back to the UTC
/// midnight if a clock change makes local midnight ambiguous or missing.
fn midnight(day: NaiveDate) -> chrono::DateTime<Utc> {
    let local = day.and_hms_opt(0, 0, 0).expect("midnight exists");
    Local
        .from_local_datetime(&local)
        .earliest()
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(|| Utc.from_utc_datetime(&local))
}

/// Everything Wrap up shows for today.
#[tauri::command]
pub fn wrap_up_view(db: Managed<Db>) -> Reply<WrapUpView> {
    let today = Local::now().date_naive();
    let tomorrow = today.succ_opt().unwrap_or(today);
    let conn = lock(&db)?;
    Ok(wrap_up::view(
        &conn,
        today,
        midnight(today),
        midnight(tomorrow),
    )?)
}

/// Marks today as wrapped up.
#[tauri::command]
pub fn finish_wrap_up(
    app: AppHandle,
    db: Managed<Db>,
    mind_load: Option<u8>,
    note: Option<String>,
) -> Reply<()> {
    let today = Local::now().date_naive();
    wrap_up::finish(&*lock(&db)?, today, mind_load, note.as_deref(), Utc::now())?;
    announce(&app);
    Ok(())
}
