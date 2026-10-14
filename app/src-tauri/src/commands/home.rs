// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for Life Home: the whole view, the top 3 and Nudge (handbook P6).
//! "Today" is the person's local day.

use super::{announce, lock, Reply};
use crate::Db;
use bindle_core::week_of;
use bindle_store::home::{self, HomeView};
use chrono::{Local, NaiveDate, TimeZone, Utc};
use tauri::{AppHandle, State as Managed};

fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Monday 00:00 of this week, local, as UTC. Falls back to now if the
/// clock change rules make that moment ambiguous or missing.
fn week_start() -> chrono::DateTime<Utc> {
    let monday = week_of(today())[0]
        .and_hms_opt(0, 0, 0)
        .expect("midnight exists");
    Local
        .from_local_datetime(&monday)
        .earliest()
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(Utc::now)
}

/// Everything Home shows.
#[tauri::command]
pub fn home_view(db: Managed<Db>) -> Reply<HomeView> {
    let conn = lock(&db)?;
    Ok(home::home(&conn, today(), week_start())?)
}

/// Puts a promise in a day's top 3: today, unless `day` says otherwise
/// (Wrap up picks tomorrow's).
#[tauri::command]
pub fn add_to_top3(
    app: AppHandle,
    db: Managed<Db>,
    id: String,
    day: Option<NaiveDate>,
) -> Reply<()> {
    home::add_to_top3(&mut *lock(&db)?, day.unwrap_or_else(today), &id, Utc::now())?;
    announce(&app);
    Ok(())
}

/// Takes a promise out of a day's top 3: today, unless `day` says otherwise.
#[tauri::command]
pub fn remove_from_top3(
    app: AppHandle,
    db: Managed<Db>,
    id: String,
    day: Option<NaiveDate>,
) -> Reply<()> {
    home::remove_from_top3(&mut *lock(&db)?, day.unwrap_or_else(today), &id, Utc::now())?;
    announce(&app);
    Ok(())
}

/// Records that the person nudged someone. Bindle never sends the message.
#[tauri::command]
pub fn nudge(app: AppHandle, db: Managed<Db>, id: String) -> Reply<()> {
    home::nudge(&*lock(&db)?, &id, Utc::now())?;
    announce(&app);
    Ok(())
}
