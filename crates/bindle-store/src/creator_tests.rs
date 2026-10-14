// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Creator checks: looks at YouTube, growth, top posts and comments.

use super::*;
use crate::open_in_memory;
use chrono::TimeZone;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

fn channel(followers: u64, views: u64) -> Channel {
    Channel {
        id: "UC123".into(),
        title: "Aspersh".into(),
        handle: Some("@aspersh".into()),
        description: "Long builds and walkthroughs.".into(),
        followers: Some(followers),
        views,
        posts: 40,
    }
}

fn post(id: &str, days_ago: i64, views: u64, short: bool) -> FoundPost {
    FoundPost {
        id: id.into(),
        title: format!("Video {id}"),
        url: format!("https://www.youtube.com/watch?v={id}"),
        short,
        duration_seconds: if short { 58 } else { 1084 },
        published_at: stamp(now() - Duration::days(days_ago)),
        views,
        likes: 10,
        comments: 2,
    }
}

fn comment(id: &str, hours_ago: i64) -> FoundComment {
    FoundComment {
        id: id.into(),
        post_id: "v1".into(),
        post_title: "I let an AI drive my Android phone".into(),
        author: "Kunal".into(),
        text: "Which model did you use for the screen reading?".into(),
        url: format!("https://www.youtube.com/watch?v=v1&lc={id}"),
        likes: 3,
        published_at: stamp(now() - Duration::hours(hours_ago)),
    }
}

#[test]
fn a_look_saves_numbers_posts_and_comments() {
    let mut conn = open_in_memory().unwrap();
    let posts = [post("v1", 3, 2100, false), post("v2", 10, 800, true)];
    sync(
        &mut conn,
        "youtube",
        &channel(642, 50_000),
        &posts,
        &[comment("c1", 5)],
        now(),
    )
    .unwrap();
    let view = creator_view(&conn, now()).unwrap();
    assert_eq!(view.top_posts.len(), 2);
    assert_eq!(view.top_posts[0].id, "v1");
    assert_eq!(view.views_last_30_days, 2900);
    assert_eq!((view.long_posts, view.short_posts), (1, 1));
    assert_eq!(view.comments.len(), 1);
    assert_eq!(comments_waiting(&conn).unwrap(), 1);
    // One look is not a week of history yet.
    assert_eq!(view.followers_this_week, None);
    assert_eq!(view.growth.len(), 1);
}

#[test]
fn growth_compares_with_a_week_ago_and_keeps_one_point_a_week() {
    let mut conn = open_in_memory().unwrap();
    for (days_ago, followers, views) in [
        (20, 590, 40_000),
        (8, 610, 45_000),
        (7, 614, 46_000),
        (0, 642, 50_000),
    ] {
        sync(
            &mut conn,
            "youtube",
            &channel(followers, views),
            &[],
            &[],
            now() - Duration::days(days_ago),
        )
        .unwrap();
    }
    let view = creator_view(&conn, now()).unwrap();
    assert_eq!(view.followers_this_week, Some(28));
    assert_eq!(view.views_this_week, Some(4_000));
    // 14 Oct 2026 is a Wednesday: the looks fall in three different weeks.
    let weeks: Vec<_> = view
        .growth
        .iter()
        .map(|p| (p.week.as_str(), p.followers))
        .collect();
    assert_eq!(
        weeks,
        [
            ("2026-09-21", 590),
            ("2026-10-05", 614),
            ("2026-10-12", 642)
        ]
    );
}

#[test]
fn a_hidden_subscriber_count_is_not_made_up() {
    let mut conn = open_in_memory().unwrap();
    let mut hidden = channel(0, 1000);
    hidden.followers = None;
    sync(
        &mut conn,
        "youtube",
        &hidden,
        &[],
        &[],
        now() - Duration::days(7),
    )
    .unwrap();
    sync(&mut conn, "youtube", &hidden, &[], &[], now()).unwrap();
    let view = creator_view(&conn, now()).unwrap();
    assert_eq!(view.followers_this_week, None);
    assert!(view.growth.is_empty());
}

#[test]
fn old_posts_show_when_nothing_is_recent() {
    let mut conn = open_in_memory().unwrap();
    let posts = [post("old1", 60, 100, false), post("old2", 45, 50, false)];
    sync(&mut conn, "youtube", &channel(10, 10), &posts, &[], now()).unwrap();
    let view = creator_view(&conn, now()).unwrap();
    assert_eq!(
        view.top_posts
            .iter()
            .map(|p| p.id.as_str())
            .collect::<Vec<_>>(),
        ["old2", "old1"]
    );
    assert_eq!(view.views_last_30_days, 0);
}

#[test]
fn a_comment_becomes_a_reply_promise_once() {
    let mut conn = open_in_memory().unwrap();
    sync(
        &mut conn,
        "youtube",
        &channel(10, 10),
        &[],
        &[comment("c1", 5)],
        now(),
    )
    .unwrap();
    let promise = make_promise(&mut conn, "c1", now()).unwrap();
    assert_eq!(
        promise.title,
        "Reply to Kunal on \"I let an AI drive my Android phone\""
    );
    assert_eq!(promise.person.as_deref(), Some("Kunal"));
    assert_eq!(promise.mode, Mode::Work);
    assert_eq!(promise.state, State::Unsorted);
    assert!(matches!(
        make_promise(&mut conn, "c1", now()),
        Err(StoreError::AlreadyPromised)
    ));
    assert_eq!(comments_waiting(&conn).unwrap(), 0);
    let view = creator_view(&conn, now()).unwrap();
    assert_eq!(
        view.comments[0].promise.as_ref().map(|p| p.state),
        Some(State::Unsorted)
    );
}

#[test]
fn answered_comments_go_and_hidden_ones_stay_hidden() {
    let mut conn = open_in_memory().unwrap();
    sync(
        &mut conn,
        "youtube",
        &channel(10, 10),
        &[],
        &[comment("c1", 5), comment("c2", 2)],
        now(),
    )
    .unwrap();
    hide(&conn, "c2", now()).unwrap();
    assert_eq!(comments_waiting(&conn).unwrap(), 1);
    sync(
        &mut conn,
        "youtube",
        &channel(10, 10),
        &[],
        &[comment("c2", 2)],
        now(),
    )
    .unwrap();
    let view = creator_view(&conn, now()).unwrap();
    assert!(view.comments.is_empty(), "c1 was answered, c2 is hidden");
    unhide(&conn, "c2").unwrap();
    assert_eq!(creator_view(&conn, now()).unwrap().comments.len(), 1);
    assert!(matches!(
        hide(&conn, "nope", now()),
        Err(StoreError::NotFound)
    ));
}

#[test]
fn the_account_keeps_no_key_and_disconnecting_forgets_the_rest() {
    let mut conn = open_in_memory().unwrap();
    assert_eq!(youtube_account(&conn).unwrap(), CreatorAccount::default());
    let account = CreatorAccount {
        channel: Some("@aspersh".into()),
        found: Some(channel(642, 50_000)),
        ..Default::default()
    };
    save_youtube_account(&conn, &account).unwrap();
    assert_eq!(youtube_account(&conn).unwrap(), account);
    sync(
        &mut conn,
        "youtube",
        &channel(10, 10),
        &[post("v1", 1, 5, false)],
        &[comment("c1", 1)],
        now(),
    )
    .unwrap();
    forget(&conn, "youtube").unwrap();
    let view = creator_view(&conn, now()).unwrap();
    assert!(view.top_posts.is_empty() && view.comments.is_empty() && view.growth.is_empty());
}
