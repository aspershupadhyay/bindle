// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Home checks: what shows where, for Wednesday 14 October 2026.

use super::*;
use crate::lifecycle::{self, Sorting};
use crate::open_in_memory;
use crate::promises::{jot_down, Jot};
use bindle_core::{Strength, When};
use chrono::TimeZone;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

fn monday_start() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 12, 0, 0, 0).unwrap()
}

fn oct(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
}

fn today() -> NaiveDate {
    oct(14)
}

/// Jots down and keeps a promise in one go.
fn kept(
    conn: &mut Connection,
    title: &str,
    direction: Direction,
    day: Option<u32>,
    person: Option<&str>,
) -> String {
    let id = jot_down(conn, &Jot::plain(title), now()).expect("jot").id;
    let sorting = Sorting {
        title: title.into(),
        direction: Some(direction),
        strength: if day.is_some() {
            Strength::Firm
        } else {
            Strength::Soft
        },
        person: person.map(Into::into),
        first_step: None,
        when: day.map(|d| When {
            day: oct(d),
            time: None,
        }),
        is_deadline: false,
    };
    lifecycle::sort(conn, &id, &sorting, State::Open, now()).expect("keep");
    id
}

fn titles(list: &[Promise]) -> Vec<&str> {
    list.iter().map(|p| p.title.as_str()).collect()
}

fn read(conn: &Connection) -> HomeView {
    home(conn, today(), monday_start()).expect("home")
}

#[test]
fn your_promises_past_their_day_are_running_late_oldest_first() {
    let mut conn = open_in_memory().expect("open");
    kept(
        &mut conn,
        "Reply to Priya",
        Direction::Mine,
        Some(13),
        Some("Priya"),
    );
    kept(
        &mut conn,
        "Renew insurance",
        Direction::Mine,
        Some(10),
        None,
    );
    kept(&mut conn, "Pay bill", Direction::Mine, Some(14), None);
    assert_eq!(
        titles(&read(&conn).running_late),
        ["Renew insurance", "Reply to Priya"]
    );
}

#[test]
fn soft_promises_and_theirs_are_never_running_late() {
    let mut conn = open_in_memory().expect("open");
    kept(&mut conn, "Coffee with Aman", Direction::Mine, None, None);
    kept(
        &mut conn,
        "Rohan pays invoice",
        Direction::Theirs,
        Some(2),
        Some("Rohan"),
    );
    let view = read(&conn);
    assert!(view.running_late.is_empty());
    assert_eq!(view.waiting_on_others.len(), 1);
}

#[test]
fn waiting_on_others_is_oldest_first_and_remembers_nudges() {
    let mut conn = open_in_memory().expect("open");
    let rohan = kept(
        &mut conn,
        "Invoice",
        Direction::Theirs,
        Some(16),
        Some("Rohan"),
    );
    kept(
        &mut conn,
        "Thumbnails",
        Direction::Theirs,
        Some(15),
        Some("Aman"),
    );
    nudge(&conn, &rohan, now()).expect("nudge");
    let view = read(&conn);
    assert_eq!(view.waiting_on_others[0].promise.title, "Invoice");
    assert!(view.waiting_on_others[0].last_nudged.is_some());
    assert_eq!(view.waiting_on_others[1].last_nudged, None);
}

#[test]
fn only_someone_elses_open_promise_can_be_nudged() {
    let mut conn = open_in_memory().expect("open");
    let mine = kept(&mut conn, "Pay bill", Direction::Mine, Some(14), None);
    assert!(matches!(
        nudge(&conn, &mine, now()),
        Err(StoreError::NotNudgeable)
    ));
}

#[test]
fn the_top_3_holds_three_in_order_and_closes_gaps() {
    let mut conn = open_in_memory().expect("open");
    let ids: Vec<String> = ["A", "B", "C", "D"]
        .iter()
        .map(|t| kept(&mut conn, t, Direction::Mine, Some(14), None))
        .collect();
    for id in &ids[..3] {
        add_to_top3(&mut conn, today(), id, now()).expect("add");
    }
    assert!(matches!(
        add_to_top3(&mut conn, today(), &ids[3], now()),
        Err(StoreError::TopThreeFull)
    ));
    remove_from_top3(&mut conn, today(), &ids[0], now()).expect("remove");
    add_to_top3(&mut conn, today(), &ids[3], now()).expect("room again");
    assert_eq!(titles(&read(&conn).top3), ["B", "C", "D"]);
}

#[test]
fn a_promise_in_the_top_3_shows_only_there() {
    let mut conn = open_in_memory().expect("open");
    let late = kept(&mut conn, "Late one", Direction::Mine, Some(12), None);
    add_to_top3(&mut conn, today(), &late, now()).expect("add");
    let view = read(&conn);
    assert_eq!(titles(&view.top3), ["Late one"]);
    assert!(view.running_late.is_empty());
    assert!(view.coming_up.is_empty());
}

