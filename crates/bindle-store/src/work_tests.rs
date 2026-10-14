// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Work item checks: looks at GitHub, the Code screen, and Make it a promise.

use super::*;
use crate::lifecycle;
use crate::open_in_memory;
use chrono::TimeZone;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

fn item(id: &str, kind: WorkKind, created_at: &str) -> FoundItem {
    FoundItem {
        id: id.into(),
        kind,
        repo: "kunal-m/latch".into(),
        number: 42,
        title: "Permission prompts for the share sheet".into(),
        author: "kunal-m".into(),
        url: "https://github.com/kunal-m/latch/pull/42".into(),
        draft: false,
        labels: vec!["android".into()],
        created_at: created_at.into(),
        updated_at: created_at.into(),
    }
}

#[test]
fn a_look_adds_items_and_the_code_screen_sorts_them() {
    let mut conn = open_in_memory().unwrap();
    let found = [
        item("r1", WorkKind::Review, "2026-10-12T10:00:00Z"),
        item("r2", WorkKind::Review, "2026-10-14T06:00:00Z"),
        item("m1", WorkKind::Mine, "2026-10-13T09:00:00Z"),
        item("i1", WorkKind::Issue, "2026-10-10T09:00:00Z"),
        // The same item twice in one look counts once.
        item("i1", WorkKind::Issue, "2026-10-10T09:00:00Z"),
    ];
    let summary = sync(&mut conn, "github", &found, now()).unwrap();
    assert_eq!(summary, SyncSummary { new: 4, gone: 0 });

    let view = code_view(&conn, now()).unwrap();
    assert_eq!(
        view.reviews
            .iter()
            .map(|i| i.id.as_str())
            .collect::<Vec<_>>(),
        ["r1", "r2"]
    );
    assert_eq!(view.mine.len(), 1);
    assert_eq!(view.issues.len(), 1);
    assert_eq!(view.issues[0].labels, ["android"]);
    assert_eq!(view.reviews_older_than_a_day, 1);
    assert!(view.finished.is_empty());
    assert_eq!(reviews_owed(&conn).unwrap(), 2);
}

#[test]
fn an_item_github_stops_listing_is_dropped_unless_a_promise_is_open() {
    let mut conn = open_in_memory().unwrap();
    let r1 = item("r1", WorkKind::Review, "2026-10-12T10:00:00Z");
    let r2 = item("r2", WorkKind::Review, "2026-10-13T10:00:00Z");
    sync(&mut conn, "github", &[r1.clone(), r2], now()).unwrap();
    make_promise(&mut conn, "r1", now()).unwrap();

    // Both reviews are done on GitHub.
    let summary = sync(&mut conn, "github", &[], now()).unwrap();
    assert_eq!(summary, SyncSummary { new: 0, gone: 2 });
    let view = code_view(&conn, now()).unwrap();
    assert!(view.reviews.is_empty());
    assert_eq!(view.finished.len(), 1, "only the one with a promise stays");
    assert_eq!(view.finished[0].id, "r1");
    assert_eq!(reviews_owed(&conn).unwrap(), 0);

    // If it comes back (asked again), it is current again and keeps its promise.
    sync(&mut conn, "github", &[r1], now()).unwrap();
    let view = code_view(&conn, now()).unwrap();
    assert_eq!(view.reviews.len(), 1);
    assert!(view.reviews[0].promise.is_some());
    assert!(view.finished.is_empty());
}

#[test]
fn a_finished_item_goes_once_its_promise_is_closed() {
    let mut conn = open_in_memory().unwrap();
    sync(
        &mut conn,
        "github",
        &[item("r1", WorkKind::Review, "2026-10-12T10:00:00Z")],
        now(),
    )
    .unwrap();
    let promise = make_promise(&mut conn, "r1", now()).unwrap();
    sync(&mut conn, "github", &[], now()).unwrap();
    lifecycle::let_go(&mut conn, &promise.id, now()).unwrap();
    sync(&mut conn, "github", &[], now()).unwrap();
    assert!(code_view(&conn, now()).unwrap().finished.is_empty());
}

