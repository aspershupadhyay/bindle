// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

use super::*;
use crate::open_in_memory;
use crate::promises::{jot_down, sort_queue};
use chrono::TimeZone;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

fn allow(conn: &Connection, access: AiAccess) {
    let mut s = settings::load(conn).expect("load");
    s.ai_access = access;
    settings::save(conn, &s).expect("save");
}

#[test]
fn an_ai_can_do_nothing_until_the_person_lets_it() {
    let mut conn = open_in_memory().expect("open");
    assert!(matches!(check_read(&conn), Err(StoreError::AiOff)));
    let jot = Jot::plain("Book dentist");
    assert!(matches!(
        suggest(&mut conn, &jot, "Claude", None, now()),
        Err(StoreError::AiOff)
    ));
    allow(&conn, AiAccess::Read);
    check_read(&conn).expect("can read");
    assert!(matches!(
        suggest(&mut conn, &jot, "Claude", None, now()),
        Err(StoreError::AiReadOnly)
    ));
}

#[test]
fn a_suggestion_waits_in_sort_saying_who_suggested_it_and_why() {
    let mut conn = open_in_memory().expect("open");
    allow(&conn, AiAccess::Suggest);
    let jot = Jot::plain("Book dentist");
    suggest(
        &mut conn,
        &jot,
        "Claude",
        Some(" You said your tooth hurts "),
        now(),
    )
    .expect("suggest");
    let queue = sort_queue(&conn).expect("queue");
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].suggested_by.as_deref(), Some("Claude"));
    assert_eq!(
        queue[0].reason.as_deref(),
        Some("You said your tooth hurts")
    );
    assert_eq!(queue[0].promise.state, bindle_core::State::Unsorted);
}

#[test]
fn typed_jots_are_not_marked_as_suggestions() {
    let mut conn = open_in_memory().expect("open");
    jot_down(&mut conn, &Jot::plain("Call mom"), now()).expect("jot");
    let queue = sort_queue(&conn).expect("queue");
    assert_eq!(
        (queue[0].suggested_by.as_deref(), queue[0].reason.as_deref()),
        (None, None)
    );
}

#[test]
fn suggestions_stop_when_sort_already_holds_ten() {
    let mut conn = open_in_memory().expect("open");
    allow(&conn, AiAccess::Suggest);
    for i in 0..MAX_WAITING {
        suggest(
            &mut conn,
            &Jot::plain(&format!("Idea {i}")),
            "Claude",
            None,
            now(),
        )
        .expect("suggest");
    }
    assert!(matches!(
        suggest(&mut conn, &Jot::plain("One more"), "Claude", None, now()),
        Err(StoreError::TooManySuggestions(10))
    ));
    jot_down(&mut conn, &Jot::plain("Typed still works"), now())
        .expect("the person is never limited");
}

#[test]
fn a_person_kept_from_ai_takes_their_promises_with_them() {
    let mut conn = open_in_memory().expect("open");
    let mut jot = Jot::plain("Gift for Ananya");
    jot.person = Some("Ananya".into());
    let gift = jot_down(&mut conn, &jot, now()).expect("jot");
    let other = jot_down(&mut conn, &Jot::plain("Call mom"), now()).expect("jot");
    let ananya = crate::people::find_or_add(&conn, "Ananya", "x").expect("find");
    set_person_hidden(&conn, &ananya, true).expect("hide");
    let hidden = hidden_ids(&conn).expect("hidden");
    let mut list = vec![gift, other];
    without_hidden(&mut list, &hidden);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].title, "Call mom");
}

#[test]
fn the_log_shows_newest_first_and_forgets_after_ninety_days() {
    let conn = open_in_memory().expect("open");
    let old = now() - chrono::Duration::days(100);
    log(&conn, "Claude", "home", "", old).expect("log");
    log(&conn, "Claude", "search", "dentist", now()).expect("log");
    log(
        &conn,
        "Claude",
        "home",
        "",
        now() + chrono::Duration::minutes(1),
    )
    .expect("log");
    let entries = recent(&conn, 10).expect("recent");
    let tools: Vec<&str> = entries.iter().map(|e| e.tool.as_str()).collect();
    assert_eq!(tools, ["home", "search"]);
}
