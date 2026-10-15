// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Notices changes made outside the app, such as an AI suggestion arriving
//! through the AI connection, and tells every window so Sort shows it. Also
//! makes the day's safety copy.
//!
//! SQLite's `data_version` changes whenever another connection commits, so
//! the check is one cheap pragma.

use crate::{Db, Paths};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime};

const LOOK_EVERY: Duration = Duration::from_secs(2);
/// How often the daily safety copy is checked for: every 10 minutes.
const COPY_EVERY_TICKS: u32 = 300;

/// Starts the watching thread. It lives as long as the app.
pub fn start<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || {
        let mut last: Option<i64> = None;
        let mut tick: u32 = 0;
        loop {
            let db = app.state::<Db>();
            if let Ok(conn) = db.0.lock() {
                let version = conn
                    .pragma_query_value(None, "data_version", |r| r.get::<_, i64>(0))
                    .ok();
                if last.is_some() && version != last {
                    let _ = app.emit(crate::commands::PROMISES_CHANGED, ());
                }
                last = version;
                if tick == 0 && !crate::showing_sample() {
                    let paths = app.state::<Paths>();
                    if let Err(error) =
                        bindle_store::backup::daily_copy(&conn, &paths.safety, chrono::Utc::now())
                    {
                        eprintln!("Bindle: no safety copy this time: {error}");
                    }
                }
            }
            tick = (tick + 1) % COPY_EVERY_TICKS;
            std::thread::sleep(LOOK_EVERY);
        }
    });
}
