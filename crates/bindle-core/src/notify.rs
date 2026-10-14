// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Which notifications are due (handbook P9): quiet by default, at most
//! [`DAILY_LIMIT`] a day, never during quiet hours, and never about being
//! late. Pure: the caller passes the time, the settings and what was sent.

use chrono::{Duration, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

/// At most this many notifications in one day, whatever the settings.
pub const DAILY_LIMIT: usize = 3;

/// A notification is only sent within this long of its time. If Bindle was
/// closed then, it stays quiet rather than nagging late.
pub const GRACE_MINUTES: i64 = 15;

/// No notifications from 22:00 until 07:00.
pub fn is_quiet_hour(time: NaiveTime) -> bool {
    let start = NaiveTime::from_hms_opt(22, 0, 0).expect("valid");
    let end = NaiveTime::from_hms_opt(7, 0, 0).expect("valid");
    time >= start || time < end
}

/// What the person allowed. Everything is off unless they turn it on, except
/// reminders for promises they gave a time to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoticeSettings {
    /// "Your day" in the morning, at this time.
    pub morning: Option<NaiveTime>,
    /// A reminder to wrap up, at this time, if not done yet.
    pub wrap_up: Option<NaiveTime>,
    /// A reminder at the time set on a promise.
    pub timed_promises: bool,
}

impl Default for NoticeSettings {
    fn default() -> Self {
        Self {
            morning: None,
            wrap_up: None,
            timed_promises: true,
        }
    }
}

/// A promise of yours with a time today, for a reminder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimedPromise {
    pub id: String,
    pub title: String,
    pub at: NaiveDateTime,
}

/// What the day looks like right now, for the words of a notice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DayState {
    pub wrapped_up: bool,
    /// Your open promises for today.
    pub today_count: usize,
    pub timed: Vec<TimedPromise>,
}

/// One notification to show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Notice {
    /// Unique per day and subject, so nothing is ever sent twice.
    pub key: String,
    pub title: String,
    pub body: String,
}

