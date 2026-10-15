// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The sample Bindle: shown the first time Bindle opens, and again from
//! Settings. It lives in memory; the person's own database is never touched
//! while it shows.

use super::{lock, CommandError, Reply, PROMISES_CHANGED};
use crate::{Db, Paths};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State as Managed};

/// Whether the sample is showing, and whether the person has a Bindle of
/// their own to go back to.
#[derive(Debug, Serialize)]
pub struct SampleStatus {
    pub showing: bool,
    pub has_own: bool,
}

#[tauri::command]
pub fn sample_status(paths: Managed<Paths>) -> SampleStatus {
    let has_own = !crate::showing_sample()
        || bindle_store::open(&paths.db).is_ok_and(|conn| !crate::is_new(&conn));
    SampleStatus {
        showing: crate::showing_sample(),
        has_own,
    }
}

/// Leaves the sample for the person's own Bindle. A new one starts with
/// first run.
#[tauri::command]
pub fn sample_leave(app: AppHandle, db: Managed<Db>, paths: Managed<Paths>) -> Reply<()> {
    let own = bindle_store::open(&paths.db)?;
    *lock(&db)? = own;
    crate::set_sample(false);
    let _ = app.emit(PROMISES_CHANGED, ());
    Ok(())
}

/// Shows the sample again, fresh. The person's own Bindle stays as it is on
/// disk and comes back with [`sample_leave`].
#[tauri::command]
pub fn sample_open(app: AppHandle, db: Managed<Db>) -> Reply<()> {
    let sample = crate::sample_database().map_err(|e| CommandError {
        message: format!("Bindle could not make the sample: {e}"),
    })?;
    *lock(&db)? = sample;
    crate::set_sample(true);
    let _ = app.emit(PROMISES_CHANGED, ());
    Ok(())
}
