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
pub mod reminders;
pub mod watch;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

/// The open database, shared by every command.
pub struct Db(pub Mutex<rusqlite::Connection>);

/// Where the database and its safety copies live.
pub struct Paths {
    pub db: PathBuf,
    pub safety: PathBuf,
}

/// Starts the app, or only the AI connection when started with `--mcp`.
pub fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--mcp") {
        return bindle_mcp::run_from_args(args);
    }
    linux_fixes();
    run();
    std::process::ExitCode::SUCCESS
}

/// Works around a WebKitGTK bug: with some graphics drivers (NVIDIA on
/// Wayland and X11 most often) the window stays blank unless the DMA-BUF
/// renderer is off. A value the person already set is left alone.
fn linux_fixes() {
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

/// Starts the app.
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_notification::init());
    // New versions are found and installed from the app (see `UpdateBar`).
    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());
    builder
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let paths = Paths {
                db: dir.join("bindle.db"),
                safety: dir.join("safety copies"),
            };
            let conn = bindle_store::open(&paths.db)?;
            app.manage(Db(Mutex::new(conn)));
            app.manage(paths);
            #[cfg(desktop)]
            jot_window::register(app.handle())?;
            reminders::start(app.handle().clone());
            watch::start(app.handle().clone());
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
            commands::home::home_view,
            commands::home::add_to_top3,
            commands::home::remove_from_top3,
            commands::home::nudge,
            commands::browse::search_promises,
            commands::browse::state_counts,
            commands::browse::promises_in,
            commands::browse::people_list,
            commands::browse::person_detail,
            commands::browse::rename_person,
            commands::day::get_settings,
            commands::day::save_settings,
            commands::day::test_notification,
            commands::day::wrap_up_view,
            commands::day::finish_wrap_up,
            commands::data::ai_connection,
            commands::data::export_data,
            commands::data::safety_copies,
            commands::data::make_safety_copy,
            commands::data::restore_safety_copy,
            commands::data::set_person_look,
            commands::data::set_person_hidden,
        ])
        .run(tauri::generate_context!())
        .expect("Bindle could not start");
}
