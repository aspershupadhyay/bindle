// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Work items: what a source such as GitHub says needs you (handbook P14).
//!
//! Each look at the source replaces what Bindle knows: new items are added,
//! changed ones updated, and ones the source no longer lists are marked gone.
//! A gone item is kept only while it is linked to a promise that is still
//! open, so the Code screen can ask whether that promise is done too.
//! Nothing here changes a promise by itself (handbook principle 4).

use crate::promises::{self, Jot, Origin};
use crate::StoreError;
use bindle_core::{Direction, Mode, Promise, State, Strength};
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// What kind of thing needs you.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkKind {
    /// Someone asked you to review their pull request.
    Review,
    /// Your own pull request, waiting on others.
    Mine,
    /// An issue assigned to you.
    Issue,
}

impl WorkKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Mine => "mine",
            Self::Issue => "issue",
        }
    }

    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "review" => Some(Self::Review),
            "mine" => Some(Self::Mine),
            "issue" => Some(Self::Issue),
            _ => None,
        }
    }
}

/// One item as the source lists it now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundItem {
    /// The source's own id for it, stable across looks.
    pub id: String,
    pub kind: WorkKind,
    /// "owner/name".
    pub repo: String,
    pub number: u32,
    pub title: String,
    /// Who opened it: the person waiting on a review, or who reported an issue.
    pub author: String,
    pub url: String,
    pub draft: bool,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A promise made from a work item, and where it stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Linked {
    pub id: String,
    pub state: State,
}

/// A work item, as the Code screen shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkItem {
    pub id: String,
    pub kind: WorkKind,
    pub repo: String,
    pub number: u32,
    pub title: String,
    pub author: String,
    pub url: String,
    pub draft: bool,
    pub labels: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    /// The source no longer lists it: reviewed, merged, closed or reassigned.
    pub gone: bool,
    pub promise: Option<Linked>,
}

/// The GitHub connection, as Settings and the Code screen show it. The
/// token itself is never stored here: it lives in the system keychain.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GithubAccount {
    /// The GitHub user name, once connected.
    pub login: Option<String>,
    /// When Bindle last looked, successfully.
    pub synced_at: Option<String>,
    /// Why the last look failed, in plain words, if it did.
    pub error: Option<String>,
}

const GITHUB_KEY: &str = "github";

/// Reads the GitHub connection, or an empty one.
pub fn github_account(conn: &Connection) -> Result<GithubAccount, StoreError> {
    let saved: Option<String> = conn
        .query_row(
            "SELECT value FROM setting WHERE key = ?1",
            [GITHUB_KEY],
            |r| r.get(0),
        )
        .optional()?;
    Ok(saved
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default())
}

/// Saves the GitHub connection.
pub fn save_github_account(conn: &Connection, account: &GithubAccount) -> Result<(), StoreError> {
    let json = serde_json::to_string(account).expect("account serialises");
    conn.execute(
        "INSERT INTO setting (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [GITHUB_KEY, json.as_str()],
    )?;
    Ok(())
}

/// What one look at a source changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct SyncSummary {
    /// Items Bindle had not seen before.
    pub new: u32,
    /// Items the source stopped listing.
    pub gone: u32,
}

