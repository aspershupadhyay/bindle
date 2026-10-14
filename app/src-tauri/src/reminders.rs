// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Sends the notifications that are due (handbook P9). A background thread
//! looks every 30 seconds; the rules in `bindle_core::notify` decide. Each sent
//! notification is recorded first, so a crash can never send it twice.

use crate::Db;
use bindle_core::notify::{notices_due, DayState, Notice};
use bindle_store::{home, settings, wrap_up, StoreError};
use chrono::{Local, Utc};
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_notification::NotificationExt;

const LOOK_EVERY: Duration = Duration::from_secs(30);

/// Starts the reminder thread. It lives as long as the app.
pub fn start<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || loop {
        if let Err(error) = send_due(&app) {
            eprintln!("Bindle: reminders skipped this time: {error}");
        }
        std::thread::sleep(LOOK_EVERY);
    });
}

fn send_due<R: Runtime>(app: &AppHandle<R>) -> Result<(), StoreError> {
    let due = {
        let db = app.state::<Db>();
        let Ok(conn) = db.0.lock() else { return Ok(()) };
        let now = Local::now().naive_local();
        let today = now.date();
        let wanted = settings::load(&conn)?.notices();
        let day = DayState {
            wrapped_up: wrap_up::is_wrapped_up(&conn, today)?,
            today_count: home::today_count(&conn, today)?,
            timed: wrap_up::timed_on(&conn, today)?,
        };
        let due = notices_due(now, &wanted, &day, &wrap_up::sent_on(&conn, today)?);
        for notice in &due {
            wrap_up::record_sent(&conn, &notice.key, today, Utc::now())?;
        }
        due
    };
    for notice in due {
        show(app, &notice);
    }
    Ok(())
}

/// Shows one notification. If the system refuses, there is nothing more to do:
/// Bindle never retries or escalates.
pub fn show<R: Runtime>(app: &AppHandle<R>, notice: &Notice) {
    if let Err(error) = app
        .notification()
        .builder()
        .title(&notice.title)
        .body(&notice.body)
        .show()
    {
        eprintln!("Bindle: the system did not show a notification: {error}");
    }
}
