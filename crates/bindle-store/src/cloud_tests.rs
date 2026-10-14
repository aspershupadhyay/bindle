// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Ours checks: two devices, each with its own database, swapping batches
//! the way the relay would. Whatever order changes arrive in, both end up
//! with the same promises.

use super::*;
use crate::lifecycle::{self, Sorting};
use crate::open_in_memory;
use crate::promises::{self, jot_down, Jot};
use bindle_core::{Direction, State, Strength, When};
use chrono::{Duration, NaiveDate, TimeZone};

const SPACE: &str = "0123456789abcdef0123456789abcdef";

fn t(minutes: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap() + Duration::minutes(minutes)
}

struct Device {
    conn: Connection,
    device: &'static str,
    account: &'static str,
    name: &'static str,
}

impl Device {
    fn new(device: &'static str, account: &'static str, name: &'static str) -> Self {
        let conn = open_in_memory().unwrap();
        add_space(&conn, SPACE, "Upadhyay Home", "https://cloud.example", t(0)).unwrap();
        Self {
            conn,
            device,
            account,
            name,
        }
    }

    /// Collects changes and hands back the batches that would go up.
    fn send(&mut self, at: DateTime<Utc>) -> Vec<Vec<FieldOp>> {
        collect_changes(
            &mut self.conn,
            SPACE,
            self.device,
            self.account,
            self.name,
            at,
        )
        .unwrap();
        let batches = outbox(&self.conn, SPACE).unwrap();
        for (id, _) in &batches {
            sent(&self.conn, *id).unwrap();
        }
        batches
            .into_iter()
            .map(|(_, json)| serde_json::from_str(&json).unwrap())
            .collect()
    }

    fn receive(&mut self, batches: &[Vec<FieldOp>], at: DateTime<Utc>) -> usize {
        batches
            .iter()
            .map(|ops| apply_changes(&mut self.conn, SPACE, ops, at).unwrap())
            .sum()
    }

    fn promise(&self, id: &str) -> bindle_core::Promise {
        promises::get(&self.conn, id).unwrap()
    }
}

fn sparsh() -> Device {
    Device::new("laptop", "acct-sparsh", "Sparsh")
}

fn wifey() -> Device {
    Device::new("phone", "acct-wifey", "Wifey")
}

/// A kept promise in Ours: "Pay electricity bill", Friday, with Rohan.
fn shared_bill(device: &mut Device) -> String {
    let promise = jot_down(&mut device.conn, &Jot::plain("Pay electricity bill"), t(0)).unwrap();
    lifecycle::sort(
        &mut device.conn,
        &promise.id,
        &Sorting {
            title: "Pay electricity bill".into(),
            direction: Some(Direction::Mine),
            strength: Strength::Firm,
            person: Some("Rohan".into()),
            first_step: Some("Find the bill".into()),
            when: Some(When {
                day: NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
                time: None,
            }),
            is_deadline: false,
        },
        State::Open,
        t(0),
    )
    .unwrap();
    share(&device.conn, &promise.id, Some(SPACE), device.account).unwrap();
    promise.id
}

