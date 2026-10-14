// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Email that needs you (handbook P14): threads waiting on your reply,
//! your questions waiting on someone else, and promises you wrote in an
//! email. A source such as Gmail finds them; this module keeps them.
//!
//! Like work items, each look replaces what Bindle knows. "Not needed"
//! hides an item in Bindle only; nothing is ever changed in the mailbox, and
//! Bindle never sends email (handbook principle 6).

use crate::promises::{self, Jot, Origin};
use crate::work::Linked;
use crate::StoreError;
use bindle_core::{Direction, Mode, Promise, State, Strength, When};
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Why an email needs you.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailKind {
    /// Someone wrote to you and is waiting on your reply.
    Reply,
    /// You asked someone something and they have not answered yet.
    Waiting,
    /// You wrote a promise in an email: "I'll send it by Friday."
    Promise,
}

impl MailKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reply => "reply",
            Self::Waiting => "waiting",
            Self::Promise => "promise",
        }
    }

    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "reply" => Some(Self::Reply),
            "waiting" => Some(Self::Waiting),
            "promise" => Some(Self::Promise),
            _ => None,
        }
    }
}

/// A promise noticed in an email you wrote, as Jot down would read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NoticedPromise {
    pub title: String,
    pub when: Option<When>,
    pub is_deadline: bool,
}

/// One item as the source finds it now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundMail {
    /// Stable across looks: the same thread or message gives the same id.
    pub id: String,
    pub kind: MailKind,
    pub thread: String,
    pub subject: String,
    /// The other person: who wrote to you, or whom you wrote to.
    pub person: String,
    pub address: String,
    /// When the latest email in it was sent.
    pub at: String,
    /// A few plain lines from it, without quoted replies.
    pub snippet: String,
    /// Opens the email in the mailbox's own web page.
    pub url: String,
    pub promise: Option<NoticedPromise>,
}

/// An item, as the Inbox screen shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MailItem {
    pub id: String,
    pub kind: MailKind,
    pub subject: String,
    pub person: String,
    pub address: String,
    pub at: String,
    pub snippet: String,
    pub url: String,
    pub noticed: Option<NoticedPromise>,
    /// The mailbox no longer shows it as needing you.
    pub gone: bool,
    pub promise: Option<Linked>,
}

/// The Gmail connection. The app password lives in the system keychain,
/// never here.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MailAccount {
    pub address: Option<String>,
    pub synced_at: Option<String>,
    pub error: Option<String>,
}

const GMAIL_KEY: &str = "gmail";

/// Reads the Gmail connection, or an empty one.
pub fn gmail_account(conn: &Connection) -> Result<MailAccount, StoreError> {
    let saved: Option<String> = conn
        .query_row(
            "SELECT value FROM setting WHERE key = ?1",
            [GMAIL_KEY],
            |r| r.get(0),
        )
        .optional()?;
    Ok(saved
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default())
}

/// Saves the Gmail connection.
pub fn save_gmail_account(conn: &Connection, account: &MailAccount) -> Result<(), StoreError> {
    let json = serde_json::to_string(account).expect("account serialises");
    conn.execute(
        "INSERT INTO setting (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [GMAIL_KEY, json.as_str()],
    )?;
    Ok(())
}

