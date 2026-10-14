// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for Sort, the lifecycle, history and Undo (handbook P13).
//! Every change replies with the event that can undo it.

use super::{announce, lock, Reply};
use crate::Db;
use bindle_core::{State, When};
use bindle_store::lifecycle::{self, Change, HistoryEntry, Sorting};
use bindle_store::promises::{self, SortItem};
use chrono::Utc;
use tauri::{AppHandle, State as Managed};

/// Everything waiting in Sort, oldest first.
#[tauri::command]
pub fn sort_queue(db: Managed<Db>) -> Reply<Vec<SortItem>> {
    let conn = lock(&db)?;
    Ok(promises::sort_queue(&conn)?)
}

/// Keeps (`open`), puts off (`later`) or lets go (`let_go`) a promise in Sort.
#[tauri::command]
pub fn sort_promise(
    app: AppHandle,
    db: Managed<Db>,
    id: String,
    sorting: Sorting,
    to: State,
) -> Reply<Change> {
    let change = lifecycle::sort(&mut *lock(&db)?, &id, &sorting, to, Utc::now())?;
    announce(&app);
    Ok(change)
}

/// Marks a promise as kept.
#[tauri::command]
pub fn mark_done(app: AppHandle, db: Managed<Db>, id: String) -> Reply<Change> {
    let change = lifecycle::mark_done(&mut *lock(&db)?, &id, Utc::now())?;
    announce(&app);
    Ok(change)
}

/// Opens a done or let-go promise again.
#[tauri::command]
pub fn reopen(app: AppHandle, db: Managed<Db>, id: String) -> Reply<Change> {
    let change = lifecycle::reopen(&mut *lock(&db)?, &id, Utc::now())?;
    announce(&app);
    Ok(change)
}

/// Lets a promise go. It stays in history.
#[tauri::command]
pub fn let_go(app: AppHandle, db: Managed<Db>, id: String) -> Reply<Change> {
    let change = lifecycle::let_go(&mut *lock(&db)?, &id, Utc::now())?;
    announce(&app);
    Ok(change)
}

/// Moves a promise to a new day ("Move it").
#[tauri::command]
pub fn move_promise(
    app: AppHandle,
    db: Managed<Db>,
    id: String,
    when: When,
    is_deadline: bool,
) -> Reply<Change> {
    let change = lifecycle::move_to(&mut *lock(&db)?, &id, when, is_deadline, Utc::now())?;
    announce(&app);
    Ok(change)
}

/// Undoes one change, named by its event.
#[tauri::command]
pub fn undo(app: AppHandle, db: Managed<Db>, event_id: String) -> Reply<Change> {
    let change = lifecycle::undo(&mut *lock(&db)?, &event_id, Utc::now())?;
    announce(&app);
    Ok(change)
}

/// Undoes the most recent change, if there is one.
#[tauri::command]
pub fn undo_last(app: AppHandle, db: Managed<Db>) -> Reply<Option<Change>> {
    let change = lifecycle::undo_last(&mut *lock(&db)?, Utc::now())?;
    if change.is_some() {
        announce(&app);
    }
    Ok(change)
}

/// Everything that happened to one promise, oldest first.
#[tauri::command]
pub fn history(db: Managed<Db>, id: String) -> Reply<Vec<HistoryEntry>> {
    let conn = lock(&db)?;
    Ok(lifecycle::history(&conn, &id)?)
}