#[test]
fn a_promise_shared_on_one_device_appears_on_the_other() {
    let (mut a, mut b) = (sparsh(), wifey());
    let id = shared_bill(&mut a);
    let batches = a.send(t(1));
    assert_eq!(batches.len(), 1);
    assert!(b.receive(&batches, t(2)) > 0);

    let got = b.promise(&id);
    assert_eq!(got.title, "Pay electricity bill");
    assert_eq!(got.state, State::Open);
    assert_eq!(got.person.as_deref(), Some("Rohan"));
    assert_eq!(got.first_step.as_deref(), Some("Find the bill"));
    assert_eq!(
        got.do_on.map(|w| w.day),
        NaiveDate::from_ymd_opt(2026, 10, 16)
    );
    assert_eq!(got.direction, Some(Direction::Mine));
    let added_by: String = b
        .conn
        .query_row("SELECT created_by FROM promise WHERE id = ?1", [&id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(added_by, "acct-sparsh");
    // Who carries Ours: Sparsh announced himself.
    assert_eq!(members(&b.conn, SPACE).unwrap()[0].name, "Sparsh");
    // The other device has history of it arriving.
    assert!(lifecycle::history(&b.conn, &id)
        .unwrap()
        .iter()
        .any(|h| h.kind == "received"));
}

#[test]
fn nothing_changed_means_nothing_to_send_and_echoes_change_nothing() {
    let (mut a, mut b) = (sparsh(), wifey());
    shared_bill(&mut a);
    let first = a.send(t(1));
    assert!(a.send(t(2)).is_empty(), "the same state is not sent twice");
    b.receive(&first, t(2));
    // B has nothing new of its own except its member record.
    let echo = b.send(t(3));
    let fields: Vec<_> = echo.concat().iter().map(|op| op.record.clone()).collect();
    assert!(
        fields.iter().all(|r| r.starts_with("member:")),
        "{fields:?}"
    );
    // Its own batch coming back changes nothing.
    assert_eq!(a.receive(&first, t(4)), 0);
}

#[test]
fn edits_to_different_fields_both_survive() {
    let (mut a, mut b) = (sparsh(), wifey());
    let id = shared_bill(&mut a);
    b.receive(&a.send(t(1)), t(1));

    // Offline at the same time: Sparsh renames it, Wifey marks it done.
    a.conn
        .execute(
            "UPDATE promise SET title = 'Pay the electricity bill' WHERE id = ?1",
            [&id],
        )
        .unwrap();
    lifecycle::mark_done(&mut b.conn, &id, t(5)).unwrap();
    let from_a = a.send(t(6));
    let from_b = b.send(t(7));
    a.receive(&from_b, t(8));
    b.receive(&from_a, t(8));

    for device in [&a, &b] {
        let p = device.promise(&id);
        assert_eq!(p.title, "Pay the electricity bill");
        assert_eq!(p.state, State::Done);
        let done_by: Option<String> = device
            .conn
            .query_row("SELECT done_by FROM promise WHERE id = ?1", [&id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            done_by.as_deref(),
            Some("acct-wifey"),
            "kept by Wifey, on both devices"
        );
    }
}

#[test]
fn the_later_edit_to_the_same_field_wins_in_any_order() {
    let (mut a, mut b) = (sparsh(), wifey());
    let id = shared_bill(&mut a);
    let base = a.send(t(1));
    b.receive(&base, t(1));

    a.conn
        .execute(
            "UPDATE promise SET first_step = 'Open the app' WHERE id = ?1",
            [&id],
        )
        .unwrap();
    b.conn
        .execute(
            "UPDATE promise SET first_step = 'Ask Papa' WHERE id = ?1",
            [&id],
        )
        .unwrap();
    let early = a.send(t(10));
    let late = b.send(t(20));

    // A third device that hears them in the wrong order agrees with both.
    let mut c = Device::new("tablet", "acct-wifey", "Wifey");
    c.receive(&base, t(21));
    c.receive(&late, t(21));
    c.receive(&early, t(22));
    a.receive(&late, t(23));
    b.receive(&early, t(23));
    for device in [&a, &b, &c] {
        assert_eq!(device.promise(&id).first_step.as_deref(), Some("Ask Papa"));
    }
}

#[test]
fn a_change_that_breaks_the_rules_is_skipped_not_fatal() {
    let (mut a, mut b) = (sparsh(), wifey());
    let id = shared_bill(&mut a);
    let mut batches = a.send(t(1));
    // A future version sends a state this one does not know.
    let bad = FieldOp {
        record: format!("promise:{id}"),
        field: "state".into(),
        value: Some("\"archived\"".into()),
        stamp: Stamp {
            millis: u64::MAX / 2,
            counter: 0,
            device: "future".into(),
        },
    };
    batches.push(vec![bad]);
    b.receive(&batches, t(2));
    assert_eq!(b.promise(&id).state, State::Open);
}

#[test]
fn leaving_keeps_the_promises_in_mine() {
    let (mut a, mut b) = (sparsh(), wifey());
    let id = shared_bill(&mut a);
    b.receive(&a.send(t(1)), t(1));
    leave(&b.conn, SPACE).unwrap();
    assert!(spaces(&b.conn).unwrap().is_empty());
    let space: Option<String> = b
        .conn
        .query_row("SELECT space_id FROM promise WHERE id = ?1", [&id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(space, None);
    assert_eq!(b.promise(&id).title, "Pay electricity bill");
    assert!(matches!(
        share(&b.conn, &id, Some(SPACE), "x"),
        Err(StoreError::NoSuchSpace)
    ));
}

#[test]
fn spaces_list_their_promises_and_waiting_changes() {
    let mut a = sparsh();
    shared_bill(&mut a);
    collect_changes(&mut a.conn, SPACE, a.device, a.account, a.name, t(1)).unwrap();
    let list = spaces(&a.conn).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Upadhyay Home");
    assert_eq!(list[0].promises, 1);
    assert_eq!(list[0].waiting, 1);
    set_pulled(&a.conn, SPACE, 7, t(2)).unwrap();
    set_pulled(&a.conn, SPACE, 3, t(3)).unwrap();
    assert_eq!(
        pulled(&a.conn, SPACE).unwrap(),
        7,
        "the cursor never goes back"
    );
}

#[test]
fn the_cloud_account_is_saved_without_secrets() {
    let conn = open_in_memory().unwrap();
    assert_eq!(cloud_account(&conn).unwrap(), CloudAccount::default());
    let account = CloudAccount {
        relay: Some("https://cloud.example".into()),
        name: Some("Sparsh".into()),
        account: Some("acct".into()),
        device: Some("laptop".into()),
        kit_saved_at: None,
    };
    save_cloud_account(&conn, &account).unwrap();
    assert_eq!(cloud_account(&conn).unwrap(), account);
}
