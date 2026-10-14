// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Creator (handbook P14): what a creator source such as YouTube shows.
//!
//! Each look saves the channel's numbers for the day (so growth builds up
//! by itself), replaces the recent posts, and updates the comments waiting
//! on a reply. A comment becomes a promise only when the person says so,
//! and nothing is ever posted back (handbook principle 6). No scores: the
//! numbers are the channel's own, compared only with its own past.

use crate::promises::{self, Jot, Origin};
use crate::work::Linked;
use crate::StoreError;
use bindle_core::{Direction, Mode, Promise, State, Strength};
use chrono::{DateTime, Datelike, Duration, NaiveDate, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A channel's public numbers at one look.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub title: String,
    /// "@aspersh", when the channel has one.
    pub handle: Option<String>,
    pub description: String,
    /// `None` when the channel hides its subscriber count.
    pub followers: Option<u64>,
    pub views: u64,
    pub posts: u64,
}

/// A recent post, as the source lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FoundPost {
    pub id: String,
    pub title: String,
    pub url: String,
    pub short: bool,
    pub duration_seconds: u32,
    pub published_at: String,
    pub views: u64,
    pub likes: u64,
    pub comments: u64,
}

/// A comment waiting on the creator's reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FoundComment {
    pub id: String,
    pub post_id: String,
    pub post_title: String,
    pub author: String,
    pub text: String,
    pub url: String,
    pub likes: u64,
    pub published_at: String,
}

/// A comment as the Creator screen shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Comment {
    pub id: String,
    pub post_title: String,
    pub author: String,
    pub text: String,
    pub url: String,
    pub likes: u64,
    pub published_at: String,
    pub promise: Option<Linked>,
}

/// The YouTube connection. The API key lives in the system keychain.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatorAccount {
    /// What the person typed: "@aspersh" or a channel id.
    pub channel: Option<String>,
    /// The channel as of the last good look.
    pub found: Option<Channel>,
    pub synced_at: Option<String>,
    pub error: Option<String>,
}

const YOUTUBE_KEY: &str = "youtube";

/// Reads the YouTube connection, or an empty one.
pub fn youtube_account(conn: &Connection) -> Result<CreatorAccount, StoreError> {
    let saved: Option<String> = conn
        .query_row(
            "SELECT value FROM setting WHERE key = ?1",
            [YOUTUBE_KEY],
            |r| r.get(0),
        )
        .optional()?;
    Ok(saved
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default())
}

/// Saves the YouTube connection (never the key).
pub fn save_youtube_account(conn: &Connection, account: &CreatorAccount) -> Result<(), StoreError> {
    let json = serde_json::to_string(account).expect("account serialises");
    conn.execute(
        "INSERT INTO setting (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [YOUTUBE_KEY, json.as_str()],
    )?;
    Ok(())
}

