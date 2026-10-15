// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Insights and the Work Desk (v2 design, Desk; handbook P14).
//!
//! The Desk answers three questions about work: what needs you now, how
//! long people wait on you, and how the week is going. Waits are kept as
//! samples ([`WaitSample`]) taken at each look at Gmail and GitHub, so the
//! history grows by itself. Numbers are medians, compared with your own
//! last four weeks, never with anyone else (handbook principle 3).

use crate::mail::{self, MailAccount};
use crate::promises::{self, COLUMNS, FROM};
use crate::work::{self, GithubAccount, Linked, WorkKind};
use crate::StoreError;
use bindle_core::{week_of, Promise, State};
use chrono::{DateTime, Duration, NaiveDate, SecondsFormat, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// What kind of wait a sample measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitKind {
    /// Someone emailed you; you replied.
    EmailReply,
    /// Someone opened a pull request asking for your review; you reviewed.
    Review,
}

impl WaitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmailReply => "email_reply",
            Self::Review => "review",
        }
    }
}

/// One answer: when it was asked and when you answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitSample {
    /// Stable, so the same answer is counted once however often it is seen.
    pub id: String,
    pub kind: WaitKind,
    pub person: String,
    pub asked_at: DateTime<Utc>,
    pub answered_at: DateTime<Utc>,
}

/// Saves new samples. Ones already saved are left as they are; answers
/// before the question (clock trouble) are skipped.
pub fn record_waits(conn: &mut Connection, samples: &[WaitSample]) -> Result<u32, StoreError> {
    let tx = conn.transaction()?;
    let mut added = 0;
    for sample in samples.iter().filter(|s| s.answered_at >= s.asked_at) {
        let hours = (sample.answered_at - sample.asked_at).num_seconds() as f64 / 3600.0;
        added += tx.execute(
            "INSERT OR IGNORE INTO wait_sample (id, kind, person, asked_at, answered_at, hours)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                sample.id,
                sample.kind.as_str(),
                sample.person,
                stamp(sample.asked_at),
                stamp(sample.answered_at),
                hours
            ],
        )? as u32;
    }
    tx.commit()?;
    Ok(added)
}

/// The ids already saved for `kind`, so a source can skip work it did before.
pub fn known_waits(conn: &Connection, kind: WaitKind) -> Result<Vec<String>, StoreError> {
    let mut stmt = conn.prepare("SELECT id FROM wait_sample WHERE kind = ?1")?;
    let ids = stmt.query_map([kind.as_str()], |r| r.get(0))?;
    Ok(ids.collect::<Result<_, _>>()?)
}

/// The median wait in hours for answers given in `[from, to)`, if any.
pub fn median_hours(
    conn: &Connection,
    kind: WaitKind,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Option<f64>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT hours FROM wait_sample WHERE kind = ?1 AND answered_at >= ?2 AND answered_at < ?3
         ORDER BY hours",
    )?;
    let hours: Vec<f64> = stmt
        .query_map(params![kind.as_str(), stamp(from), stamp(to)], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(median(&hours))
}

fn median(sorted: &[f64]) -> Option<f64> {
    match sorted.len() {
        0 => None,
        n if n % 2 == 1 => Some(sorted[n / 2]),
        n => Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0),
    }
}

/// How long people wait on you, for one kind of wait.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WaitTrend {
    /// Median over the last 7 days.
    pub recent: Option<f64>,
    /// Median over the four weeks before that, to compare with.
    pub before: Option<f64>,
    /// Median of each of the last 8 weeks (Monday to Sunday), oldest first.
    pub weekly: Vec<Option<f64>>,
}

/// Work promises kept, this week and in the weeks before.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Shipped {
    pub this_week: u32,
    /// Kept in each of the last 8 weeks, oldest first; the last is this week.
    pub weekly: Vec<u32>,
    pub from_code: u32,
    pub from_inbox: u32,
    pub other: u32,
}

/// Something that needs you now, from any source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeskItem {
    /// The work item or email id, for Make it a promise.
    pub id: String,
    /// "github" or "gmail".
    pub source: String,
    /// "review", "issue" or "reply".
    pub kind: String,
    pub title: String,
    /// Who is waiting: the pull request's author, the sender.
    pub person: String,
    /// "latch #42" for GitHub; empty for email.
    pub reference: String,
    /// When it started needing you.
    pub at: String,
    pub url: String,
    pub promise: Option<Linked>,
}

/// How full each day of the week is with work, and how much of it is done.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkDay {
    pub day: NaiveDate,
    pub open: u32,
    pub done: u32,
    /// The same promises by where they came from: email, code, content
    /// (YouTube) and everything else.
    pub email: u32,
    pub code: u32,
    pub content: u32,
    pub other: u32,
}

/// Where a work promise came from, by the label of its link.
fn source_of(label: Option<&str>) -> &'static str {
    match label {
        Some(l) if l.starts_with("GitHub") => "code",
        Some("Gmail") => "email",
        Some(l) if l.starts_with("YouTube") => "content",
        _ => "other",
    }
}

/// Everything the Desk shows.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeskView {
    /// Oldest first: whoever has waited longest comes first.
    pub needs_you: Vec<DeskItem>,
    pub email_reply: WaitTrend,
    pub review_wait: WaitTrend,
    pub shipped: Shipped,
    pub week: Vec<WorkDay>,
    pub github: GithubAccount,
    pub gmail: MailAccount,
    pub reviews_older_than_a_day: u32,
    pub replies_owed: u32,
}

