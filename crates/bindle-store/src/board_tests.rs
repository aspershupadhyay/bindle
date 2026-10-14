// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Board checks: which lane each work promise lands in.

use super::*;
use crate::lifecycle::{self, Sorting};
use crate::open_in_memory;
use crate::promises::{jot_down_in, Jot};
use bindle_core::{Direction, Mode, State, Strength, When};
use chrono::{NaiveDate, TimeZone};

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

fn monday() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 12, 0, 0, 0).unwrap()
}

/// A work promise sorted into `to`, owed by `direction`.
fn work(conn: &mut Connection, title: &str, direction: Direction, to: State) -> String {
    let promise = jot_down_in(conn, &Jot::plain(title), Mode::Work, now()).unwrap();
    let sorting = Sorting {
        title: title.into(),
        direction: Some(direction),
        strength: Strength::Firm,
        person: (direction == Direction::Theirs).then(|| "Neha".into()),
        first_step: None,
        when: Some(When {
            day: NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
            time: None,
        }),
        is_deadline: false,
    };
    lifecycle::sort(conn, &promise.id, &sorting, to, now()).unwrap();
    promise.id
}

fn titles(cards: &[Card]) -> Vec<&str> {
    cards.iter().map(|c| c.promise.title.as_str()).collect()
}

#[test]
fn each_work_promise_has_one_lane() {
    let mut conn = open_in_memory().unwrap();
    work(
        &mut conn,
        "Case 020: pick a myth",
        Direction::Mine,
        State::Later,
    );
    work(&mut conn, "Review latch #42", Direction::Mine, State::Open);
    let making = work(&mut conn, "Fix token refresh", Direction::Mine, State::Open);
    work(
        &mut conn,
        "Curl timings from Neha",
        Direction::Theirs,
        State::Open,
    );
    let shipped = work(
        &mut conn,
        "Latch share sheet v2",
        Direction::Mine,
        State::Open,
    );
    // Life promises stay off the Board.
    jot_down_in(&mut conn, &Jot::plain("Call Mum"), Mode::Life, now()).unwrap();

    set_making(&conn, &making, true).unwrap();
    lifecycle::mark_done(&mut conn, &shipped, now()).unwrap();

    let board = board_view(&conn, monday()).unwrap();
    assert_eq!(titles(&board.ideas), ["Case 020: pick a myth"]);
    assert_eq!(titles(&board.next), ["Review latch #42"]);
    assert_eq!(titles(&board.making), ["Fix token refresh"]);
    assert_eq!(titles(&board.waiting), ["Curl timings from Neha"]);
    assert_eq!(titles(&board.shipped), ["Latch share sheet v2"]);

    // Back out of Making, it returns to Next.
    set_making(&conn, &making, false).unwrap();
    let board = board_view(&conn, monday()).unwrap();
    assert!(board.making.is_empty());
    assert_eq!(board.next.len(), 2);
}

#[test]
fn only_open_promises_can_be_making() {
    let mut conn = open_in_memory().unwrap();
    let idea = work(&mut conn, "Someday", Direction::Mine, State::Later);
    assert!(matches!(
        set_making(&conn, &idea, true),
        Err(StoreError::NotForMaking)
    ));
    assert!(matches!(
        set_making(&conn, "nope", true),
        Err(StoreError::NotFound)
    ));
}

#[test]
fn shipped_counts_only_this_week() {
    let mut conn = open_in_memory().unwrap();
    let id = work(&mut conn, "Last week's thing", Direction::Mine, State::Open);
    lifecycle::mark_done(&mut conn, &id, monday() - chrono::Duration::days(2)).unwrap();
    assert!(board_view(&conn, monday()).unwrap().shipped.is_empty());
}

#[test]
fn a_card_says_where_it_came_from() {
    let mut conn = open_in_memory().unwrap();
    crate::work::sync(
        &mut conn,
        "github",
        &[crate::work::FoundItem {
            id: "r1".into(),
            kind: crate::work::WorkKind::Review,
            repo: "kunal-m/latch".into(),
            number: 42,
            title: "Permission prompts".into(),
            author: "kunal-m".into(),
            url: "https://github.com/kunal-m/latch/pull/42".into(),
            draft: false,
            labels: vec![],
            created_at: "2026-10-12T10:00:00Z".into(),
            updated_at: "2026-10-12T10:00:00Z".into(),
        }],
        now(),
    )
    .unwrap();
    let promise = crate::work::make_promise(&mut conn, "r1", now()).unwrap();
    lifecycle::sort(
        &mut conn,
        &promise.id,
        &Sorting {
            title: promise.title.clone(),
            direction: Some(Direction::Mine),
            strength: Strength::Firm,
            person: Some("kunal-m".into()),
            first_step: None,
            when: Some(When {
                day: NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(),
                time: None,
            }),
            is_deadline: false,
        },
        State::Open,
        now(),
    )
    .unwrap();
    let board = board_view(&conn, monday()).unwrap();
    assert_eq!(
        board.next[0].from.as_deref(),
        Some("GitHub kunal-m/latch#42")
    );
}