/// The notifications due at `now`, given `sent_today` (their keys).
/// Promise reminders come first, then wrap up, then the morning note.
pub fn notices_due(
    now: NaiveDateTime,
    settings: &NoticeSettings,
    day: &DayState,
    sent_today: &[String],
) -> Vec<Notice> {
    if is_quiet_hour(now.time()) {
        return Vec::new();
    }
    let date = now.date().format("%Y-%m-%d");
    let in_window = |at: NaiveDateTime| at <= now && now < at + Duration::minutes(GRACE_MINUTES);
    let at_today = |time: NaiveTime| now.date().and_time(time);
    let mut due = Vec::new();

    if settings.timed_promises {
        for promise in day.timed.iter().filter(|p| in_window(p.at)) {
            due.push(Notice {
                key: format!("promise:{}:{date}", promise.id),
                title: promise.title.clone(),
                body: format!("Now, as you planned ({}).", promise.at.format("%-I:%M %p")),
            });
        }
    }
    if settings.wrap_up.is_some_and(|t| in_window(at_today(t))) && !day.wrapped_up {
        due.push(Notice {
            key: format!("wrap_up:{date}"),
            title: "Time to wrap up".into(),
            body: "About one minute. Close the day so your head can rest.".into(),
        });
    }
    if settings.morning.is_some_and(|t| in_window(at_today(t))) {
        due.push(Notice {
            key: format!("morning:{date}"),
            title: "Your day".into(),
            body: match day.today_count {
                0 => "Nothing planned yet. Jot down anything that comes up.".into(),
                1 => "1 thing today.".into(),
                n => format!("{n} things today."),
            },
        });
    }

    let room = DAILY_LIMIT.saturating_sub(sent_today.len());
    due.into_iter()
        .filter(|n| !sent_today.contains(&n.key))
        .take(room)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn at(h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, 14)
            .unwrap()
            .and_hms_opt(h, m, 0)
            .unwrap()
    }

    fn time(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    fn timed(id: &str, h: u32, m: u32) -> TimedPromise {
        TimedPromise {
            id: id.into(),
            title: format!("Call {id}"),
            at: at(h, m),
        }
    }

    #[test]
    fn by_default_only_promises_with_a_time_remind() {
        let day = DayState {
            timed: vec![timed("aman", 16, 0)],
            ..DayState::default()
        };
        let due = notices_due(at(16, 5), &NoticeSettings::default(), &day, &[]);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].title, "Call aman");
        assert_eq!(due[0].body, "Now, as you planned (4:00 PM).");
        assert!(notices_due(
            at(21, 0),
            &NoticeSettings::default(),
            &DayState::default(),
            &[]
        )
        .is_empty());
    }

    #[test]
    fn a_reminder_missed_by_more_than_the_grace_is_never_sent_late() {
        let day = DayState {
            timed: vec![timed("aman", 16, 0)],
            ..DayState::default()
        };
        assert!(notices_due(at(16, 15), &NoticeSettings::default(), &day, &[]).is_empty());
        assert!(notices_due(at(15, 59), &NoticeSettings::default(), &day, &[]).is_empty());
    }

    #[test]
    fn nothing_is_sent_twice() {
        let day = DayState {
            timed: vec![timed("aman", 16, 0)],
            ..DayState::default()
        };
        let first = notices_due(at(16, 0), &NoticeSettings::default(), &day, &[]);
        let sent: Vec<String> = first.iter().map(|n| n.key.clone()).collect();
        assert!(notices_due(at(16, 1), &NoticeSettings::default(), &day, &sent).is_empty());
    }

    #[test]
    fn wrap_up_reminds_only_when_the_day_is_not_wrapped_up() {
        let settings = NoticeSettings {
            wrap_up: Some(time(21, 0)),
            ..NoticeSettings::default()
        };
        assert_eq!(
            notices_due(at(21, 2), &settings, &DayState::default(), &[])[0].key,
            "wrap_up:2026-10-14"
        );
        let done = DayState {
            wrapped_up: true,
            ..DayState::default()
        };
        assert!(notices_due(at(21, 2), &settings, &done, &[]).is_empty());
    }

    #[test]
    fn quiet_hours_win_over_every_setting() {
        let settings = NoticeSettings {
            wrap_up: Some(time(22, 30)),
            ..NoticeSettings::default()
        };
        let day = DayState {
            timed: vec![timed("late", 23, 0)],
            ..DayState::default()
        };
        assert!(notices_due(at(22, 31), &settings, &day, &[]).is_empty());
        assert!(notices_due(at(23, 0), &settings, &day, &[]).is_empty());
        assert!(is_quiet_hour(time(6, 59)));
        assert!(!is_quiet_hour(time(7, 0)));
    }

    #[test]
    fn never_more_than_three_a_day() {
        let settings = NoticeSettings {
            morning: Some(time(9, 0)),
            wrap_up: Some(time(9, 0)),
            timed_promises: true,
        };
        let day = DayState {
            timed: vec![timed("a", 9, 0), timed("b", 9, 0), timed("c", 9, 0)],
            ..DayState::default()
        };
        let due = notices_due(at(9, 0), &settings, &day, &[]);
        assert_eq!(due.len(), 3);
        assert!(
            due.iter().all(|n| n.key.starts_with("promise:")),
            "promises come first"
        );
        let two_sent = vec!["x".to_string(), "y".to_string()];
        assert_eq!(notices_due(at(9, 0), &settings, &day, &two_sent).len(), 1);
    }

    #[test]
    fn the_morning_note_says_how_many_things_today() {
        let settings = NoticeSettings {
            morning: Some(time(8, 30)),
            ..NoticeSettings::default()
        };
        let day = DayState {
            today_count: 3,
            ..DayState::default()
        };
        assert_eq!(
            notices_due(at(8, 30), &settings, &day, &[])[0].body,
            "3 things today."
        );
    }
}
