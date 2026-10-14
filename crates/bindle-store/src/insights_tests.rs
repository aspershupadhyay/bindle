// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Desk checks, for Wednesday 14 October 2026: what needs you now, how long
//! people wait, what shipped, and the week's work.

use super::*;
use crate::lifecycle::{self, Sorting};
use crate::mail::{FoundMail, MailKind};
use crate::open_in_memory;
use crate::promises::{jot_down_in, Jot};
use crate::work::FoundItem;
use bindle_core::{Mode, Strength, When};
use chrono::TimeZone;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

fn monday() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 12, 0, 0, 0).unwrap()
}

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 14).unwrap()
}

fn sample(id: &str, kind: WaitKind, answered: DateTime<Utc>, hours: i64) -> WaitSample {
    WaitSample {
        id: id.into(),
        kind,
        person: "Priya".into(),
        asked_at: answered - Duration::hours(hours),
        answered_at: answered,
    }
}

#[test]
fn medians_are_medians() {
    assert_eq!(median(&[]), None);
    assert_eq!(median(&[3.0]), Some(3.0));
    assert_eq!(median(&[1.0, 2.0, 10.0]), Some(2.0));
    assert_eq!(median(&[1.0, 3.0]), Some(2.0));
}

#[test]
fn a_wait_is_counted_once_and_backwards_clocks_are_skipped() {
    let mut conn = open_in_memory().unwrap();
    let s = sample("e1", WaitKind::EmailReply, now(), 3);
    assert_eq!(record_waits(&mut conn, &[s.clone(), s.clone()]).unwrap(), 1);
    assert_eq!(record_waits(&mut conn, &[s]).unwrap(), 0);
    let backwards = WaitSample {
        asked_at: now(),
        answered_at: now() - Duration::hours(1),
        ..sample("e2", WaitKind::EmailReply, now(), 0)
    };
    assert_eq!(record_waits(&mut conn, &[backwards]).unwrap(), 0);
    assert_eq!(known_waits(&conn, WaitKind::EmailReply).unwrap(), ["e1"]);
    assert!(known_waits(&conn, WaitKind::Review).unwrap().is_empty());
}

#[test]
fn recent_waits_are_compared_with_the_four_weeks_before() {
    let mut conn = open_in_memory().unwrap();
    record_waits(
        &mut conn,
        &[
            // This week: 2 h and 4 h.
            sample("a", WaitKind::EmailReply, now() - Duration::days(1), 2),
            sample("b", WaitKind::EmailReply, now() - Duration::days(2), 4),
            // The four weeks before: 5 h, 6 h, 7 h.
            sample("c", WaitKind::EmailReply, now() - Duration::days(10), 5),
            sample("d", WaitKind::EmailReply, now() - Duration::days(17), 6),
            sample("e", WaitKind::EmailReply, now() - Duration::days(30), 7),
            // Too old to compare.
            sample("f", WaitKind::EmailReply, now() - Duration::days(60), 100),
            // Reviews are their own story.
            sample("r", WaitKind::Review, now() - Duration::days(1), 18),
        ],
    )
    .unwrap();
    let desk = desk_view(&conn, now(), today(), monday()).unwrap();
    assert_eq!(desk.email_reply.recent, Some(3.0));
    assert_eq!(desk.email_reply.before, Some(6.0));
    assert_eq!(desk.email_reply.weekly.len(), 8);
    assert_eq!(desk.email_reply.weekly[7], Some(3.0), "this week");
    assert_eq!(
        desk.email_reply.weekly[6], None,
        "nothing answered last week"
    );
    assert_eq!(
        desk.email_reply.weekly[5],
        Some(5.0),
        "Sunday 4 October, two weeks back"
    );
    assert_eq!(desk.email_reply.weekly[0], None);
    assert_eq!(desk.review_wait.recent, Some(18.0));
    assert_eq!(desk.review_wait.before, None);
}

fn kept_work(conn: &mut Connection, title: &str, day: u32) -> String {
    let promise = jot_down_in(conn, &Jot::plain(title), Mode::Work, now()).unwrap();
    let sorting = Sorting {
        title: title.into(),
        direction: Some(bindle_core::Direction::Mine),
        strength: Strength::Firm,
        person: None,
        first_step: None,
        when: Some(When {
            day: NaiveDate::from_ymd_opt(2026, 10, day).unwrap(),
            time: None,
        }),
        is_deadline: false,
    };
    lifecycle::sort(conn, &promise.id, &sorting, State::Open, now()).unwrap();
    promise.id
}

