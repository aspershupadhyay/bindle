// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

use super::*;
use crate::open_in_memory;
use crate::promises::{jot_down, list_by_state, sort_queue, Jot};
use bindle_core::SortError;
use chrono::{NaiveDate, TimeZone};

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 18, 30, 0).unwrap()
}

fn later(minutes: i64) -> DateTime<Utc> {
    now() + chrono::Duration::minutes(minutes)
}

fn friday() -> When {
    When {
        day: NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
        time: None,
    }
}

fn keep(title: &str) -> Sorting {
    Sorting {
        title: title.into(),
        direction: Some(Direction::Mine),
        strength: Strength::Firm,
        person: None,
        first_step: Some("Open the electricity app".into()),
        when: Some(friday()),
        is_deadline: false,
    }
}

fn jotted(conn: &mut Connection, text: &str) -> String {
    jot_down(conn, &Jot::plain(text), now()).expect("jot").id
}

fn kinds(conn: &Connection, id: &str) -> Vec<String> {
    history(conn, id)
        .expect("history")
        .into_iter()
        .map(|e| e.kind)
        .collect()
}

#[test]
fn keeping_a_promise_opens_it_with_its_first_step_and_day() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "pay bill");
    let change = sort(
        &mut conn,
        &id,
        &keep("Pay electricity bill"),
        State::Open,
        later(1),
    )
    .expect("sort");
    assert_eq!(change.promise.state, State::Open);
    assert_eq!(change.promise.title, "Pay electricity bill");
    assert_eq!(
        change.promise.first_step.as_deref(),
        Some("Open the electricity app")
    );
    assert_eq!(change.promise.do_on, Some(friday()));
    assert_eq!(kinds(&conn, &id), ["created", "sorted"]);
}

#[test]
fn a_promise_without_a_day_cannot_be_kept_unless_it_is_soft() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "coffee with Aman");
    let no_day = Sorting {
        when: None,
        ..keep("Coffee with Aman")
    };
    let refused = sort(&mut conn, &id, &no_day, State::Open, later(1));
    assert!(matches!(
        refused,
        Err(StoreError::NotSorted(SortError::NoDay))
    ));
    assert_eq!(
        kinds(&conn, &id),
        ["created"],
        "a refused sort writes nothing"
    );
    let soft = Sorting {
        strength: Strength::Soft,
        ..no_day
    };
    assert!(sort(&mut conn, &id, &soft, State::Open, later(1)).is_ok());
}

#[test]
fn later_and_let_go_need_no_day() {
    let mut conn = open_in_memory().expect("open");
    let a = jotted(&mut conn, "learn guitar");
    let b = jotted(&mut conn, "old idea");
    let loose = Sorting {
        when: None,
        direction: None,
        ..keep("Learn guitar")
    };
    assert_eq!(
        sort(&mut conn, &a, &loose, State::Later, later(1))
            .expect("later")
            .promise
            .state,
        State::Later
    );
    let gone = sort(
        &mut conn,
        &b,
        &Sorting {
            title: "Old idea".into(),
            ..loose
        },
        State::LetGo,
        later(2),
    )
    .expect("let go");
    assert_eq!(gone.promise.state, State::LetGo);
}

#[test]
fn sorting_with_a_new_name_adds_the_person() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "invoice");
    let theirs = Sorting {
        person: Some("Rohan".into()),
        direction: Some(Direction::Theirs),
        ..keep("Pay the invoice")
    };
    let change = sort(&mut conn, &id, &theirs, State::Open, later(1)).expect("sort");
    assert_eq!(change.promise.person.as_deref(), Some("Rohan"));
}

#[test]
fn done_and_reopen_follow_the_lifecycle() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "pay bill");
    sort(&mut conn, &id, &keep("Pay bill"), State::Open, later(1)).expect("sort");
    assert_eq!(
        mark_done(&mut conn, &id, later(2))
            .expect("done")
            .promise
            .state,
        State::Done
    );
    assert_eq!(
        reopen(&mut conn, &id, later(3))
            .expect("reopen")
            .promise
            .state,
        State::Open
    );
    assert_eq!(kinds(&conn, &id), ["created", "sorted", "done", "reopened"]);
}