/// Saves one look at `source`: today's numbers, the recent posts, and the
/// comments waiting on a reply, in one transaction. Comments no longer
/// waiting are marked gone; they are kept only while their promise is open.
pub fn sync(
    conn: &mut Connection,
    source: &str,
    channel: &Channel,
    posts: &[FoundPost],
    comments: &[FoundComment],
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    let at = stamp(now);
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO creator_snapshot (source, day, followers, views, posts)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(source, day) DO UPDATE SET
           followers = excluded.followers, views = excluded.views, posts = excluded.posts",
        params![
            source,
            now.date_naive().to_string(),
            channel.followers.map(|f| f as i64),
            channel.views as i64,
            channel.posts as i64,
        ],
    )?;
    tx.execute("DELETE FROM creator_post WHERE source = ?1", [source])?;
    for post in posts {
        tx.execute(
            "INSERT OR REPLACE INTO creator_post (id, source, title, url, short, duration_seconds,
                                                  published_at, views, likes, comments, seen_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                post.id,
                source,
                post.title,
                post.url,
                post.short,
                post.duration_seconds,
                post.published_at,
                post.views as i64,
                post.likes as i64,
                post.comments as i64,
                at,
            ],
        )?;
    }
    let mut listed = HashSet::new();
    for comment in comments {
        if !listed.insert(comment.id.as_str()) {
            continue;
        }
        tx.execute(
            "INSERT INTO creator_comment (id, source, post_id, post_title, author, text, url,
                                          likes, published_at, seen_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
               post_title = excluded.post_title, text = excluded.text, likes = excluded.likes,
               seen_at = excluded.seen_at, gone_at = NULL",
            params![
                comment.id,
                source,
                comment.post_id,
                comment.post_title,
                comment.author,
                comment.text,
                comment.url,
                comment.likes as i64,
                comment.published_at,
                at,
            ],
        )?;
    }
    let known: Vec<String> = {
        let mut stmt =
            tx.prepare("SELECT id FROM creator_comment WHERE source = ?1 AND gone_at IS NULL")?;
        let ids = stmt.query_map([source], |r| r.get(0))?;
        ids.collect::<Result<_, _>>()?
    };
    for id in known.iter().filter(|id| !listed.contains(id.as_str())) {
        tx.execute(
            "UPDATE creator_comment SET gone_at = ?2 WHERE id = ?1",
            params![id, at],
        )?;
    }
    tx.execute(
        "DELETE FROM creator_comment WHERE source = ?1 AND gone_at IS NOT NULL
           AND (promise_id IS NULL
                OR promise_id IN (SELECT id FROM promise WHERE state IN ('done','let_go')))",
        [source],
    )?;
    tx.commit()?;
    Ok(())
}

/// Forgets everything from `source`, as when it is disconnected. Promises
/// made from its comments stay as they are.
pub fn forget(conn: &Connection, source: &str) -> Result<(), StoreError> {
    for table in ["creator_snapshot", "creator_post", "creator_comment"] {
        conn.execute(&format!("DELETE FROM {table} WHERE source = ?1"), [source])?;
    }
    Ok(())
}

/// A post as the Creator screen shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub url: String,
    pub short: bool,
    pub duration_seconds: u32,
    pub published_at: String,
    pub views: u64,
    pub likes: u64,
    pub comments: u64,
}

/// Followers at the end of one week, for the growth chart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WeekPoint {
    /// The Monday the week starts on.
    pub week: String,
    pub followers: u64,
}

/// Everything the Creator screen shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatorView {
    pub account: CreatorAccount,
    /// Followers gained or lost since a week ago, when Bindle has a week of history.
    pub followers_this_week: Option<i64>,
    /// Views gained since a week ago, the same way.
    pub views_this_week: Option<i64>,
    /// Followers at the end of each of the last 12 weeks that have a look.
    pub growth: Vec<WeekPoint>,
    /// The most watched posts of the last 30 days (the newest, when none are that recent).
    pub top_posts: Vec<Post>,
    /// Views on posts from the last 30 days.
    pub views_last_30_days: u64,
    /// Of posts from the last 90 days, how many were long and how many shorts.
    pub long_posts: u32,
    pub short_posts: u32,
    /// Comments waiting on your reply, newest first.
    pub comments: Vec<Comment>,
}

