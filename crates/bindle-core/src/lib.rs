// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The rules every Bindle promise follows.
//!
//! This crate holds pure logic only: no database, no network, no clock.
//! Callers pass "today" in, so every rule is easy to test.
//! The rules come from the handbook, chapter P5 (data model).

pub mod notify;

use chrono::{Datelike, Duration, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Where a promise is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// Just jotted down, waiting in Sort.
    Unsorted,
    /// Sorted and active.
    Open,
    /// Kept for later, out of the way.
    Later,
    /// Kept.
    Done,
    /// Consciously let go. Still in history, never deleted.
    LetGo,
}

/// Who the promise is owed by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// "I'll do it": you owe this.
    Mine,
    /// "They'll do it": someone owes you this.
    Theirs,
    /// "Just for me": a promise to yourself.
    #[serde(rename = "self")]
    SelfPromise,
}

/// How firm the promise is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strength {
    /// Has a real date that matters.
    Firm,
    /// "Let's catch up sometime". Never shown as running late.
    Soft,
}

/// Which side of life a promise belongs to (handbook P14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Life,
    Work,
}

/// Gives each enum the exact word the database stores, and reads it back.
macro_rules! stored_words {
    ($name:ident { $($variant:ident => $word:literal),+ $(,)? }) => {
        impl $name {
            /// The word stored in the database for this value.
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $word),+ }
            }

            /// Reads a stored word back. Unknown words give `None`.
            pub fn from_word(word: &str) -> Option<Self> {
                match word { $($word => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

stored_words!(State {
    Unsorted => "unsorted",
    Open => "open",
    Later => "later",
    Done => "done",
    LetGo => "let_go",
});
stored_words!(Direction { Mine => "mine", Theirs => "theirs", SelfPromise => "self" });
stored_words!(Strength { Firm => "firm", Soft => "soft" });
stored_words!(Mode { Life => "life", Work => "work" });

/// A day, with a time of day when the person gave one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct When {
    pub day: NaiveDate,
    pub time: Option<NaiveTime>,
}

/// An amount of money in minor units (paise, cents), so sums never drift.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// For example 234000 for ₹2,340.
    pub minor: i64,
    /// ISO 4217 code, for example "INR".
    pub currency: String,
}

/// A promise as the app sees it: the fields every screen needs.
///
/// Days and times are local to the person. Timestamps are RFC 3339 text, as stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Promise {
    pub id: String,
    pub title: String,
    pub state: State,
    pub direction: Option<Direction>,
    pub strength: Strength,
    pub mode: Mode,
    pub first_step: Option<String>,
    /// The day it is planned for.
    pub do_on: Option<When>,
    /// The day it must be done by.
    pub due: Option<When>,
    pub amount: Option<Money>,
    /// The display name of the person it is with.
    pub person: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// When it was done or let go.
    pub closed_at: Option<String>,
}

impl Promise {
    /// The day that matters for this promise: the deadline if there is one,
    /// otherwise the planned day.
    pub fn day(&self) -> Option<NaiveDate> {
        self.due.or(self.do_on).map(|w| w.day)
    }

    /// Running late, measured against `today`. See [`is_running_late`].
    pub fn is_running_late(&self, today: NaiveDate) -> bool {
        let day = |when: Option<When>| when.map(|w| w.day);
        is_running_late(
            self.state,
            self.strength,
            day(self.due),
            day(self.do_on),
            today,
        )
    }
}

/// A change that the lifecycle does not allow.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("a promise cannot go from {from:?} to {to:?}")]
pub struct TransitionError {
    pub from: State,
    pub to: State,
}

/// Checks a state change against the lifecycle in handbook P5.
///
/// Moving a promise to a new date keeps it `Open`, so `Open -> Open` is allowed.
pub fn check_transition(from: State, to: State) -> Result<(), TransitionError> {
    use State::*;
    let allowed = match from {
        Unsorted => matches!(to, Open | Later | LetGo),
        Open => matches!(to, Open | Done | Later | LetGo),
        Later => matches!(to, Open | LetGo),
        Done => matches!(to, Open),
        LetGo => matches!(to, Open),
    };
    if allowed {
        Ok(())
    } else {
        Err(TransitionError { from, to })
    }
}

/// Why a promise cannot be kept as sorted yet. The messages are shown as is.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SortError {
    #[error("Say who will do it: you, them, or just for you.")]
    NoDirection,
    #[error("A firm promise needs a day. Pick one, or make it Sometime.")]
    NoDay,
}

/// Checks a promise being kept from Sort (handbook principle 1, close the
/// loop): it must say who owes it, and a firm promise must have a day.
/// Soft promises ("sometime") may stay without one.
pub fn check_sorted(
    direction: Option<Direction>,
    strength: Strength,
    when: Option<When>,
) -> Result<(), SortError> {
    if direction.is_none() {
        return Err(SortError::NoDirection);
    }
    if strength == Strength::Firm && when.is_none() {
        return Err(SortError::NoDay);
    }
    Ok(())
}

