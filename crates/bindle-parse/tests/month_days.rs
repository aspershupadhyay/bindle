// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Days of the month said without a month: "rent on the 1st", "by 5th",
//! "15th tak". "Now" is Wednesday 14 October 2026, 6:30 PM, as in the table.

use bindle_parse::read;
use chrono::{NaiveDate, NaiveTime};

fn reads(text: &str) -> (String, Option<NaiveDate>, Option<NaiveTime>, bool) {
    let now = NaiveDate::from_ymd_opt(2026, 10, 14)
        .unwrap()
        .and_hms_opt(18, 30, 0)
        .unwrap();
    let r = read(text, now, &[]);
    (
        r.title,
        r.when.map(|w| w.day),
        r.when.and_then(|w| w.time),
        r.is_deadline,
    )
}

fn day(month: u32, day: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(2026, month, day)
}

#[test]
fn a_day_still_ahead_is_this_month_and_a_past_one_next_month() {
    assert_eq!(reads("pay rent on the 1st").1, day(11, 1));
    assert_eq!(reads("pay rent on 1st").1, day(11, 1));
    assert_eq!(reads("pay rent on the 20th").1, day(10, 20));
    assert_eq!(
        reads("call the bank on 14th").1,
        day(10, 14),
        "today counts"
    );
}

#[test]
fn the_words_around_it_leave_the_title() {
    assert_eq!(reads("pay rent on the 1st").0, "Pay rent");
    let (title, when, _, deadline) = reads("file GST by the 20th");
    assert_eq!(
        (title.as_str(), when, deadline),
        ("File GST", day(10, 20), true)
    );
    let (title, when, _, deadline) = reads("15th tak invoice bhejna");
    assert_eq!(
        (title.as_str(), when, deadline),
        ("Invoice bhejna", day(10, 15), true)
    );
}

#[test]
fn a_time_after_it_is_read_too() {
    let (_, when, time, _) = reads("dentist on the 3rd at 5pm");
    assert_eq!(when, day(11, 3));
    assert_eq!(time, NaiveTime::from_hms_opt(17, 0, 0));
}

#[test]
fn a_month_without_that_day_is_skipped() {
    // November has no 31st, so the next 31st is in December.
    let now = NaiveDate::from_ymd_opt(2026, 11, 2)
        .unwrap()
        .and_hms_opt(9, 0, 0)
        .unwrap();
    let r = read("pay the loan on the 31st", now, &[]);
    assert_eq!(r.when.map(|w| w.day), day(12, 31));
}

#[test]
fn ordinals_that_are_not_days_are_left_alone() {
    for text in [
        "send the 1st draft to Priya",
        "fix the leak on the 2nd floor",
        "she came 1st in class",
        "pay rent on 1",
    ] {
        assert_eq!(reads(text).1, None, "{text:?} has no day");
    }
}
