// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Ours from end to end: two people, each with their own database, share a
//! space through a real relay on this machine. One invites, the other
//! joins with the link, and promises flow both ways, sealed on the way.

use bindle_cloud::client::Batch;
use bindle_cloud::codes::{self, Invite};
use bindle_cloud::{Key, Relay};
use bindle_core::State;
use bindle_store::promises::{jot_down, Jot};
use bindle_store::{cloud, lifecycle, open_in_memory};
use chrono::Utc;
use rusqlite::Connection;

/// One sync round for a device: send what changed, take what is new.
async fn sync(
    conn: &mut Connection,
    relay: &Relay,
    key: &Key,
    device: &str,
    account: &str,
    name: &str,
) {
    let space = key.id();
    cloud::collect_changes(conn, &space, device, account, name, Utc::now()).unwrap();
    for (id, json) in cloud::outbox(conn, &space).unwrap() {
        let batch = Batch {
            ops: serde_json::from_str(&json).unwrap(),
        };
        relay
            .push(key, &serde_json::to_vec(&batch).unwrap())
            .await
            .unwrap();
        cloud::sent(conn, id).unwrap();
    }
    let after = cloud::pulled(conn, &space).unwrap();
    for (seq, bytes) in relay.pull(key, after).await.unwrap() {
        let batch: Batch = serde_json::from_slice(&bytes).unwrap();
        cloud::apply_changes(conn, &space, &batch.ops, Utc::now()).unwrap();
        cloud::set_pulled(conn, &space, seq, Utc::now()).unwrap();
    }
}

#[tokio::test]
async fn two_people_share_ours_through_the_relay() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(bindle_relay::serve(
        listener,
        Connection::open_in_memory().unwrap(),
    ));
    let relay = Relay::new(&base).unwrap();

    // Sparsh makes Ours and invites Wifey.
    let mut sparsh = open_in_memory().unwrap();
    let key = Key::random();
    cloud::add_space(&sparsh, &key.id(), "Upadhyay Home", &base, Utc::now()).unwrap();
    let secret = Key::random();
    relay
        .put_invite(
            &secret,
            &Invite {
                space_key: key.to_text(),
                space_name: "Upadhyay Home".into(),
                from: "Sparsh".into(),
                relay: base.clone(),
            },
        )
        .await
        .unwrap();
    let link = codes::invite_link(&secret, &base);

    // He shares a promise.
    let bill = jot_down(&mut sparsh, &Jot::plain("Pay electricity bill"), Utc::now()).unwrap();
    cloud::share(&sparsh, &bill.id, Some(&key.id()), "acct-sparsh").unwrap();
    sync(&mut sparsh, &relay, &key, "laptop", "acct-sparsh", "Sparsh").await;

    // Wifey joins with nothing but the link.
    let (got_secret, got_relay) = codes::read_invite_link(&link).unwrap();
    let joined = Relay::new(&got_relay.unwrap()).unwrap();
    let invite = joined.take_invite(&got_secret).await.unwrap();
    assert_eq!(invite.from, "Sparsh");
    let her_key = Key::from_text(&invite.space_key).unwrap();
    let mut wifey = open_in_memory().unwrap();
    cloud::add_space(
        &wifey,
        &her_key.id(),
        &invite.space_name,
        &invite.relay,
        Utc::now(),
    )
    .unwrap();
    sync(
        &mut wifey,
        &joined,
        &her_key,
        "phone",
        "acct-wifey",
        "Wifey",
    )
    .await;

    let seen = bindle_store::promises::get(&wifey, &bill.id).unwrap();
    assert_eq!(seen.title, "Pay electricity bill");

    // She adds one and keeps his; he sees both.
    let gas = jot_down(&mut wifey, &Jot::plain("Refill gas cylinder"), Utc::now()).unwrap();
    cloud::share(&wifey, &gas.id, Some(&her_key.id()), "acct-wifey").unwrap();
    lifecycle::let_go(&mut wifey, &bill.id, Utc::now()).unwrap();
    sync(
        &mut wifey,
        &joined,
        &her_key,
        "phone",
        "acct-wifey",
        "Wifey",
    )
    .await;
    sync(&mut sparsh, &relay, &key, "laptop", "acct-sparsh", "Sparsh").await;

    assert_eq!(
        bindle_store::promises::get(&sparsh, &gas.id).unwrap().title,
        "Refill gas cylinder"
    );
    assert_eq!(
        bindle_store::promises::get(&sparsh, &bill.id)
            .unwrap()
            .state,
        State::LetGo
    );
    let names: Vec<String> = cloud::members(&sparsh, &key.id())
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();
    assert!(names.contains(&"Wifey".to_string()), "{names:?}");

    // The invite cannot be used a second time.
    assert!(joined.take_invite(&got_secret).await.is_err());
}