/// Replaces what Bindle knows from `source` with `items`. Items the person
/// marked "Not needed" stay hidden; items that are gone are dropped unless
/// their promise is still open.
pub fn sync(
    conn: &mut Connection,
    source: &str,
    items: &[FoundMail],
    now: DateTime<Utc>,
) -> Result<crate::work::SyncSummary, StoreError> {
    let at = stamp(now);
    let tx = conn.transaction()?;
    let known: HashSet<String> = {
        let mut stmt = tx.prepare("SELECT id FROM mail_item WHERE source = ?1")?;
        let ids = stmt.query_map([source], |r| r.get(0))?;
        ids.collect::<Result<_, _>>()?
    };
    let mut summary = crate::work::SyncSummary::default();
    let mut listed = HashSet::new();
    for item in items {
        if !listed.insert(item.id.as_str()) {
            continue;
        }
        if !known.contains(&item.id) {
            summary.new += 1;
        }
        let noticed = item.promise.as_ref();
        tx.execute(
            "INSERT INTO mail_item (id, source, kind, thread, subject, person, address, at, snippet,
                                    url, promise_title, promise_when, promise_has_time,
                                    promise_deadline, seen_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(id) DO UPDATE SET
               kind = excluded.kind, thread = excluded.thread, subject = excluded.subject,
               person = excluded.person, address = excluded.address, at = excluded.at,
               snippet = excluded.snippet, url = excluded.url,
               promise_title = excluded.promise_title, promise_when = excluded.promise_when,
               promise_has_time = excluded.promise_has_time,
               promise_deadline = excluded.promise_deadline,
               seen_at = excluded.seen_at, gone_at = NULL",
            params![
                item.id,
                source,
                item.kind.as_str(),
                item.thread,
                item.subject,
                item.person,
                item.address,
                item.at,
                item.snippet,
                item.url,
                noticed.map(|p| p.title.as_str()),
                noticed.and_then(|p| p.when).map(promises::stored_when),
                noticed.and_then(|p| p.when).is_some_and(|w| w.time.is_some()),
                noticed.is_some_and(|p| p.is_deadline),
                at,
            ],
        )?;
    }
    for id in known.iter().filter(|id| !listed.contains(id.as_str())) {
        summary.gone += tx.execute(
            "UPDATE mail_item SET gone_at = ?2 WHERE id = ?1 AND gone_at IS NULL",
            params![id, at],
        )? as u32;
    }
    tx.execute(
        "DELETE FROM mail_item WHERE source = ?1 AND gone_at IS NOT NULL
           AND (promise_id IS NULL
                OR promise_id IN (SELECT id FROM promise WHERE state IN ('done','let_go')))",
        [source],
    )?;
    tx.commit()?;
    Ok(summary)
}

/// Forgets everything from `source`. Promises made from it stay.
pub fn forget(conn: &Connection, source: &str) -> Result<(), StoreError> {
    conn.execute("DELETE FROM mail_item WHERE source = ?1", [source])?;
    Ok(())
}

/// Everything the Inbox screen shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InboxView {
    pub account: MailAccount,
    /// Waiting on your reply, oldest first: whoever waited longest comes first.
    pub reply: Vec<MailItem>,
    /// Your questions waiting on others, oldest first.
    pub waiting: Vec<MailItem>,
    /// Promises you wrote in emails and have not added yet, newest first.
    pub noticed: Vec<MailItem>,
    /// Gone from "needs you" while their promise is still open.
    pub finished: Vec<MailItem>,
    /// Items marked "Not needed" that are still current.
    pub hidden: u32,
}

