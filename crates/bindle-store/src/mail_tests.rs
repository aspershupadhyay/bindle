// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Email checks: looks at Gmail, the Inbox screen, Not needed and promises.

use super::*;
use crate::lifecycle;
use crate::open_in_memory;
use chrono::{NaiveDate, TimeZone};

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

fn found(id: &str, kind: MailKind, at: &str) -> FoundMail {
    FoundMail {
        id: id.into(),
        kind,
        thread: format!("t-{id}"),
        subject: "Sponsor brief: Q4 carousel series".into(),
        person: "Priya Sharma".into(),
        address: "priya@example.com".into(),
        at: at.into(),
        snippet: "We would love 4 carousels between 20 Oct and 10 Nov".into(),
        url: format!("https://mail.google.com/mail/u/0/#search/rfc822msgid%3A{id}"),
        promise: None,
    }
}

fn noticed(id: &str) -> FoundMail {
    FoundMail {
        promise: Some(NoticedPromise {
            title: "Send Priya the first concept".into(),
            when: Some(When {
                day: NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
                time: None,
            }),
            is_deadline: true,
        }),
        ..found(id, MailKind::Promise, "2026-10-12T10:12:00Z")
    }
}

#[test]
fn a_look_fills_the_inbox_in_order() {
    let mut conn = open_in_memory().unwrap();
    let items = [
        found("r2", MailKind::Reply, "2026-10-13T09:00:00Z"),
        found("r1", MailKind::Reply, "2026-10-12T09:00:00Z"),
        found("w1", MailKind::Waiting, "2026-10-12T09:00:00Z"),
        noticed("p1"),
        found("r1", MailKind::Reply, "2026-10-12T09:00:00Z"),
    ];
    let summary = sync(&mut conn, "gmail", &items, now()).unwrap();
    assert_eq!(summary.new, 4);
    let view = inbox_view(&conn).unwrap();
    assert_eq!(
        view.reply.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        ["r1", "r2"]
    );
    assert_eq!(view.waiting.len(), 1);
    assert_eq!(view.noticed.len(), 1);
    let promise = view.noticed[0].noticed.as_ref().unwrap();
    assert_eq!(
        promise.when.unwrap().day,
        NaiveDate::from_ymd_opt(2026, 10, 16).unwrap()
    );
    assert!(promise.is_deadline);
    assert_eq!(replies_owed(&conn).unwrap(), 2);
}

#[test]
fn not_needed_hides_an_item_until_undone() {
    let mut conn = open_in_memory().unwrap();
    let item = found("r1", MailKind::Reply, "2026-10-12T09:00:00Z");
    sync(&mut conn, "gmail", std::slice::from_ref(&item), now()).unwrap();
    dismiss(&conn, "r1", now()).unwrap();
    assert!(inbox_view(&conn).unwrap().reply.is_empty());
    assert_eq!(inbox_view(&conn).unwrap().hidden, 1);
    assert_eq!(replies_owed(&conn).unwrap(), 0);

    // A later look keeps it hidden.
    sync(&mut conn, "gmail", &[item], now()).unwrap();
    assert!(inbox_view(&conn).unwrap().reply.is_empty());

    restore(&conn, "r1").unwrap();
    assert_eq!(inbox_view(&conn).unwrap().reply.len(), 1);
    assert!(matches!(
        dismiss(&conn, "nope", now()),
        Err(StoreError::NotFound)
    ));
}

