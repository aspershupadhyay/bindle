// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The Bindle desktop app (handbook P4).
//!
//! This crate only wires things together: it opens the database in the app's
//! data folder and exposes the typed commands in [`commands`] to the React views.
//! All rules live in `bindle-core`; all SQL lives in `bindle-store`.

pub mod commands;
#[cfg(desktop)]
pub mod jot_window;

use std::sync::Mutex;
use tauri::Manager;

/// The open database, shared by every command.
pub struct Db(pub Mutex<rusqlite::Connection>);

/// Starts the app.
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = bindle_store::open(&dir.join("bindle.db"))?;
            app.manage(Db(Mutex::new(conn)));
            #[cfg(desktop)]
            jot_window::register(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::jot_down,
            commands::read_jot,
            commands::show_jot,
            commands::hide_jot,
            commands::list_promises,
            commands::count_promises,
            commands::lifecycle::sort_queue,
            commands::lifecycle::sort_promise,
            commands::lifecycle::mark_done,
            commands::lifecycle::reopen,
            commands::lifecycle::let_go,
            commands::lifecycle::move_promise,
            commands::lifecycle::undo,
            commands::lifecycle::undo_last,
            commands::lifecycle::history,
        ])
        .run(tauri::generate_context!())
        .expect("Bindle could not start");
}
