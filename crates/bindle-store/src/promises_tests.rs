// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

use super::*;
use crate::open_in_memory;
use chrono::TimeZone;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 18, 30, 0).unwrap()
}

fn friday(time: Option<NaiveTime>) -> When {
    When {
        day: NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
        time,
    }
}

#[test]
fn a_jotted_down_promise_waits_in_sort() {
    let mut conn = open_in_memory().expect("open");
    let promise = jot_down(&mut conn, &Jot::plain("  Pay electricity bill "), now()).expect("save");
    assert_eq!(promise.title, "Pay electricity bill");
    assert_eq!(promise.state, State::Unsorted);
    assert_eq!(promise.mode, Mode::Life);
    assert_eq!(count_by_state(&conn, State::Unsorted).expect("count"), 1);
}

#[test]
fn jotting_down_records_a_created_event_and_keeps_the_typed_text() {
    let mut conn = open_in_memory().expect("open");
    let jot = Jot {
        title: "Call mom".into(),
        ..Jot::plain("call mom kal")
    };
    let promise = jot_down(&mut conn, &jot, now()).expect("save");
    let (kind, snippet): (String, String) = conn
        .query_row(
            "SELECT e.kind, s.snippet FROM promise p
             JOIN promise_event e ON e.promise_id = p.id JOIN source s ON s.id = p.source_id
             WHERE p.id = ?1",
            [&promise.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("event and source");
    assert_eq!(kind, "created");
    assert_eq!(snippet, "call mom kal");
}

#[test]
fn unsure_words_are_kept_for_sort_to_ask_about() {
    let mut conn = open_in_memory().expect("open");
    let jot = Jot {
        unsure_words: Some("kal".into()),
        ..Jot::plain("Call mom kal")
    };
    let promise = jot_down(&mut conn, &jot, now()).expect("save");
    let data: String = conn
        .query_row(
            "SELECT data FROM promise_event WHERE promise_id = ?1",
            [&promise.id],
            |r| r.get(0),
        )
        .expect("event");
    assert_eq!(data, r#"{"unsure_words":"kal"}"#);
}

#[test]
fn an_empty_jot_is_refused_and_nothing_is_saved() {
    let mut conn = open_in_memory().expect("open");
    assert!(matches!(
        jot_down(&mut conn, &Jot::plain("   "), now()),
        Err(StoreError::EmptyTitle)
    ));
    assert_eq!(count_by_state(&conn, State::Unsorted).expect("count"), 0);
    let sources: u32 = conn
        .query_row("SELECT COUNT(*) FROM source", [], |r| r.get(0))
        .expect("count");
    assert_eq!(sources, 0);
}

#[test]
fn promises_are_listed_oldest_first() {
    let mut conn = open_in_memory().expect("open");
    jot_down(&mut conn, &Jot::plain("First"), now()).expect("save");
    jot_down(
        &mut conn,
        &Jot::plain("Second"),
        now() + chrono::Duration::minutes(1),
    )
    .expect("save");
    let titles: Vec<_> = list_by_state(&conn, State::Unsorted)
        .expect("list")
        .into_iter()
        .map(|p| p.title)
        .collect();
    assert_eq!(titles, ["First", "Second"]);
}

#[test]
fn a_planned_day_and_time_read_back_the_same() {
    let mut conn = open_in_memory().expect("open");
    let when = friday(NaiveTime::from_hms_opt(20, 30, 0));
    let jot = Jot {
        when: Some(when),
        ..Jot::plain("Gym")
    };
    let promise = jot_down(&mut conn, &jot, now()).expect("save");
    assert_eq!(promise.do_on, Some(when));
    assert_eq!(promise.due, None);
}

#[test]
fn a_deadline_is_saved_as_due() {
    let mut conn = open_in_memory().expect("open");
    let jot = Jot {
        when: Some(friday(None)),
        is_deadline: true,
        ..Jot::plain("Submit report")
    };
    let promise = jot_down(&mut conn, &jot, now()).expect("save");
    assert_eq!(promise.due, Some(friday(None)));
    assert_eq!(promise.do_on, None);
}

#[test]
fn amount_person_and_direction_are_kept() {
    let mut conn = open_in_memory().expect("open");
    let jot = Jot {
        amount: Some(Money {
            minor: 1_800_000,
            currency: "INR".into(),
        }),
        person: Some("Rohan".into()),
        direction: Some(Direction::Theirs),
        ..Jot::plain("Rohan will pay the invoice")
    };
    let promise = jot_down(&mut conn, &jot, now()).expect("save");
    assert_eq!(promise.amount, jot.amount);
    assert_eq!(promise.person.as_deref(), Some("Rohan"));
    assert_eq!(promise.direction, Some(Direction::Theirs));
}

#[test]
fn two_jots_about_the_same_person_share_one_person() {
    let mut conn = open_in_memory().expect("open");
    for text in ["Call Rohan", "Pay rohan"] {
        let name = text.split(' ').nth(1).map(str::to_string);
        jot_down(
            &mut conn,
            &Jot {
                person: name,
                ..Jot::plain(text)
            },
            now(),
        )
        .expect("save");
    }
    assert_eq!(people::names(&conn).expect("names"), ["Rohan"]);
}

#[test]
fn stored_days_and_words_read_back_as_core_types() {
    let mut conn = open_in_memory().expect("open");
    let promise = jot_down(&mut conn, &Jot::plain("Book Diwali tickets"), now()).expect("save");
    conn.execute(
        "UPDATE promise SET state = 'open', direction = 'self', due = '2026-10-17' WHERE id = ?1",
        [&promise.id],
    )
    .expect("update");
    let read = get(&conn, &promise.id).expect("read");
    assert_eq!(read.direction, Some(Direction::SelfPromise));
    assert_eq!(
        read.due.map(|w| w.day),
        NaiveDate::from_ymd_opt(2026, 10, 17)
    );
}

#[test]
fn counts_cover_every_state_in_one_pass() {
    let mut conn = open_in_memory().expect("open");
    jot_down(&mut conn, &Jot::plain("One"), now()).expect("save");
    jot_down(&mut conn, &Jot::plain("Two"), now()).expect("save");
    let counts = state_counts(&conn).expect("counts");
    assert_eq!(
        counts,
        StateCounts {
            unsorted: 2,
            ..StateCounts::default()
        }
    );
}

#[test]
fn open_promises_list_soonest_first_with_no_day_last() {
    let mut conn = open_in_memory().expect("open");
    for (title, day) in [
        ("No day", None),
        ("Friday", Some(16)),
        ("Thursday", Some(15)),
    ] {
        let when = day.map(|d| When {
            day: NaiveDate::from_ymd_opt(2026, 10, d).unwrap(),
            time: None,
        });
        let id = jot_down(
            &mut conn,
            &Jot {
                when,
                ..Jot::plain(title)
            },
            now(),
        )
        .expect("save")
        .id;
        conn.execute("UPDATE promise SET state = 'open' WHERE id = ?1", [&id])
            .expect("open");
    }
    let titles: Vec<_> = list_for_screen(&conn, State::Open, 10)
        .expect("list")
        .into_iter()
        .map(|p| p.title)
        .collect();
    assert_eq!(titles, ["Thursday", "Friday", "No day"]);
}
