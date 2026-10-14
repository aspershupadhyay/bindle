// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The device side against a real relay on this machine: batches go up
//! sealed and come back in order, only key holders can read or write a
//! space, invites work once, and a recovery kit brings spaces back.

use bindle_cloud::codes::{Backup, BackupSpace, Invite};
use bindle_cloud::{CloudError, Key, Relay};

async fn relay() -> (Relay, rusqlite::Connection) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let path = std::env::temp_dir().join(format!("bindle-relay-test-{}.db", Key::random().id()));
    let conn = rusqlite::Connection::open(&path).unwrap();
    tokio::spawn(bindle_relay::serve(listener, conn));
    (
        Relay::new(&base).unwrap(),
        rusqlite::Connection::open(&path).unwrap(),
    )
}

#[tokio::test]
async fn batches_go_up_sealed_and_come_back_in_order() {
    let (relay, peek) = relay().await;
    let space = Key::random();
    assert_eq!(
        relay
            .push(&space, b"first: Pay electricity bill")
            .await
            .unwrap(),
        1
    );
    assert_eq!(relay.push(&space, b"second").await.unwrap(), 2);

    let all = relay.pull(&space, 0).await.unwrap();
    assert_eq!(
        all,
        vec![
            (1, b"first: Pay electricity bill".to_vec()),
            (2, b"second".to_vec())
        ]
    );
    assert_eq!(relay.pull(&space, 1).await.unwrap().len(), 1);

    // What the relay stores says nothing about what is inside.
    let stored: Vec<String> = peek
        .prepare("SELECT sealed FROM batch")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(stored.len(), 2);
    assert!(stored.iter().all(|s| !s.contains("electricity")));
    let token_hashes: String = peek
        .query_row("SELECT token_hash FROM space", [], |r| r.get(0))
        .unwrap();
    assert!(
        !token_hashes.contains(&space.write_token()),
        "only a hash of the token is kept"
    );
}

#[tokio::test]
async fn only_a_key_holder_can_write_to_a_space() {
    let (relay, _) = relay().await;
    let space = Key::random();
    relay.push(&space, b"ours").await.unwrap();
    // Someone who knows only the id cannot use another token.
    let impostor = reqwest::Client::new()
        .post(format!("{}/v1/spaces/{}/batches", relay.base(), space.id()))
        .bearer_auth(Key::random().write_token())
        .json(&space.seal(b"not ours"))
        .send()
        .await
        .unwrap();
    assert_eq!(impostor.status().as_u16(), 403);
    assert_eq!(relay.pull(&space, 0).await.unwrap().len(), 1);
}

#[tokio::test]
async fn an_invite_works_once() {
    let (relay, _) = relay().await;
    let secret = Key::random();
    let invite = Invite {
        space_key: Key::random().to_text(),
        space_name: "Upadhyay Home".into(),
        from: "Sparsh".into(),
        relay: relay.base().into(),
    };
    relay.put_invite(&secret, &invite).await.unwrap();
    assert_eq!(relay.take_invite(&secret).await.unwrap(), invite);
    assert_eq!(
        relay.take_invite(&secret).await,
        Err(CloudError::InviteGone)
    );
    assert_eq!(
        relay.take_invite(&Key::random()).await,
        Err(CloudError::InviteGone)
    );
}

#[tokio::test]
async fn a_recovery_kit_brings_spaces_back() {
    let (relay, _) = relay().await;
    let kit = Key::random();
    let backup = Backup {
        name: "Sparsh".into(),
        spaces: vec![BackupSpace {
            key: Key::random().to_text(),
            name: "Upadhyay Home".into(),
            relay: relay.base().into(),
        }],
    };
    assert_eq!(relay.get_backup(&kit).await, Err(CloudError::NotFound));
    relay.put_backup(&kit, &backup).await.unwrap();
    assert_eq!(relay.get_backup(&kit).await.unwrap(), backup);
    let updated = Backup {
        name: "Sparsh U".into(),
        ..backup
    };
    relay.put_backup(&kit, &updated).await.unwrap();
    assert_eq!(relay.get_backup(&kit).await.unwrap(), updated);
}

#[tokio::test]
async fn no_relay_means_offline() {
    let relay = Relay::new("http://127.0.0.1:9").unwrap();
    assert_eq!(
        relay.push(&Key::random(), b"x").await,
        Err(CloudError::Offline)
    );
}