/// Replaces what Bindle knows from `source` with `items`, all in one
/// transaction. Items stay linked to their promises.
pub fn sync(
    conn: &mut Connection,
    source: &str,
    items: &[FoundItem],
    now: DateTime<Utc>,
) -> Result<SyncSummary, StoreError> {
    let at = stamp(now);
    let tx = conn.transaction()?;
    let known: HashSet<String> = {
        let mut stmt = tx.prepare("SELECT id FROM work_item WHERE source = ?1")?;
        let ids = stmt.query_map([source], |r| r.get(0))?;
        ids.collect::<Result<_, _>>()?
    };
    let mut summary = SyncSummary::default();
    let mut listed = HashSet::new();
    for item in items {
        if !listed.insert(item.id.as_str()) {
            continue;
        }
        if !known.contains(&item.id) {
            summary.new += 1;
        }
        tx.execute(
            "INSERT INTO work_item (id, source, kind, repo, number, title, author, url, draft,
                                    labels, created_at, updated_at, seen_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
               kind = excluded.kind, repo = excluded.repo, number = excluded.number,
               title = excluded.title, author = excluded.author, url = excluded.url,
               draft = excluded.draft, labels = excluded.labels,
               updated_at = excluded.updated_at, seen_at = excluded.seen_at, gone_at = NULL",
            params![
                item.id,
                source,
                item.kind.as_str(),
                item.repo,
                item.number,
                item.title,
                item.author,
                item.url,
                item.draft,
                serde_json::to_string(&item.labels).expect("labels serialise"),
                item.created_at,
                item.updated_at,
                at,
            ],
        )?;
    }
    for id in known.iter().filter(|id| !listed.contains(id.as_str())) {
        summary.gone += tx.execute(
            "UPDATE work_item SET gone_at = ?2 WHERE id = ?1 AND gone_at IS NULL",
            params![id, at],
        )? as u32;
    }
    // A gone item is worth keeping only while its promise is still open.
    tx.execute(
        "DELETE FROM work_item WHERE source = ?1 AND gone_at IS NOT NULL
           AND (promise_id IS NULL
                OR promise_id IN (SELECT id FROM promise WHERE state IN ('done','let_go')))",
        [source],
    )?;
    tx.commit()?;
    Ok(summary)
}

/// Forgets everything from `source`, as when it is disconnected. Promises
/// made from its items stay as they are.
pub fn forget(conn: &Connection, source: &str) -> Result<(), StoreError> {
    conn.execute("DELETE FROM work_item WHERE source = ?1", [source])?;
    Ok(())
}

/// Everything the Code screen shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodeView {
    pub account: GithubAccount,
    /// Pull requests waiting on your review, oldest first.
    pub reviews: Vec<WorkItem>,
    /// Your pull requests waiting on others, oldest first.
    pub mine: Vec<WorkItem>,
    /// Issues assigned to you, most recently active first.
    pub issues: Vec<WorkItem>,
    /// Gone from GitHub while their promise is still open: maybe done.
    pub finished: Vec<WorkItem>,
    /// Reviews asked for more than a day ago.
    pub reviews_older_than_a_day: u32,
}

/// Reads the Code screen. `now` decides what is "older than a day".
pub fn code_view(conn: &Connection, now: DateTime<Utc>) -> Result<CodeView, StoreError> {
    let all = list(conn, "github")?;
    let day_ago = now - chrono::Duration::days(1);
    let (finished, current): (Vec<_>, Vec<_>) = all.into_iter().partition(|i| i.gone);
    let of = |kind: WorkKind| -> Vec<WorkItem> {
        current.iter().filter(|i| i.kind == kind).cloned().collect()
    };
    let reviews = of(WorkKind::Review);
    let mut issues = of(WorkKind::Issue);
    issues.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(CodeView {
        account: github_account(conn)?,
        reviews_older_than_a_day: reviews
            .iter()
            .filter(|i| DateTime::parse_from_rfc3339(&i.created_at).is_ok_and(|at| at < day_ago))
            .count() as u32,
        reviews,
        mine: of(WorkKind::Mine),
        issues,
        finished,
    })
}

/// How many reviews you owe right now, for the sidebar.
pub fn reviews_owed(conn: &Connection) -> Result<u32, StoreError> {
    Ok(conn.query_row(
        "SELECT count(*) FROM work_item WHERE kind = 'review' AND gone_at IS NULL",
        [],
        |r| r.get(0),
    )?)
}