/// Reads the Inbox screen.
pub fn inbox_view(conn: &Connection) -> Result<InboxView, StoreError> {
    let sql = format!(
        "SELECT {COLUMNS} FROM mail_item m LEFT JOIN promise p ON p.id = m.promise_id
         WHERE m.source = 'gmail' AND m.dismissed_at IS NULL ORDER BY m.at, m.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let all: Vec<MailItem> = stmt.query_map([], read_item)?.collect::<Result<_, _>>()?;
    let (finished, current): (Vec<_>, Vec<_>) = all.into_iter().partition(|i| i.gone);
    let of = |kind: MailKind| -> Vec<MailItem> {
        current.iter().filter(|i| i.kind == kind).cloned().collect()
    };
    // A noticed promise that is already a promise has done its job.
    let mut noticed: Vec<MailItem> = of(MailKind::Promise)
        .into_iter()
        .filter(|i| i.promise.is_none())
        .collect();
    noticed.reverse();
    Ok(InboxView {
        account: gmail_account(conn)?,
        reply: of(MailKind::Reply),
        waiting: of(MailKind::Waiting),
        noticed,
        finished,
        hidden: conn.query_row(
            "SELECT count(*) FROM mail_item
             WHERE source = 'gmail' AND dismissed_at IS NOT NULL AND gone_at IS NULL",
            [],
            |r| r.get(0),
        )?,
    })
}

/// How many emails wait on your reply, for the sidebar.
pub fn replies_owed(conn: &Connection) -> Result<u32, StoreError> {
    Ok(conn.query_row(
        "SELECT count(*) FROM mail_item
         WHERE kind = 'reply' AND gone_at IS NULL AND dismissed_at IS NULL",
        [],
        |r| r.get(0),
    )?)
}

/// "Not needed": hides an item in Bindle. The mailbox is not touched.
pub fn dismiss(conn: &Connection, id: &str, now: DateTime<Utc>) -> Result<(), StoreError> {
    let changed = conn.execute(
        "UPDATE mail_item SET dismissed_at = ?2 WHERE id = ?1",
        params![id, stamp(now)],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

/// Shows a hidden item again (Undo for "Not needed").
pub fn restore(conn: &Connection, id: &str) -> Result<(), StoreError> {
    let changed = conn.execute(
        "UPDATE mail_item SET dismissed_at = NULL WHERE id = ?1",
        [id],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

/// "Make it a promise" or "Add to promises": a work promise in Sort, with
/// who it is for and, for a noticed promise, its day.
pub fn make_promise(
    conn: &mut Connection,
    id: &str,
    now: DateTime<Utc>,
) -> Result<Promise, StoreError> {
    let item = get(conn, id)?;
    if item
        .promise
        .as_ref()
        .is_some_and(|p| !matches!(p.state, State::Done | State::LetGo))
    {
        return Err(StoreError::AlreadyPromised);
    }
    let jot = promise_words(&item);
    let promise = promises::insert(
        conn,
        &jot,
        Origin::Link {
            label: "Gmail",
            url: &item.url,
        },
        Mode::Work,
        now,
    )?;
    conn.execute(
        "UPDATE mail_item SET promise_id = ?2 WHERE id = ?1",
        params![item.id, promise.id],
    )?;
    Ok(promise)
}

/// The words of the promise made from an item.
pub fn promise_words(item: &MailItem) -> Jot {
    let first_name = item
        .person
        .split_whitespace()
        .next()
        .unwrap_or(&item.person);
    let (title, direction, when, is_deadline) = match (&item.kind, &item.noticed) {
        (MailKind::Promise, Some(noticed)) => (
            noticed.title.clone(),
            Direction::Mine,
            noticed.when,
            noticed.is_deadline,
        ),
        (MailKind::Waiting, _) => (
            format!("{first_name} to reply: {}", item.subject),
            Direction::Theirs,
            None,
            false,
        ),
        _ => (
            format!("Reply to {first_name}: {}", item.subject),
            Direction::Mine,
            None,
            false,
        ),
    };
    Jot {
        text: format!("{} ({})", item.subject, item.snippet),
        title,
        when,
        is_deadline,
        amount: None,
        person: Some(item.person.clone()),
        direction: Some(direction),
        strength: Strength::Firm,
        tz: None,
        unsure_words: None,
    }
}

const COLUMNS: &str = "m.id, m.kind, m.subject, m.person, m.address, m.at, m.snippet, m.url, \
                       m.promise_title, m.promise_when, m.promise_has_time, m.promise_deadline, \
                       m.gone_at, p.id, p.state";

fn get(conn: &Connection, id: &str) -> Result<MailItem, StoreError> {
    let sql = format!(
        "SELECT {COLUMNS} FROM mail_item m LEFT JOIN promise p ON p.id = m.promise_id WHERE m.id = ?1"
    );
    conn.query_row(&sql, [id], read_item)
        .optional()?
        .ok_or(StoreError::NotFound)
}

fn read_item(row: &Row) -> rusqlite::Result<MailItem> {
    let kind: String = row.get(1)?;
    let title: Option<String> = row.get(8)?;
    let when = promises::when(row, 9, 10)?;
    let promise_id: Option<String> = row.get(13)?;
    let state: Option<String> = row.get(14)?;
    Ok(MailItem {
        id: row.get(0)?,
        kind: MailKind::from_word(&kind).unwrap_or(MailKind::Reply),
        subject: row.get(2)?,
        person: row.get(3)?,
        address: row.get(4)?,
        at: row.get(5)?,
        snippet: row.get(6)?,
        url: row.get(7)?,
        noticed: title.map(|title| NoticedPromise {
            title,
            when,
            is_deadline: row.get(11).unwrap_or(false),
        }),
        gone: row.get::<_, Option<String>>(12)?.is_some(),
        promise: promise_id
            .zip(state.and_then(|s| State::from_word(&s)))
            .map(|(id, state)| Linked { id, state }),
    })
}

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
#[path = "mail_tests.rs"]
mod tests;
