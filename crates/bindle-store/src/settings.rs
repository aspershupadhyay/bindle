// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The person's settings (handbook P6, Settings), kept as one JSON value in
//! the `setting` table. Missing fields take their defaults, so adding a
//! setting later needs no migration.

use crate::ai::AiAccess;
use crate::StoreError;
use bindle_core::notify::NoticeSettings;
use chrono::NaiveTime;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

const KEY: &str = "settings";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// What to call the person: "Good evening, Sparsh."
    pub name: Option<String>,
    /// First run is finished.
    pub onboarded: bool,
    /// When the day is wrapped up.
    pub wrap_up_time: NaiveTime,
    /// Remind at wrap-up time. Off until the person turns it on.
    pub wrap_up_reminder: bool,
    /// When the morning note comes, if it is on.
    pub morning_time: NaiveTime,
    /// "Your day" in the morning. Off until the person turns it on.
    pub morning_note: bool,
    /// Remind at the time set on a promise.
    pub timed_reminders: bool,
    /// "system", "light" or "dark".
    pub theme: String,
    /// How far a connected AI may go. Off until the person turns it on.
    pub ai_access: AiAccess,
    /// The person's own picture, one of [`crate::people::AVATARS`].
    pub avatar: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            name: None,
            onboarded: false,
            wrap_up_time: NaiveTime::from_hms_opt(21, 0, 0).expect("valid"),
            wrap_up_reminder: false,
            morning_time: NaiveTime::from_hms_opt(8, 30, 0).expect("valid"),
            morning_note: false,
            timed_reminders: true,
            theme: "system".into(),
            ai_access: AiAccess::Off,
            avatar: None,
        }
    }
}

impl Settings {
    /// What the notification rules need.
    pub fn notices(&self) -> NoticeSettings {
        NoticeSettings {
            morning: self.morning_note.then_some(self.morning_time),
            wrap_up: self.wrap_up_reminder.then_some(self.wrap_up_time),
            timed_promises: self.timed_reminders,
        }
    }
}

/// Reads the settings, or the defaults if none were saved yet.
pub fn load(conn: &Connection) -> Result<Settings, StoreError> {
    let saved: Option<String> = conn
        .query_row("SELECT value FROM setting WHERE key = ?1", [KEY], |r| {
            r.get(0)
        })
        .optional()?;
    // A value that cannot be read gives the defaults rather than a broken app.
    Ok(saved
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default())
}

/// Saves the settings. A name is trimmed, and an empty one is no name.
pub fn save(conn: &Connection, settings: &Settings) -> Result<Settings, StoreError> {
    let mut clean = settings.clone();
    clean.name = clean
        .name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(Into::into);
    if !matches!(clean.theme.as_str(), "system" | "light" | "dark") {
        clean.theme = "system".into();
    }
    if clean
        .avatar
        .as_deref()
        .is_some_and(|a| !crate::people::is_avatar(a))
    {
        clean.avatar = None;
    }
    // Plain strings, booleans and times: serialising cannot fail.
    let json = serde_json::to_string(&clean).expect("settings serialise");
    conn.execute(
        "INSERT INTO setting (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [KEY, json.as_str()],
    )?;
    Ok(clean)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;

    #[test]
    fn a_new_person_gets_quiet_defaults() {
        let conn = open_in_memory().expect("open");
        let settings = load(&conn).expect("load");
        assert!(!settings.onboarded);
        assert_eq!(
            settings.notices(),
            NoticeSettings::default(),
            "only timed promises remind"
        );
    }

    #[test]
    fn settings_round_trip_with_a_trimmed_name() {
        let conn = open_in_memory().expect("open");
        let wanted = Settings {
            name: Some("  Sparsh ".into()),
            onboarded: true,
            wrap_up_reminder: true,
            ..Settings::default()
        };
        save(&conn, &wanted).expect("save");
        let read = load(&conn).expect("load");
        assert_eq!(read.name.as_deref(), Some("Sparsh"));
        assert!(read.notices().wrap_up.is_some());
    }

    #[test]
    fn missing_fields_take_their_defaults() {
        let conn = open_in_memory().expect("open");
        conn.execute(
            "INSERT INTO setting (key, value) VALUES ('settings', '{\"name\":\"Aspersh\"}')",
            [],
        )
        .expect("old value");
        let read = load(&conn).expect("load");
        assert_eq!(read.name.as_deref(), Some("Aspersh"));
        assert_eq!(read.theme, "system");
    }
}
