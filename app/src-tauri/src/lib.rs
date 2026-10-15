// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The Bindle app for desktops and phones (handbook P4).
//!
//! This crate only wires things together: it opens the database in the app's
//! data folder and exposes the typed commands in [`commands`] to the React views.
//! All rules live in `bindle-core`; all SQL lives in `bindle-store`.

pub mod commands;
#[cfg(desktop)]
pub mod jot_window;
pub mod reminders;
pub mod secrets;
pub mod watch;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::Manager;

/// The open database, shared by every command.
pub struct Db(pub Mutex<rusqlite::Connection>);

/// Whether the sample Bindle is showing instead of the person's own.
static SAMPLE: AtomicBool = AtomicBool::new(false);

/// True while the sample Bindle is showing. Nothing then reaches the
/// network, the keychain, reminders or safety copies.
pub fn showing_sample() -> bool {
    SAMPLE.load(Ordering::SeqCst)
}

pub(crate) fn set_sample(on: bool) {
    SAMPLE.store(on, Ordering::SeqCst);
}

/// A fresh database holding the sample Bindle, dated from today.
pub(crate) fn sample_database() -> Result<rusqlite::Connection, bindle_store::StoreError> {
    let mut conn = bindle_store::open_in_memory()?;
    bindle_store::sample::seed(
        &mut conn,
        chrono::Local::now().date_naive(),
        chrono::Utc::now(),
    )?;
    Ok(conn)
}

/// A brand-new Bindle: first run not done and nothing saved yet.
pub(crate) fn is_new(conn: &rusqlite::Connection) -> bool {
    let onboarded = bindle_store::settings::load(conn).is_ok_and(|s| s.onboarded);
    let saved: i64 = conn
        .query_row("SELECT count(*) FROM promise", [], |r| r.get(0))
        .unwrap_or(1);
    !onboarded && saved == 0
}

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

/// Starts the app. On phones this is where the app begins.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init());
    // New versions are found and installed from the app (see `UpdateBar`).
    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());
    builder
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            secrets::use_folder(&dir);
            let paths = Paths {
                db: dir.join("bindle.db"),
                safety: dir.join("safety copies"),
            };
            let mut conn = bindle_store::open(&paths.db)?;
            // The first time, Bindle opens on the sample so nobody meets an
            // empty screen. The person's own Bindle starts when they choose.
            if is_new(&conn) {
                conn = sample_database()?;
                set_sample(true);
            }
            app.manage(Db(Mutex::new(conn)));
            app.manage(paths);
            #[cfg(desktop)]
            jot_window::register(app.handle())?;
            reminders::start(app.handle().clone());
            watch::start(app.handle().clone());
            commands::work::start(app.handle().clone());
            commands::mail::start(app.handle().clone());
            commands::cloud::start(app.handle().clone());
            commands::creator::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::sample::sample_status,
            commands::sample::sample_leave,
            commands::sample::sample_open,
            commands::jot_down,
            commands::read_jot,
            commands::catch::catch_chat,
            commands::catch::catch_keep,
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
            commands::browse::archive_person,
            commands::browse::unarchive_person,
            commands::browse::archived_people,
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
            commands::creator::creator_view,
            commands::creator::comments_waiting,
            commands::creator::youtube_key_page,
            commands::creator::youtube_connect,
            commands::creator::youtube_disconnect,
            commands::creator::youtube_refresh,
            commands::creator::comment_make_promise,
            commands::creator::comment_hide,
            commands::creator::comment_unhide,
            commands::creator::open_on_youtube,
            commands::work::code_view,
            commands::work::code_cards,
            commands::work::reviews_owed,
            commands::work::github_token_page,
            commands::work::github_connect,
            commands::work::github_disconnect,
            commands::work::github_refresh,
            commands::work::make_promise,
            commands::work::open_on_github,
            commands::mail::inbox_view,
            commands::mail::replies_owed,
            commands::mail::mail_providers,
            commands::mail::gmail_connect,
            commands::mail::gmail_disconnect,
            commands::mail::gmail_refresh,
            commands::mail::mail_make_promise,
            commands::mail::mail_dismiss,
            commands::mail::mail_restore,
            commands::mail::open_in_gmail,
            commands::board::board_view,
            commands::desk::desk_view,
            commands::cloud::cloud_status,
            commands::cloud::cloud_start,
            commands::cloud::cloud_recovery_kit,
            commands::cloud::cloud_kit_saved,
            commands::cloud::cloud_restore,
            commands::cloud::ours_create,
            commands::cloud::ours_invite,
            commands::cloud::ours_join,
            commands::cloud::ours_share,
            commands::cloud::ours_jot,
            commands::cloud::ours_view,
            commands::cloud::ours_leave,
            commands::cloud::cloud_sync,
            commands::board::set_making,
        ])
        .run(tauri::generate_context!())
        .expect("Bindle could not start");
}