/// Reads the Creator screen.
pub fn creator_view(conn: &Connection, now: DateTime<Utc>) -> Result<CreatorView, StoreError> {
    let source = "youtube";
    let snapshots = snapshots(conn, source)?;
    let today = snapshots.last();
    let week_ago = (now - Duration::days(7)).date_naive();
    let before = snapshots.iter().rev().find(|s| s.0 <= week_ago);
    let (followers_this_week, views_this_week) = match (today, before) {
        (Some(t), Some(b)) => (
            t.1.zip(b.1).map(|(t, b)| t as i64 - b as i64),
            Some(t.2 as i64 - b.2 as i64),
        ),
        _ => (None, None),
    };
    let posts = posts(conn, source)?;
    let since_30 = stamp(now - Duration::days(30));
    let since_90 = stamp(now - Duration::days(90));
    let recent: Vec<&Post> = posts
        .iter()
        .filter(|p| p.published_at >= since_30)
        .collect();
    let mut top_posts: Vec<Post> = if recent.is_empty() {
        posts.iter().take(5).cloned().collect()
    } else {
        let mut by_views: Vec<Post> = recent.iter().map(|p| (*p).clone()).collect();
        by_views.sort_by_key(|p| std::cmp::Reverse(p.views));
        by_views
    };
    top_posts.truncate(5);
    let quarter: Vec<&Post> = posts
        .iter()
        .filter(|p| p.published_at >= since_90)
        .collect();
    Ok(CreatorView {
        account: youtube_account(conn)?,
        followers_this_week,
        views_this_week,
        growth: growth(&snapshots, now),
        views_last_30_days: recent.iter().map(|p| p.views).sum(),
        long_posts: quarter.iter().filter(|p| !p.short).count() as u32,
        short_posts: quarter.iter().filter(|p| p.short).count() as u32,
        top_posts,
        comments: comments(conn, source)?,
    })
}

/// How many comments wait on your reply, for the sidebar.
pub fn comments_waiting(conn: &Connection) -> Result<u32, StoreError> {
    Ok(conn.query_row(
        "SELECT count(*) FROM creator_comment c LEFT JOIN promise p ON p.id = c.promise_id
         WHERE c.gone_at IS NULL AND c.hidden_at IS NULL
           AND (c.promise_id IS NULL OR p.state IN ('done','let_go'))",
        [],
        |r| r.get(0),
    )?)
}

/// "Make it a promise": a work promise to reply, waiting in Sort with the
/// commenter as the person. Refused while one is still open.
pub fn make_promise(
    conn: &mut Connection,
    comment_id: &str,
    now: DateTime<Utc>,
) -> Result<Promise, StoreError> {
    let comment = get(conn, comment_id)?;
    if comment
        .promise
        .as_ref()
        .is_some_and(|p| !matches!(p.state, State::Done | State::LetGo))
    {
        return Err(StoreError::AlreadyPromised);
    }
    let jot = promise_words(&comment);
    let promise = promises::insert(
        conn,
        &jot,
        Origin::Link {
            label: "YouTube comment",
            url: &comment.url,
        },
        Mode::Work,
        now,
    )?;
    conn.execute(
        "UPDATE creator_comment SET promise_id = ?2 WHERE id = ?1",
        params![comment.id, promise.id],
    )?;
    Ok(promise)
}

/// The promise made from a comment: reply to its author.
pub fn promise_words(comment: &Comment) -> Jot {
    Jot {
        text: format!("{}: {} ({})", comment.author, comment.text, comment.url),
        title: format!(
            "Reply to {} on \"{}\"",
            comment.author,
            short(&comment.post_title, 60)
        ),
        when: None,
        is_deadline: false,
        amount: None,
        person: Some(comment.author.clone()),
        direction: Some(Direction::Mine),
        strength: Strength::Soft,
        tz: None,
        unsure_words: None,
    }
}

/// "Not needed": hides a comment in Bindle. YouTube is not touched.
pub fn hide(conn: &Connection, comment_id: &str, now: DateTime<Utc>) -> Result<(), StoreError> {
    let changed = conn.execute(
        "UPDATE creator_comment SET hidden_at = ?2 WHERE id = ?1",
        params![comment_id, stamp(now)],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

/// Shows a hidden comment again.
pub fn unhide(conn: &Connection, comment_id: &str) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE creator_comment SET hidden_at = NULL WHERE id = ?1",
        [comment_id],
    )?;
    Ok(())
}

fn short(text: &str, most: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= most {
        return text.to_string();
    }
    let cut: String = text.chars().take(most - 1).collect();
    format!("{}…", cut.trim_end())
}

