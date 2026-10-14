// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for the Work Desk and its insights (v2 design, Desk).

use super::{lock, Reply};
use crate::Db;
use bindle_store::insights::{self, DeskView};
use chrono::{Local, Utc};
use tauri::State as Managed;

/// Everything the Desk shows: what needs you now, how long people wait on
/// you, what shipped, and the week's work.
#[tauri::command]
pub fn desk_view(db: Managed<Db>) -> Reply<DeskView> {
    let today = Local::now().date_naive();
    Ok(insights::desk_view(
        &*lock(&db)?,
        Utc::now(),
        today,
        super::home::week_start(),
    )?)
}