#[test]
fn done_promises_stay_in_the_top_3_as_kept() {
    let mut conn = open_in_memory().expect("open");
    let id = kept(
        &mut conn,
        "Send lesson plan",
        Direction::Mine,
        Some(14),
        None,
    );
    add_to_top3(&mut conn, today(), &id, now()).expect("add");
    lifecycle::mark_done(&mut conn, &id, now()).expect("done");
    let view = read(&conn);
    assert_eq!(view.top3[0].state, State::Done);
    assert!(view.top3[0].closed_at.is_some());
}

#[test]
fn a_promise_still_in_sort_cannot_go_in_the_top_3() {
    let mut conn = open_in_memory().expect("open");
    let id = jot_down(&mut conn, &Jot::plain("unsorted"), now())
        .expect("jot")
        .id;
    assert!(matches!(
        add_to_top3(&mut conn, today(), &id, now()),
        Err(StoreError::NotForTopThree)
    ));
}

#[test]
fn suggestions_offer_today_and_late_only_while_there_is_room() {
    let mut conn = open_in_memory().expect("open");
    let a = kept(&mut conn, "Today A", Direction::Mine, Some(14), None);
    kept(&mut conn, "Late B", Direction::Mine, Some(13), None);
    kept(&mut conn, "Friday C", Direction::Mine, Some(16), None);
    assert_eq!(titles(&read(&conn).suggestions), ["Late B", "Today A"]);
    add_to_top3(&mut conn, today(), &a, now()).expect("add");
    assert_eq!(titles(&read(&conn).suggestions), ["Late B"]);
}

#[test]
fn coming_up_is_soonest_first_with_no_day_last() {
    let mut conn = open_in_memory().expect("open");
    kept(&mut conn, "Someday", Direction::Mine, None, None);
    kept(&mut conn, "Friday", Direction::Mine, Some(16), None);
    kept(&mut conn, "Today", Direction::SelfPromise, Some(14), None);
    assert_eq!(
        titles(&read(&conn).coming_up),
        ["Today", "Friday", "Someday"]
    );
}

#[test]
fn the_week_strip_counts_your_promises_each_day_and_marks_full_days() {
    let mut conn = open_in_memory().expect("open");
    for i in 0..5 {
        kept(
            &mut conn,
            &format!("Friday {i}"),
            Direction::Mine,
            Some(16),
            None,
        );
    }
    kept(
        &mut conn,
        "Theirs on Friday",
        Direction::Theirs,
        Some(16),
        Some("Rohan"),
    );
    let done = kept(&mut conn, "Done today", Direction::Mine, Some(14), None);
    lifecycle::mark_done(&mut conn, &done, now()).expect("done");
    let week = read(&conn).week;
    assert_eq!(week.len(), 7);
    assert_eq!(
        (week[4].day, week[4].count, week[4].full),
        (oct(16), 5, true)
    );
    assert_eq!(
        (week[2].count, week[2].full),
        (1, false),
        "done today still filled today"
    );
}

#[test]
fn this_week_counts_ignore_undone_changes() {
    let mut conn = open_in_memory().expect("open");
    let a = kept(&mut conn, "A", Direction::Mine, Some(14), None);
    let b = kept(&mut conn, "B", Direction::Mine, Some(14), None);
    lifecycle::mark_done(&mut conn, &a, now()).expect("done");
    let change = lifecycle::mark_done(&mut conn, &b, now()).expect("done");
    lifecycle::undo(&mut conn, &change.event_id, now()).expect("undo");
    assert_eq!(
        read(&conn).this_week,
        WeekCounts {
            kept: 1,
            moved: 0,
            let_go: 0
        }
    );
}

#[test]
fn the_summary_counts_today_and_the_people_waiting_on_you() {
    let mut conn = open_in_memory().expect("open");
    kept(&mut conn, "Pay bill", Direction::Mine, Some(14), None);
    kept(
        &mut conn,
        "Reply to Priya",
        Direction::Mine,
        Some(13),
        Some("Priya"),
    );
    kept(
        &mut conn,
        "Send deck to Priya",
        Direction::Mine,
        Some(14),
        Some("Priya"),
    );
    kept(
        &mut conn,
        "Send edit to Rohan",
        Direction::Mine,
        Some(14),
        Some("Rohan"),
    );
    kept(
        &mut conn,
        "Rohan pays",
        Direction::Theirs,
        Some(14),
        Some("Rohan"),
    );
    let view = read(&conn);
    assert_eq!(view.today_count, 3);
    assert_eq!(
        today_count(&conn, today()).expect("count"),
        3,
        "the quick count agrees"
    );
    assert_eq!(view.people_waiting_on_you, 2);
}
