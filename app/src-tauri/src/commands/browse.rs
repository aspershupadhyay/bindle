// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for the Promises and People screens, and search (handbook P6).

use super::{announce, lock, Reply};
use crate::Db;
use bindle_core::{Promise, State};
use bindle_store::people::{self, PersonDetail, PersonSummary};
use bindle_store::promises::{self, StateCounts};
use bindle_store::search;
use chrono::{Duration, SecondsFormat, Utc};
use tauri::{AppHandle, State as Managed};

/// The most a list or search shows at once. Plenty for a screen, and keeps
/// the reply small however many promises there are.
const SHOW_AT_MOST: usize = 200;

/// How far back "kept lately" looks on a person's page.
const KEPT_LATELY_DAYS: i64 = 30;

/// Finds promises in any state by their words, the person or the first step.
#[tauri::command]
pub fn search_promises(db: Managed<Db>, query: String) -> Reply<Vec<Promise>> {
    let conn = lock(&db)?;
    Ok(search::search(&conn, &query, SHOW_AT_MOST)?)
}

/// How many promises are in each state.
#[tauri::command]
pub fn state_counts(db: Managed<Db>) -> Reply<StateCounts> {
    let conn = lock(&db)?;
    Ok(promises::state_counts(&conn)?)
}

/// One state's promises, in the order the Promises screen shows them.
#[tauri::command]
pub fn promises_in(db: Managed<Db>, state: State) -> Reply<Vec<Promise>> {
    let conn = lock(&db)?;
    Ok(promises::list_for_screen(&conn, state, SHOW_AT_MOST)?)
}

/// Everyone, with what you owe each other.
#[tauri::command]
pub fn people_list(db: Managed<Db>) -> Reply<Vec<PersonSummary>> {
    let conn = lock(&db)?;
    Ok(people::summaries(&conn)?)
}

/// Everything between you and one person.
#[tauri::command]
pub fn person_detail(db: Managed<Db>, id: String) -> Reply<PersonDetail> {
    let since = (Utc::now() - Duration::days(KEPT_LATELY_DAYS))
        .to_rfc3339_opts(SecondsFormat::Millis, true);
    let conn = lock(&db)?;
    Ok(people::detail(&conn, &id, &since)?)
}

/// Renames a person.
#[tauri::command]
pub fn rename_person(app: AppHandle, db: Managed<Db>, id: String, name: String) -> Reply<()> {
    let at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    people::rename(&*lock(&db)?, &id, &name, &at)?;
    announce(&app);
    Ok(())
}