#[test]
fn an_unsorted_promise_cannot_be_marked_done() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "pay bill");
    assert!(matches!(
        mark_done(&mut conn, &id, later(1)),
        Err(StoreError::NotAllowed(_))
    ));
}

#[test]
fn moving_a_promise_changes_its_day_and_keeps_it_open() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "pay bill");
    sort(&mut conn, &id, &keep("Pay bill"), State::Open, later(1)).expect("sort");
    let monday = When {
        day: NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(),
        time: None,
    };
    let moved = move_to(&mut conn, &id, monday, true, later(2)).expect("move");
    assert_eq!(moved.promise.due, Some(monday));
    assert_eq!(moved.promise.do_on, None);
    assert_eq!(moved.promise.state, State::Open);
}

#[test]
fn undo_puts_back_exactly_what_was_there() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "pay bill");
    let before = crate::promises::get(&conn, &id).expect("read");
    let change = sort(
        &mut conn,
        &id,
        &keep("Pay electricity bill"),
        State::Open,
        later(1),
    )
    .expect("sort");
    let undone = undo(&mut conn, &change.event_id, later(2)).expect("undo");
    assert_eq!(undone.promise.state, State::Unsorted);
    assert_eq!(undone.promise.title, before.title);
    assert_eq!(undone.promise.do_on, None);
    assert_eq!(sort_queue(&conn).expect("queue").len(), 1, "back in Sort");
    let entries = history(&conn, &id).expect("history");
    assert!(entries[1].undone, "the sort is marked as undone");
    assert_eq!(entries[2].kind, "undone");
}

#[test]
fn the_same_change_cannot_be_undone_twice() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "pay bill");
    let change = sort(&mut conn, &id, &keep("Pay bill"), State::Open, later(1)).expect("sort");
    undo(&mut conn, &change.event_id, later(2)).expect("undo");
    assert!(matches!(
        undo(&mut conn, &change.event_id, later(3)),
        Err(StoreError::NothingToUndo)
    ));
}

#[test]
fn undo_is_refused_when_the_promise_changed_since() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "pay bill");
    let sorted = sort(&mut conn, &id, &keep("Pay bill"), State::Open, later(1)).expect("sort");
    mark_done(&mut conn, &id, later(2)).expect("done");
    assert!(matches!(
        undo(&mut conn, &sorted.event_id, later(3)),
        Err(StoreError::ChangedSince)
    ));
    assert_eq!(
        crate::promises::get(&conn, &id).expect("read").state,
        State::Done
    );
}

#[test]
fn undo_last_walks_back_one_change_at_a_time() {
    let mut conn = open_in_memory().expect("open");
    let id = jotted(&mut conn, "pay bill");
    sort(&mut conn, &id, &keep("Pay bill"), State::Open, later(1)).expect("sort");
    mark_done(&mut conn, &id, later(2)).expect("done");
    let first = undo_last(&mut conn, later(3))
        .expect("undo")
        .expect("a change");
    assert_eq!(first.promise.state, State::Open);
    let second = undo_last(&mut conn, later(4))
        .expect("undo")
        .expect("a change");
    assert_eq!(second.promise.state, State::Unsorted);
    assert_eq!(
        undo_last(&mut conn, later(5)).expect("undo"),
        None,
        "creating is not undone"
    );
}

#[test]
fn the_sort_queue_shows_what_was_typed_and_what_to_check() {
    let mut conn = open_in_memory().expect("open");
    let jot = Jot {
        title: "Call mom".into(),
        unsure_words: Some("kal".into()),
        ..Jot::plain("call mom kal")
    };
    jot_down(&mut conn, &jot, now()).expect("jot");
    let queue = sort_queue(&conn).expect("queue");
    assert_eq!(queue[0].typed.as_deref(), Some("call mom kal"));
    assert_eq!(queue[0].unsure_words.as_deref(), Some("kal"));
    let id = queue[0].promise.id.clone();
    sort(&mut conn, &id, &keep("Call mom"), State::Open, later(1)).expect("sort");
    assert!(sort_queue(&conn).expect("queue").is_empty());
    assert_eq!(list_by_state(&conn, State::Open).expect("open").len(), 1);
}