#[test]
fn shipped_counts_work_promises_by_week_and_source() {
    let mut conn = open_in_memory().unwrap();
    let a = kept_work(&mut conn, "Ship the release", 13);
    let b = kept_work(&mut conn, "Write the notes", 14);
    let old = kept_work(&mut conn, "Old thing", 14);
    lifecycle::mark_done(&mut conn, &a, now()).unwrap();
    lifecycle::mark_done(&mut conn, &b, now()).unwrap();
    lifecycle::mark_done(&mut conn, &old, monday() - Duration::days(3)).unwrap();
    // A Life promise kept this week does not count at work.
    let life = jot_down_in(&mut conn, &Jot::plain("Call Mum"), Mode::Life, now()).unwrap();
    lifecycle::let_go(&mut conn, &life.id, now()).unwrap();

    let desk = desk_view(&conn, now(), today(), monday()).unwrap();
    assert_eq!(desk.shipped.this_week, 2);
    assert_eq!(desk.shipped.weekly[7], 2);
    assert_eq!(desk.shipped.weekly[6], 1);
    assert_eq!(desk.shipped.other, 2);
    assert_eq!(desk.shipped.from_code, 0);
}

#[test]
fn the_week_shows_open_and_done_work_per_day() {
    let mut conn = open_in_memory().unwrap();
    kept_work(&mut conn, "Tuesday thing", 13);
    let done = kept_work(&mut conn, "Wednesday thing", 14);
    kept_work(&mut conn, "Another Wednesday thing", 14);
    lifecycle::mark_done(&mut conn, &done, now()).unwrap();
    let desk = desk_view(&conn, now(), today(), monday()).unwrap();
    assert_eq!(desk.week.len(), 7);
    assert_eq!(
        desk.week[0].day,
        NaiveDate::from_ymd_opt(2026, 10, 12).unwrap()
    );
    assert_eq!((desk.week[1].open, desk.week[1].done), (1, 0));
    assert_eq!((desk.week[2].open, desk.week[2].done), (1, 1));
}

#[test]
fn needs_you_merges_github_and_email_oldest_first() {
    let mut conn = open_in_memory().unwrap();
    work::sync(
        &mut conn,
        "github",
        &[
            FoundItem {
                id: "r1".into(),
                kind: WorkKind::Review,
                repo: "kunal-m/latch".into(),
                number: 42,
                title: "Permission prompts".into(),
                author: "kunal-m".into(),
                url: "https://github.com/kunal-m/latch/pull/42".into(),
                draft: false,
                labels: vec![],
                created_at: "2026-10-12T10:00:00Z".into(),
                updated_at: "2026-10-12T10:00:00Z".into(),
            },
            FoundItem {
                id: "m1".into(),
                kind: WorkKind::Mine,
                repo: "me/bindle".into(),
                number: 3,
                title: "Mine waits on others".into(),
                author: "me".into(),
                url: "https://github.com/me/bindle/pull/3".into(),
                draft: false,
                labels: vec![],
                created_at: "2026-10-11T10:00:00Z".into(),
                updated_at: "2026-10-11T10:00:00Z".into(),
            },
        ],
        now(),
    )
    .unwrap();
    mail::sync(
        &mut conn,
        "gmail",
        &[FoundMail {
            id: "reply:t1".into(),
            kind: MailKind::Reply,
            thread: "t1".into(),
            subject: "Sponsor brief".into(),
            person: "Priya Sharma".into(),
            address: "priya@brand.example".into(),
            // The same instant style Gmail looks give: with an offset.
            at: "2026-10-13T09:00:00+00:00".into(),
            snippet: String::new(),
            url: "https://mail.google.com/mail/u/0/#search/x".into(),
            promise: None,
        }],
        now(),
    )
    .unwrap();
    let desk = desk_view(&conn, now(), today(), monday()).unwrap();
    let order: Vec<_> = desk
        .needs_you
        .iter()
        .map(|i| (i.source.as_str(), i.kind.as_str()))
        .collect();
    assert_eq!(
        order,
        [("github", "review"), ("gmail", "reply")],
        "your own pull requests are not on the Desk"
    );
    assert_eq!(desk.needs_you[0].reference, "latch #42");
    assert_eq!(desk.replies_owed, 1);
    assert_eq!(desk.reviews_older_than_a_day, 1);
}
