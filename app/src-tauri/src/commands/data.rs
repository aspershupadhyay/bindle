// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for the person's data: the AI connection (handbook P8), export,
//! safety copies, and how people look (picture, pronouns, kept from AI).

use super::{announce, lock, CommandError, Reply};
use crate::{Db, Paths};
use bindle_store::ai::{self, LogEntry};
use bindle_store::backup::{self, SafetyCopy};
use bindle_store::{export, people};
use chrono::{Local, SecondsFormat, Utc};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State as Managed};

/// What Settings shows about the AI connection.
#[derive(Debug, Serialize)]
pub struct AiConnection {
    /// The program an AI app starts: this Bindle.
    pub command: String,
    pub args: Vec<String>,
    /// The block to paste into an AI app's MCP settings.
    pub config: String,
    /// What an AI read or suggested lately, newest first.
    pub log: Vec<LogEntry>,
}

/// How to connect an AI app, and what connected ones did lately.
#[tauri::command]
pub fn ai_connection(db: Managed<Db>, paths: Managed<Paths>) -> Reply<AiConnection> {
    let command = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "bindle".into());
    let args = vec![
        "--mcp".to_string(),
        "--db".to_string(),
        paths.db.to_string_lossy().into_owned(),
    ];
    let config = serde_json::to_string_pretty(&serde_json::json!({
        "mcpServers": { "bindle": { "command": command, "args": args } }
    }))
    .expect("plain strings serialise");
    let log = ai::recent(&*lock(&db)?, 30)?;
    Ok(AiConnection {
        command,
        args,
        config,
        log,
    })
}

/// Writes everything to a file in Downloads ("json" for all of it, "csv" for
/// the promises) and returns where it went.
#[tauri::command]
pub fn export_data(app: AppHandle, db: Managed<Db>, format: String) -> Reply<String> {
    let conn = lock(&db)?;
    let (body, extension) = match format.as_str() {
        "csv" => (export::promises_csv(&conn)?, "csv"),
        _ => {
            let value = export::to_json(&conn, Utc::now())?;
            (
                serde_json::to_string_pretty(&value).expect("JSON values serialise"),
                "json",
            )
        }
    };
    drop(conn);
    let folder = export_folder(&app)?;
    let stamp = Local::now().format("%Y-%m-%d %H%M");
    let path = folder.join(format!("Bindle export {stamp}.{extension}"));
    std::fs::write(&path, body).map_err(|e| CommandError {
        message: format!("could not write {}: {e}", path.display()),
    })?;
    Ok(path.to_string_lossy().into_owned())
}

/// Downloads, or else Documents, the home folder, or the app's own folder.
fn export_folder(app: &AppHandle) -> Reply<PathBuf> {
    let paths = app.path();
    let folder = paths
        .download_dir()
        .or_else(|_| paths.document_dir())
        .or_else(|_| paths.home_dir())
        .or_else(|_| paths.app_data_dir())
        .map_err(|e| CommandError {
            message: format!("no folder to export to: {e}"),
        })?;
    std::fs::create_dir_all(&folder).map_err(|e| CommandError {
        message: format!("could not make {}: {e}", folder.display()),
    })?;
    Ok(folder)
}

/// Every safety copy, newest first.
#[tauri::command]
pub fn safety_copies(paths: Managed<Paths>) -> Reply<Vec<SafetyCopy>> {
    Ok(backup::list(&paths.safety)?)
}

/// Makes a safety copy now.
#[tauri::command]
pub fn make_safety_copy(db: Managed<Db>, paths: Managed<Paths>) -> Reply<SafetyCopy> {
    Ok(backup::make_copy(&*lock(&db)?, &paths.safety, Utc::now())?)
}

/// Puts a safety copy back. Today's data is copied first, and that copy is
/// returned so the view can offer to go back to it.
#[tauri::command]
pub fn restore_safety_copy(
    app: AppHandle,
    db: Managed<Db>,
    paths: Managed<Paths>,
    file_name: String,
) -> Reply<SafetyCopy> {
    let before = backup::restore(&mut *lock(&db)?, &paths.safety, &file_name, Utc::now())?;
    announce(&app);
    super::day::announce_settings(&app, &db);
    Ok(before)
}

/// Sets a person's picture (or their first letter, for none) and pronouns.
#[tauri::command]
pub fn set_person_look(
    app: AppHandle,
    db: Managed<Db>,
    id: String,
    avatar: Option<String>,
    pronouns: Option<String>,
) -> Reply<()> {
    let at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    people::set_look(
        &*lock(&db)?,
        &id,
        avatar.as_deref(),
        pronouns.as_deref(),
        &at,
    )?;
    announce(&app);
    Ok(())
}

/// Keeps a person, and their promises, from any connected AI, or stops.
#[tauri::command]
pub fn set_person_hidden(app: AppHandle, db: Managed<Db>, id: String, hidden: bool) -> Reply<()> {
    ai::set_person_hidden(&*lock(&db)?, &id, hidden)?;
    announce(&app);
    Ok(())
}
