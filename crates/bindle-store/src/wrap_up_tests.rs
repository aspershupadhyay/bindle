// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

use super::*;
use crate::lifecycle::{self, Sorting};
use crate::open_in_memory;
use crate::promises::{jot_down, Jot};
use bindle_core::{State, Strength, When};
use chrono::{NaiveTime, TimeZone};

fn at(h: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, h, 0, 0).unwrap()
}

fn oct(d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, d).unwrap()
}

fn kept(
    conn: &mut Connection,
    title: &str,
    day: u32,
    time: Option<u32>,
    direction: Direction,
) -> String {
    let id = jot_down(conn, &Jot::plain(title), at(8)).expect("jot").id;
    let sorting = Sorting {
        title: title.into(),
        direction: Some(direction),
        strength: Strength::Firm,
        person: None,
        first_step: None,
        when: Some(When {
            day: oct(day),
            time: time.map(|h| NaiveTime::from_hms_opt(h, 0, 0).unwrap()),
        }),
        is_deadline: false,
    };
    lifecycle::sort(conn, &id, &sorting, State::Open, at(8)).expect("keep");
    id
}

fn read(conn: &Connection) -> WrapUpView {
    view(
        conn,
        oct(14),
        Utc.with_ymd_and_hms(2026, 10, 14, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 10, 15, 0, 0, 0).unwrap(),
    )
    .expect("view")
}

#[test]
fn wrap_up_shows_what_was_kept_what_is_open_and_tomorrow() {
    let mut conn = open_in_memory().expect("open");
    let done = kept(&mut conn, "Pay bill", 14, None, Direction::Mine);
    lifecycle::mark_done(&mut conn, &done, at(12)).expect("done");
    kept(&mut conn, "Reply to Priya", 13, None, Direction::Mine);
    kept(&mut conn, "Call mom", 14, None, Direction::SelfPromise);
    kept(&mut conn, "Rohan pays", 14, None, Direction::Theirs);
    kept(&mut conn, "Gym", 15, None, Direction::Mine);
    let view = read(&conn);
    let titles = |list: &[Promise]| list.iter().map(|p| p.title.clone()).collect::<Vec<_>>();
    assert_eq!(titles(&view.kept_today), ["Pay bill"]);
    assert_eq!(titles(&view.still_open), ["Reply to Priya", "Call mom"]);
    assert_eq!(titles(&view.tomorrow), ["Gym"]);
}

#[test]
fn finishing_marks_the_day_and_keeps_mind_load_in_range() {
    let conn = open_in_memory().expect("open");
    assert!(!is_wrapped_up(&conn, oct(14)).expect("read"));
    finish(&conn, oct(14), Some(9), Some("  "), at(21)).expect("finish");
    assert!(is_wrapped_up(&conn, oct(14)).expect("read"));
    let view = read(&conn);
    assert_eq!(view.mind_load, None, "9 is not a 1 to 5 answer");
    finish(&conn, oct(14), Some(2), Some("calm day"), at(21)).expect("again");
    assert_eq!(read(&conn).mind_load, Some(2));
}

#[test]
fn only_your_open_promises_with_a_time_today_get_reminders() {
    let mut conn = open_in_memory().expect("open");
    kept(&mut conn, "Call Aman", 14, Some(16), Direction::Mine);
    kept(&mut conn, "No time", 14, None, Direction::Mine);
    kept(&mut conn, "Tomorrow", 15, Some(9), Direction::Mine);
    kept(&mut conn, "Theirs", 14, Some(10), Direction::Theirs);
    let timed = timed_on(&conn, oct(14)).expect("timed");
    assert_eq!(timed.len(), 1);
    assert_eq!(timed[0].title, "Call Aman");
    assert_eq!(timed[0].at, oct(14).and_hms_opt(16, 0, 0).unwrap());
}

#[test]
fn a_time_typed_in_jot_down_reminds_before_it_is_sorted() {
    let mut conn = open_in_memory().expect("open");
    let when = When {
        day: oct(14),
        time: NaiveTime::from_hms_opt(18, 34, 0),
    };
    jot_down(
        &mut conn,
        &Jot {
            when: Some(when),
            ..Jot::plain("call Aman")
        },
        at(8),
    )
    .expect("jot");
    assert_eq!(timed_on(&conn, oct(14)).expect("timed").len(), 1);
}

#[test]
fn sent_notifications_are_remembered_per_day() {
    let conn = open_in_memory().expect("open");
    record_sent(&conn, "wrap_up:2026-10-14", oct(14), at(21)).expect("record");
    record_sent(&conn, "wrap_up:2026-10-14", oct(14), at(21)).expect("twice is fine");
    assert_eq!(
        sent_on(&conn, oct(14)).expect("sent"),
        ["wrap_up:2026-10-14"]
    );
    assert!(sent_on(&conn, oct(15)).expect("sent").is_empty());
}