#[test]
fn make_it_a_promise_fills_in_what_and_for_whom() {
    let mut conn = open_in_memory().unwrap();
    sync(
        &mut conn,
        "github",
        &[
            item("r1", WorkKind::Review, "2026-10-12T10:00:00Z"),
            item("m1", WorkKind::Mine, "2026-10-12T10:00:00Z"),
        ],
        now(),
    )
    .unwrap();

    let review = make_promise(&mut conn, "r1", now()).unwrap();
    assert_eq!(
        review.title,
        "Review latch #42: Permission prompts for the share sheet"
    );
    assert_eq!(review.state, State::Unsorted);
    assert_eq!(review.mode, Mode::Work);
    assert_eq!(review.person.as_deref(), Some("kunal-m"));
    assert_eq!(review.direction, Some(Direction::Mine));

    let mine = make_promise(&mut conn, "m1", now()).unwrap();
    assert!(mine.title.starts_with("Get latch #42 merged"));
    assert_eq!(mine.person, None);
    assert_eq!(mine.direction, Some(Direction::SelfPromise));

    // The source link is kept, so the promise can open the pull request.
    let url: String = conn
        .query_row(
            "SELECT source.url FROM promise JOIN source ON source.id = promise.source_id WHERE promise.id = ?1",
            [&review.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(url, "https://github.com/kunal-m/latch/pull/42");

    let view = code_view(&conn, now()).unwrap();
    assert_eq!(
        view.reviews[0].promise,
        Some(Linked {
            id: review.id.clone(),
            state: State::Unsorted
        })
    );
}

#[test]
fn an_item_becomes_a_promise_only_once_at_a_time() {
    let mut conn = open_in_memory().unwrap();
    sync(
        &mut conn,
        "github",
        &[item("r1", WorkKind::Review, "2026-10-12T10:00:00Z")],
        now(),
    )
    .unwrap();
    let first = make_promise(&mut conn, "r1", now()).unwrap();
    assert!(matches!(
        make_promise(&mut conn, "r1", now()),
        Err(StoreError::AlreadyPromised)
    ));

    // Once that promise is let go, it can be made again.
    lifecycle::let_go(&mut conn, &first.id, now()).unwrap();
    let second = make_promise(&mut conn, "r1", now()).unwrap();
    assert_ne!(first.id, second.id);
    assert!(matches!(
        make_promise(&mut conn, "nope", now()),
        Err(StoreError::NotFound)
    ));
}

#[test]
fn disconnecting_forgets_items_but_keeps_promises() {
    let mut conn = open_in_memory().unwrap();
    sync(
        &mut conn,
        "github",
        &[item("r1", WorkKind::Review, "2026-10-12T10:00:00Z")],
        now(),
    )
    .unwrap();
    let promise = make_promise(&mut conn, "r1", now()).unwrap();
    forget(&conn, "github").unwrap();
    assert!(code_view(&conn, now()).unwrap().reviews.is_empty());
    assert_eq!(
        promises::get(&conn, &promise.id).unwrap().title,
        promise.title
    );
}

#[test]
fn the_github_account_is_saved_without_a_token() {
    let conn = open_in_memory().unwrap();
    assert_eq!(github_account(&conn).unwrap(), GithubAccount::default());
    let account = GithubAccount {
        login: Some("aspershupadhyay".into()),
        synced_at: Some("2026-10-14T12:00:00.000Z".into()),
        error: None,
    };
    save_github_account(&conn, &account).unwrap();
    assert_eq!(github_account(&conn).unwrap(), account);
    let raw: String = conn
        .query_row("SELECT value FROM setting WHERE key = 'github'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(!raw.contains("token"));
}

#[test]
fn jot_down_in_work_saves_a_work_promise() {
    let mut conn = open_in_memory().unwrap();
    let promise = promises::jot_down_in(
        &mut conn,
        &Jot::plain("Write the release notes"),
        Mode::Work,
        now(),
    )
    .unwrap();
    assert_eq!(promise.mode, Mode::Work);
    let life = promises::jot_down(&mut conn, &Jot::plain("Call Mum"), now()).unwrap();
    assert_eq!(life.mode, Mode::Life);
}