/// "Make it a promise": a new work promise from the item, waiting in Sort
/// with who it is for already filled in. Refused while the item already
/// has a promise that is not done or let go.
pub fn make_promise(
    conn: &mut Connection,
    item_id: &str,
    now: DateTime<Utc>,
) -> Result<Promise, StoreError> {
    let item = get(conn, item_id)?;
    if item
        .promise
        .as_ref()
        .is_some_and(|p| !matches!(p.state, State::Done | State::LetGo))
    {
        return Err(StoreError::AlreadyPromised);
    }
    let jot = promise_words(&item);
    let label = format!("GitHub {}#{}", item.repo, item.number);
    let promise = promises::insert(
        conn,
        &jot,
        Origin::Link {
            label: &label,
            url: &item.url,
        },
        Mode::Work,
        now,
    )?;
    conn.execute(
        "UPDATE work_item SET promise_id = ?2 WHERE id = ?1",
        params![item.id, promise.id],
    )?;
    Ok(promise)
}

/// The words of the promise made from an item, as Jot down would have read
/// them: what to do, and for whom.
pub fn promise_words(item: &WorkItem) -> Jot {
    let short = format!("{} #{}", repo_name(&item.repo), item.number);
    let (title, person, direction) = match item.kind {
        WorkKind::Review => (
            format!("Review {short}: {}", item.title),
            Some(item.author.clone()),
            Some(Direction::Mine),
        ),
        WorkKind::Issue => (
            format!("Fix {short}: {}", item.title),
            Some(item.author.clone()),
            Some(Direction::Mine),
        ),
        WorkKind::Mine => (
            format!("Get {short} merged: {}", item.title),
            None,
            Some(Direction::SelfPromise),
        ),
    };
    Jot {
        text: format!("{} ({})", item.title, item.url),
        title,
        when: None,
        is_deadline: false,
        amount: None,
        person,
        direction,
        strength: Strength::Firm,
        tz: None,
        unsure_words: None,
    }
}

/// "latch" from "kunal/latch".
fn repo_name(repo: &str) -> &str {
    repo.rsplit('/').next().unwrap_or(repo)
}

const ITEM_COLUMNS: &str = "w.id, w.kind, w.repo, w.number, w.title, w.author, w.url, w.draft, \
                            w.labels, w.created_at, w.updated_at, w.gone_at, p.id, p.state";

fn get(conn: &Connection, id: &str) -> Result<WorkItem, StoreError> {
    let sql = format!(
        "SELECT {ITEM_COLUMNS} FROM work_item w LEFT JOIN promise p ON p.id = w.promise_id
         WHERE w.id = ?1"
    );
    conn.query_row(&sql, [id], read_item)
        .optional()?
        .ok_or(StoreError::NotFound)
}

fn list(conn: &Connection, source: &str) -> Result<Vec<WorkItem>, StoreError> {
    let sql = format!(
        "SELECT {ITEM_COLUMNS} FROM work_item w LEFT JOIN promise p ON p.id = w.promise_id
         WHERE w.source = ?1 ORDER BY w.created_at, w.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([source], read_item)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn read_item(row: &Row) -> rusqlite::Result<WorkItem> {
    let kind: String = row.get(1)?;
    let labels: String = row.get(8)?;
    let promise_id: Option<String> = row.get(12)?;
    let state: Option<String> = row.get(13)?;
    Ok(WorkItem {
        id: row.get(0)?,
        kind: WorkKind::from_word(&kind).unwrap_or(WorkKind::Issue),
        repo: row.get(2)?,
        number: row.get(3)?,
        title: row.get(4)?,
        author: row.get(5)?,
        url: row.get(6)?,
        draft: row.get(7)?,
        labels: serde_json::from_str(&labels).unwrap_or_default(),
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        gone: row.get::<_, Option<String>>(11)?.is_some(),
        promise: promise_id
            .zip(state.and_then(|s| State::from_word(&s)))
            .map(|(id, state)| Linked { id, state }),
    })
}

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
#[path = "work_tests.rs"]
mod tests;