/// Returns `true` when a promise belongs under "Running late" on Home.
///
/// Only open, firm promises with a date before today count. Soft promises are
/// never late, because nagging about them creates guilt without helping anyone.
/// The due date wins; if there is none, the planned day is used.
pub fn is_running_late(
    state: State,
    strength: Strength,
    due: Option<NaiveDate>,
    do_on: Option<NaiveDate>,
    today: NaiveDate,
) -> bool {
    state == State::Open
        && strength == Strength::Firm
        && due.or(do_on).is_some_and(|date| date < today)
}

/// How many promises fill a day on the week strip. A day with this many or
/// more shows in honey, with a gentle note (handbook P6). Never red.
pub const DAY_CAPACITY: usize = 5;

/// The Monday-to-Sunday week that holds `today`.
pub fn week_of(today: NaiveDate) -> [NaiveDate; 7] {
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    std::array::from_fn(|i| monday + Duration::days(i as i64))
}

/// Returns `true` when someone else owes you this and it is still open.
pub fn is_waiting_on_others(state: State, direction: Option<Direction>) -> bool {
    state == State::Open && direction == Some(Direction::Theirs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
    }

    #[test]
    fn a_firm_open_promise_past_its_due_date_is_running_late() {
        let today = day(2026, 10, 14);
        assert!(is_running_late(
            State::Open,
            Strength::Firm,
            Some(day(2026, 10, 12)),
            None,
            today
        ));
    }

    #[test]
    fn a_soft_promise_is_never_running_late() {
        let today = day(2026, 10, 14);
        assert!(!is_running_late(
            State::Open,
            Strength::Soft,
            Some(day(2026, 1, 1)),
            None,
            today
        ));
    }

    #[test]
    fn a_promise_due_today_is_not_late_yet() {
        let today = day(2026, 10, 14);
        assert!(!is_running_late(
            State::Open,
            Strength::Firm,
            Some(today),
            None,
            today
        ));
    }

    #[test]
    fn the_planned_day_counts_when_there_is_no_due_date() {
        let today = day(2026, 10, 14);
        assert!(is_running_late(
            State::Open,
            Strength::Firm,
            None,
            Some(day(2026, 10, 13)),
            today
        ));
    }

    #[test]
    fn a_promise_uses_the_same_words_as_the_database() {
        let json = serde_json::to_string(&(State::LetGo, Direction::SelfPromise, Mode::Work))
            .expect("serialise");
        assert_eq!(json, r#"["let_go","self","work"]"#);
    }

    #[test]
    fn stored_words_read_back_to_the_same_value() {
        for state in [
            State::Unsorted,
            State::Open,
            State::Later,
            State::Done,
            State::LetGo,
        ] {
            assert_eq!(State::from_word(state.as_str()), Some(state));
        }
        assert_eq!(State::from_word("overdue"), None);
    }

    #[test]
    fn done_promises_can_be_reopened_but_not_sent_to_later() {
        assert!(check_transition(State::Done, State::Open).is_ok());
        assert_eq!(
            check_transition(State::Done, State::Later),
            Err(TransitionError {
                from: State::Done,
                to: State::Later
            })
        );
    }

    #[test]
    fn a_kept_promise_says_who_owes_it_and_when() {
        let friday = Some(When {
            day: day(2026, 10, 16),
            time: None,
        });
        assert_eq!(
            check_sorted(None, Strength::Firm, friday),
            Err(SortError::NoDirection)
        );
        assert_eq!(
            check_sorted(Some(Direction::Mine), Strength::Firm, None),
            Err(SortError::NoDay)
        );
        assert!(check_sorted(Some(Direction::Mine), Strength::Firm, friday).is_ok());
    }

    #[test]
    fn a_soft_promise_can_be_kept_without_a_day() {
        assert!(check_sorted(Some(Direction::Theirs), Strength::Soft, None).is_ok());
    }

    #[test]
    fn moving_a_promise_keeps_it_open() {
        assert!(check_transition(State::Open, State::Open).is_ok());
    }

    #[test]
    fn the_week_runs_monday_to_sunday() {
        let week = week_of(day(2026, 10, 14));
        assert_eq!(week[0], day(2026, 10, 12));
        assert_eq!(week[6], day(2026, 10, 18));
        assert_eq!(
            week_of(day(2026, 10, 18))[0],
            day(2026, 10, 12),
            "Sunday ends the week"
        );
    }

    #[test]
    fn only_open_theirs_promises_are_waiting_on_others() {
        assert!(is_waiting_on_others(State::Open, Some(Direction::Theirs)));
        assert!(!is_waiting_on_others(State::Done, Some(Direction::Theirs)));
        assert!(!is_waiting_on_others(State::Open, Some(Direction::Mine)));
    }
}