#[test]
fn a_reply_becomes_a_promise_for_that_person() {
    let mut conn = open_in_memory().unwrap();
    sync(
        &mut conn,
        "gmail",
        &[
            found("r1", MailKind::Reply, "2026-10-12T09:00:00Z"),
            found("w1", MailKind::Waiting, "2026-10-12T09:00:00Z"),
        ],
        now(),
    )
    .unwrap();
    let reply = make_promise(&mut conn, "r1", now()).unwrap();
    assert_eq!(
        reply.title,
        "Reply to Priya: Sponsor brief: Q4 carousel series"
    );
    assert_eq!(reply.person.as_deref(), Some("Priya Sharma"));
    assert_eq!(reply.direction, Some(Direction::Mine));
    assert_eq!(reply.mode, Mode::Work);
    assert_eq!(reply.state, State::Unsorted);

    let waiting = make_promise(&mut conn, "w1", now()).unwrap();
    assert_eq!(
        waiting.title,
        "Priya to reply: Sponsor brief: Q4 carousel series"
    );
    assert_eq!(waiting.direction, Some(Direction::Theirs));

    assert!(matches!(
        make_promise(&mut conn, "r1", now()),
        Err(StoreError::AlreadyPromised)
    ));
    let view = inbox_view(&conn).unwrap();
    assert_eq!(
        view.reply[0].promise.as_ref().map(|p| p.state),
        Some(State::Unsorted)
    );
}

#[test]
fn a_noticed_promise_keeps_its_day_and_leaves_the_list_once_added() {
    let mut conn = open_in_memory().unwrap();
    sync(&mut conn, "gmail", &[noticed("p1")], now()).unwrap();
    let promise = make_promise(&mut conn, "p1", now()).unwrap();
    assert_eq!(promise.title, "Send Priya the first concept");
    assert_eq!(
        promise.due.map(|w| w.day),
        NaiveDate::from_ymd_opt(2026, 10, 16)
    );
    assert!(inbox_view(&conn).unwrap().noticed.is_empty());
}

#[test]
fn a_reply_sent_elsewhere_moves_the_item_to_finished() {
    let mut conn = open_in_memory().unwrap();
    sync(
        &mut conn,
        "gmail",
        &[
            found("r1", MailKind::Reply, "2026-10-12T09:00:00Z"),
            found("r2", MailKind::Reply, "2026-10-12T09:00:00Z"),
        ],
        now(),
    )
    .unwrap();
    let promise = make_promise(&mut conn, "r1", now()).unwrap();
    // Both were answered from the phone.
    sync(&mut conn, "gmail", &[], now()).unwrap();
    let view = inbox_view(&conn).unwrap();
    assert!(view.reply.is_empty());
    assert_eq!(
        view.finished
            .iter()
            .map(|i| i.id.as_str())
            .collect::<Vec<_>>(),
        ["r1"]
    );

    lifecycle::let_go(&mut conn, &promise.id, now()).unwrap();
    sync(&mut conn, "gmail", &[], now()).unwrap();
    assert!(inbox_view(&conn).unwrap().finished.is_empty());
}

#[test]
fn the_account_is_saved_without_a_password() {
    let conn = open_in_memory().unwrap();
    assert_eq!(gmail_account(&conn).unwrap(), MailAccount::default());
    let account = MailAccount {
        address: Some("me@gmail.com".into()),
        synced_at: None,
        error: None,
        ..Default::default()
    };
    save_gmail_account(&conn, &account).unwrap();
    assert_eq!(gmail_account(&conn).unwrap(), account);
    forget(&conn, "gmail").unwrap();
}

#[test]
fn accounts_saved_before_other_mailboxes_read_as_gmail() {
    let conn = open_in_memory().unwrap();
    conn.execute(
        "INSERT INTO setting (key, value) VALUES ('gmail', '{\"address\":\"me@gmail.com\",\"synced_at\":null,\"error\":null}')",
        [],
    )
    .unwrap();
    let account = gmail_account(&conn).unwrap();
    assert_eq!(account.address.as_deref(), Some("me@gmail.com"));
    assert_eq!(account.provider, None);
    let icloud = MailAccount {
        address: Some("me@icloud.com".into()),
        provider: Some("icloud".into()),
        ..Default::default()
    };
    save_gmail_account(&conn, &icloud).unwrap();
    assert_eq!(gmail_account(&conn).unwrap(), icloud);
}