/// Reads the Desk. `week_start` is this Monday's local midnight, as UTC.
pub fn desk_view(
    conn: &Connection,
    now: DateTime<Utc>,
    today: NaiveDate,
    week_start: DateTime<Utc>,
) -> Result<DeskView, StoreError> {
    let code = work::code_view(conn, now)?;
    let inbox = mail::inbox_view(conn)?;
    let mut needs_you: Vec<DeskItem> = code
        .reviews
        .iter()
        .chain(&code.issues)
        .map(|item| DeskItem {
            id: item.id.clone(),
            source: "github".into(),
            kind: match item.kind {
                WorkKind::Review => "review",
                _ => "issue",
            }
            .into(),
            title: item.title.clone(),
            person: item.author.clone(),
            reference: format!(
                "{} #{}",
                item.repo.rsplit('/').next().unwrap_or(&item.repo),
                item.number
            ),
            at: item.created_at.clone(),
            url: item.url.clone(),
            promise: item.promise.clone(),
        })
        .chain(inbox.reply.iter().map(|item| DeskItem {
            id: item.id.clone(),
            source: "gmail".into(),
            kind: "reply".into(),
            title: item.subject.clone(),
            person: item.person.clone(),
            reference: String::new(),
            at: item.at.clone(),
            url: item.url.clone(),
            promise: item.promise.clone(),
        }))
        .collect();
    // Times come from different sources in slightly different forms; compare instants.
    needs_you.sort_by_key(|i| {
        DateTime::parse_from_rfc3339(&i.at)
            .map(|t| t.timestamp())
            .unwrap_or(i64::MAX)
    });

    Ok(DeskView {
        needs_you,
        email_reply: trend(conn, WaitKind::EmailReply, now, week_start)?,
        review_wait: trend(conn, WaitKind::Review, now, week_start)?,
        shipped: shipped(conn, week_start)?,
        week: work_week(conn, today)?,
        github: code.account,
        gmail: inbox.account,
        reviews_older_than_a_day: code.reviews_older_than_a_day,
        replies_owed: inbox.reply.len() as u32,
    })
}

fn trend(
    conn: &Connection,
    kind: WaitKind,
    now: DateTime<Utc>,
    week_start: DateTime<Utc>,
) -> Result<WaitTrend, StoreError> {
    let week_ago = now - Duration::days(7);
    let weekly = (0..8)
        .rev()
        .map(|back| {
            let from = week_start - Duration::weeks(back);
            median_hours(conn, kind, from, from + Duration::weeks(1))
        })
        .collect::<Result<_, _>>()?;
    Ok(WaitTrend {
        recent: median_hours(conn, kind, week_ago, now + Duration::seconds(1))?,
        before: median_hours(conn, kind, week_ago - Duration::weeks(4), week_ago)?,
        weekly,
    })
}

fn shipped(conn: &Connection, week_start: DateTime<Utc>) -> Result<Shipped, StoreError> {
    let count = |from: DateTime<Utc>, to: DateTime<Utc>| -> Result<u32, StoreError> {
        Ok(conn.query_row(
            "SELECT count(*) FROM promise WHERE mode = 'work' AND state = 'done'
               AND closed_at >= ?1 AND closed_at < ?2",
            [stamp(from), stamp(to)],
            |r| r.get(0),
        )?)
    };
    let weekly = (0..8)
        .rev()
        .map(|back| {
            let from = week_start - Duration::weeks(back);
            count(from, from + Duration::weeks(1))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let by_source = |label: &str| -> Result<u32, StoreError> {
        Ok(conn.query_row(
            "SELECT count(*) FROM promise p JOIN source s ON s.id = p.source_id
             WHERE p.mode = 'work' AND p.state = 'done' AND p.closed_at >= ?1
               AND s.kind = 'link' AND s.label LIKE ?2",
            [stamp(week_start), label.to_string()],
            |r| r.get(0),
        )?)
    };
    let this_week = *weekly.last().unwrap_or(&0);
    let from_code = by_source("GitHub %")?;
    let from_inbox = by_source("Gmail")?;
    Ok(Shipped {
        this_week,
        weekly,
        from_code,
        from_inbox,
        other: this_week.saturating_sub(from_code + from_inbox),
    })
}

/// Work promises planned for, or kept on, each day of this week.
fn work_week(conn: &Connection, today: NaiveDate) -> Result<Vec<WorkDay>, StoreError> {
    let sql = format!(
        "SELECT {COLUMNS} {FROM} WHERE p.mode = 'work' AND p.state IN ('open','done')
           AND coalesce(p.direction, '') <> 'theirs'"
    );
    let mut stmt = conn.prepare(&sql)?;
    let work: Vec<Promise> = stmt
        .query_map([], promises::read_row)?
        .collect::<Result<_, _>>()?;
    let mut labels = std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT p.id, s.label FROM promise p JOIN source s ON s.id = p.source_id
             WHERE p.mode = 'work' AND s.kind = 'link'",
        )?;
        for row in stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
        })? {
            let (id, label) = row?;
            labels.insert(id, label);
        }
    }
    Ok(week_of(today)
        .iter()
        .map(|&day| {
            let on_day: Vec<&Promise> = work.iter().filter(|p| p.day() == Some(day)).collect();
            let from = |source: &str| {
                on_day
                    .iter()
                    .filter(|p| source_of(labels.get(&p.id).and_then(|l| l.as_deref())) == source)
                    .count() as u32
            };
            WorkDay {
                day,
                open: on_day.iter().filter(|p| p.state == State::Open).count() as u32,
                done: on_day.iter().filter(|p| p.state == State::Done).count() as u32,
                email: from("email"),
                code: from("code"),
                content: from("content"),
                other: from("other"),
            }
        })
        .collect())
}

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
#[path = "insights_tests.rs"]
mod tests;
