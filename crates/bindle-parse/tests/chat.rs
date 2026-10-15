// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Catching promises in pasted chats. "Now" is Wednesday 14 October 2026.

use bindle_core::Direction;
use bindle_parse::chat::{catch, messages};
use chrono::{NaiveDate, NaiveDateTime};

fn now() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 10, 14)
        .unwrap()
        .and_hms_opt(18, 30, 0)
        .unwrap()
}

fn day(m: u32, d: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(2026, m, d)
}

const IPHONE: &str = "\
[12/10/26, 9:14:03 PM] Rohan Mehta: bro dinner was fun
[12/10/26, 9:15:10 PM] Rohan Mehta: I'll pay you back the 500 kal
[12/10/26, 9:16:00 PM] Sparsh: haha sure
[12/10/26, 9:20:41 PM] Sparsh: Also I'll send you the trip photos by Friday
[13/10/26, 11:02:00 AM] Rohan Mehta: Can you share the Goa hotel link?
[13/10/26, 11:05:00 AM] Sparsh: ok will do
[13/10/26, 11:06:00 AM] Rohan Mehta: \u{200e}image omitted";

#[test]
fn an_iphone_export_finds_both_sides_and_the_quiet_yes() {
    let found = catch(IPHONE, Some("Sparsh"), now(), &[]);
    assert_eq!(found.speakers, ["Rohan Mehta", "Sparsh"]);
    let rows: Vec<_> = found
        .found
        .iter()
        .map(|c| {
            (
                c.reading.title.as_str(),
                c.reading.direction,
                c.reading.person.as_deref(),
                c.reading.when.map(|w| w.day),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            // Said on Monday 12th: "kal" is the 13th, not tomorrow.
            (
                "Rohan Mehta will pay me back",
                Some(Direction::Theirs),
                Some("Rohan Mehta"),
                day(10, 13)
            ),
            (
                "Send Rohan Mehta the trip photos",
                Some(Direction::Mine),
                Some("Rohan Mehta"),
                day(10, 16)
            ),
            (
                "Share the Goa hotel link",
                Some(Direction::Mine),
                Some("Rohan Mehta"),
                None
            ),
        ]
    );
    assert!(found.found[0].reading.amount.is_some(), "the 500 is money");
    assert!(
        found.found[1].reading.is_deadline,
        "by Friday is a deadline"
    );
}

#[test]
fn an_android_export_and_hinglish() {
    let chat = "\
14/10/2026, 10:02 am - Meera: kal tak bill bhar dena please
14/10/2026, 10:05 am - Aarav: haan pakka
14/10/2026, 10:06 am - Meera: main sabzi le aaungi shaam ko
14/10/2026, 10:07 am - Aarav: Messages and calls are end-to-end encrypted";
    let found = catch(chat, Some("Aarav"), now(), &[]);
    assert_eq!(found.found.len(), 2);
    let bill = &found.found[0];
    assert_eq!(bill.reading.direction, Some(Direction::Mine));
    assert_eq!(bill.reading.person.as_deref(), Some("Meera"));
    assert_eq!(bill.reading.when.map(|w| w.day), day(10, 15));
    let veg = &found.found[1];
    assert_eq!(veg.reading.direction, Some(Direction::Theirs));
    assert_eq!(veg.reading.person.as_deref(), Some("Meera"));
    assert_eq!(veg.reading.title, "Meera: sabzi le aaungi");
}

#[test]
fn a_pasted_email_with_no_names_is_your_own_words() {
    let email = "Hi Priya,\nThanks for the brief. I'll send the first concept by Friday, and I can share the rate card tomorrow.\nBest, Sparsh";
    let found = catch(email, None, now(), &["Priya".to_string()]);
    let titles: Vec<_> = found
        .found
        .iter()
        .map(|c| c.reading.title.as_str())
        .collect();
    assert_eq!(titles, ["Send the first concept", "Share the rate card"]);
    assert!(found
        .found
        .iter()
        .all(|c| c.reading.direction == Some(Direction::Mine)));
}

#[test]
fn not_every_i_will_is_a_promise() {
    let chat = "\
Rohan: I'll be late, traffic
Rohan: I can't make it tomorrow
Sparsh: I won't forget lol
Sparsh: ok
Rohan: see you";
    assert!(catch(chat, Some("Sparsh"), now(), &[]).found.is_empty());
}

#[test]
fn a_promise_said_twice_is_kept_once() {
    let chat = "You: I'll call the landlord\nYou: I'll call the landlord";
    let found = catch(chat, None, now(), &[]);
    assert_eq!(found.me.as_deref(), Some("You"));
    assert_eq!(found.found.len(), 1);
}

#[test]
fn lines_that_start_no_message_continue_the_last() {
    let chat = "[12/10/26, 9:14 PM] Neha: hey\nI'll send the curl timings tomorrow\n[12/10/26, 9:15 PM] Sparsh: thanks";
    let all = messages(chat);
    assert_eq!(all.len(), 2);
    let found = catch(chat, Some("Sparsh"), now(), &[]);
    assert_eq!(found.found[0].reading.person.as_deref(), Some("Neha"));
    assert_eq!(found.found[0].reading.when.map(|w| w.day), day(10, 13));
}