type Snapshot = (NaiveDate, Option<u64>, u64);

fn snapshots(conn: &Connection, source: &str) -> Result<Vec<Snapshot>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT day, followers, views FROM creator_snapshot WHERE source = ?1 ORDER BY day",
    )?;
    let rows = stmt.query_map([source], |r| {
        let day: String = r.get(0)?;
        Ok((
            day,
            r.get::<_, Option<i64>>(1)?.map(|f| f as u64),
            r.get::<_, i64>(2)? as u64,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (day, followers, views) = row?;
        if let Ok(day) = NaiveDate::parse_from_str(&day, "%Y-%m-%d") {
            out.push((day, followers, views));
        }
    }
    Ok(out)
}

/// The last look of each of the 12 weeks up to `now` that has followers.
fn growth(snapshots: &[Snapshot], now: DateTime<Utc>) -> Vec<WeekPoint> {
    let monday = |day: NaiveDate| day - Duration::days(day.weekday().num_days_from_monday() as i64);
    let first = monday(now.date_naive()) - Duration::weeks(11);
    let mut points: Vec<WeekPoint> = Vec::new();
    for (day, followers, _) in snapshots.iter().filter(|s| s.0 >= first) {
        let Some(followers) = followers else { continue };
        let week = monday(*day).to_string();
        match points.last_mut() {
            Some(last) if last.week == week => last.followers = *followers,
            _ => points.push(WeekPoint {
                week,
                followers: *followers,
            }),
        }
    }
    points
}

fn posts(conn: &Connection, source: &str) -> Result<Vec<Post>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, url, short, duration_seconds, published_at, views, likes, comments
         FROM creator_post WHERE source = ?1 ORDER BY published_at DESC",
    )?;
    let rows = stmt.query_map([source], |r| {
        Ok(Post {
            id: r.get(0)?,
            title: r.get(1)?,
            url: r.get(2)?,
            short: r.get(3)?,
            duration_seconds: r.get(4)?,
            published_at: r.get(5)?,
            views: r.get::<_, i64>(6)? as u64,
            likes: r.get::<_, i64>(7)? as u64,
            comments: r.get::<_, i64>(8)? as u64,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

const COMMENT_COLUMNS: &str = "c.id, c.post_title, c.author, c.text, c.url, c.likes, \
                               c.published_at, p.id, p.state";

/// Comments still waiting, plus gone ones whose promise is still open.
fn comments(conn: &Connection, source: &str) -> Result<Vec<Comment>, StoreError> {
    let sql = format!(
        "SELECT {COMMENT_COLUMNS} FROM creator_comment c LEFT JOIN promise p ON p.id = c.promise_id
         WHERE c.source = ?1 AND c.hidden_at IS NULL AND c.gone_at IS NULL
         ORDER BY c.published_at DESC, c.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([source], read_comment)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn get(conn: &Connection, id: &str) -> Result<Comment, StoreError> {
    let sql = format!(
        "SELECT {COMMENT_COLUMNS} FROM creator_comment c LEFT JOIN promise p ON p.id = c.promise_id
         WHERE c.id = ?1"
    );
    conn.query_row(&sql, [id], read_comment)
        .optional()?
        .ok_or(StoreError::NotFound)
}

fn read_comment(row: &Row) -> rusqlite::Result<Comment> {
    let promise_id: Option<String> = row.get(7)?;
    let state: Option<String> = row.get(8)?;
    Ok(Comment {
        id: row.get(0)?,
        post_title: row.get(1)?,
        author: row.get(2)?,
        text: row.get(3)?,
        url: row.get(4)?,
        likes: row.get::<_, i64>(5)? as u64,
        published_at: row.get(6)?,
        promise: promise_id
            .zip(state.and_then(|s| State::from_word(&s)))
            .map(|(id, state)| Linked { id, state }),
    })
}

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
#[path = "creator_tests.rs"]
mod tests;
