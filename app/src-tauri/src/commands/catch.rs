// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Catch from a chat: paste a conversation, see the promises in it, and send
//! the ones you choose to Sort. The chat itself is never saved; each promise
//! keeps the words that made it one.

use super::{announce, jot_from, lock, Reply};
use crate::Db;
use bindle_core::Promise;
use bindle_parse::chat::Catch;
use bindle_parse::Reading;
use bindle_store::{people, promises, settings};
use serde::Deserialize;
use tauri::{AppHandle, State as Managed};

/// Reads a pasted chat. `me` is who the person is in it, once they say.
#[tauri::command]
pub fn catch_chat(db: Managed<Db>, text: String, me: Option<String>) -> Reply<Catch> {
    let known = people::names(&*lock(&db)?)?;
    Ok(bindle_parse::chat::catch(
        &text,
        me.as_deref(),
        chrono::Local::now().naive_local(),
        &known,
    ))
}

/// One promise the person chose, with the words it came from.
#[derive(Debug, Deserialize)]
pub struct Chosen {
    pub said: String,
    pub reading: Reading,
}

/// Sends the chosen promises to Sort, where each waits to be kept.
#[tauri::command]
pub fn catch_keep(app: AppHandle, db: Managed<Db>, chosen: Vec<Chosen>) -> Reply<Vec<Promise>> {
    let mut conn = lock(&db)?;
    let mode = settings::load(&conn)?.mode;
    let tz = iana_time_zone::get_timezone().ok();
    let mut saved = Vec::new();
    for item in chosen {
        let jot = jot_from(&item.said, item.reading, tz.clone());
        saved.push(promises::jot_down_in(
            &mut conn,
            &jot,
            mode,
            chrono::Utc::now(),
        )?);
    }
    drop(conn);
    announce(&app);
    Ok(saved)
}
