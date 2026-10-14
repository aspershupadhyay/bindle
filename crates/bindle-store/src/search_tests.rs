// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

use super::*;
use crate::lifecycle::{self, Sorting};
use crate::open_in_memory;
use crate::promises::{jot_down, Jot};
use bindle_core::{Direction, State, Strength, When};
use chrono::{NaiveDate, TimeZone, Utc};
use std::time::{Duration, Instant};

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

fn titles(found: &[Promise]) -> Vec<&str> {
    found.iter().map(|p| p.title.as_str()).collect()
}

#[test]
fn words_match_from_their_start_in_any_case() {
    let mut conn = open_in_memory().expect("open");
    jot_down(&mut conn, &Jot::plain("Pay Electricity bill"), now()).expect("jot");
    jot_down(&mut conn, &Jot::plain("Call mom"), now()).expect("jot");
    assert_eq!(
        titles(&search(&conn, "elec", 10).expect("search")),
        ["Pay Electricity bill"]
    );
    assert_eq!(
        titles(&search(&conn, "PAY BILL", 10).expect("search")),
        ["Pay Electricity bill"]
    );
    assert!(
        search(&conn, "lectric", 10).expect("search").is_empty(),
        "only from the start of a word"
    );
}

#[test]
fn the_person_and_first_step_are_searched_too() {
    let mut conn = open_in_memory().expect("open");
    let jot = Jot {
        person: Some("Rohan".into()),
        ..Jot::plain("Send the lesson plan")
    };
    let id = jot_down(&mut conn, &jot, now()).expect("jot").id;
    assert_eq!(search(&conn, "rohan", 10).expect("search").len(), 1);
    let sorting = Sorting {
        title: "Send the lesson plan".into(),
        direction: Some(Direction::Mine),
        strength: Strength::Firm,
        person: Some("Rohan".into()),
        first_step: Some("Export the slides".into()),
        when: Some(When {
            day: NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
            time: None,
        }),
        is_deadline: false,
    };
    lifecycle::sort(&mut conn, &id, &sorting, State::Open, now()).expect("sort");
    assert_eq!(search(&conn, "slides", 10).expect("search").len(), 1);
}

#[test]
fn renaming_a_person_updates_what_search_finds() {
    let mut conn = open_in_memory().expect("open");
    let jot = Jot {
        person: Some("Wifey".into()),
        ..Jot::plain("Book train tickets")
    };
    jot_down(&mut conn, &jot, now()).expect("jot");
    conn.execute(
        "UPDATE person SET display_name = 'Ananya' WHERE display_name = 'Wifey'",
        [],
    )
    .expect("rename");
    assert_eq!(search(&conn, "ananya", 10).expect("search").len(), 1);
    assert!(search(&conn, "wifey", 10).expect("search").is_empty());
}

#[test]
fn undo_keeps_search_in_step() {
    let mut conn = open_in_memory().expect("open");
    let id = jot_down(&mut conn, &Jot::plain("pay bill"), now())
        .expect("jot")
        .id;
    let sorting = Sorting {
        title: "Pay electricity bill".into(),
        direction: Some(Direction::Mine),
        strength: Strength::Soft,
        person: None,
        first_step: None,
        when: None,
        is_deadline: false,
    };
    let change = lifecycle::sort(&mut conn, &id, &sorting, State::Open, now()).expect("sort");
    assert_eq!(search(&conn, "electricity", 10).expect("search").len(), 1);
    lifecycle::undo(&mut conn, &change.event_id, now()).expect("undo");
    assert!(search(&conn, "electricity", 10).expect("search").is_empty());
    assert_eq!(search(&conn, "pay", 10).expect("search").len(), 1);
}

#[test]
fn symbols_are_words_not_query_syntax() {
    let mut conn = open_in_memory().expect("open");
    jot_down(
        &mut conn,
        &Jot::plain("Post Case 018 carousel on @catchtheerror"),
        now(),
    )
    .expect("jot");
    assert_eq!(search(&conn, "@catch", 10).expect("search").len(), 1);
    for odd in ["\"", "*", "AND", "NOT", "(", "a:b", "  ", ""] {
        search(&conn, odd, 10).unwrap_or_else(|e| panic!("{odd:?} failed: {e}"));
    }
}

#[test]
fn hindi_words_are_found_too() {
    let mut conn = open_in_memory().expect("open");
    jot_down(&mut conn, &Jot::plain("मंगलवार को बैठक"), now()).expect("jot");
    assert_eq!(search(&conn, "बैठक", 10).expect("search").len(), 1);
}

/// The week 5 gate: search under 100 ms on 10,000 promises.
#[test]
fn search_takes_under_100_ms_on_10000_promises() {
    let mut conn = open_in_memory().expect("open");
    let words = [
        "pay", "call", "send", "book", "renew", "reply", "fix", "plan", "buy", "meet",
    ];
    let things = [
        "bill",
        "mom",
        "invoice",
        "tickets",
        "insurance",
        "Priya",
        "bike",
        "trip",
        "milk",
        "Rohan",
    ];
    let tx = conn.transaction().expect("tx");
    for i in 0..10_000 {
        tx.execute(
            "INSERT INTO promise (id, title, state, created_at, updated_at) VALUES (?1, ?2, 'open', ?3, ?3)",
            params![format!("p{i}"), format!("{} {} number {i}", words[i % 10], things[(i / 10) % 10]), "2026-10-14"],
        )
        .expect("insert");
    }
    tx.execute("INSERT INTO promise (id, title, created_at, updated_at) VALUES ('needle', 'Pay electricity bill', 'x', 'x')", [])
        .expect("insert");
    tx.commit().expect("commit");

    let mut slowest = Duration::ZERO;
    for query in ["electricity", "pay bill", "rohan", "p", "number 9999"] {
        let start = Instant::now();
        let found = search(&conn, query, 50).expect("search");
        slowest = slowest.max(start.elapsed());
        assert!(!found.is_empty(), "{query} should find something");
    }
    assert!(
        slowest < Duration::from_millis(100),
        "slowest search took {slowest:?}"
    );
}
