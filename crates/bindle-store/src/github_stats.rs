// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! What the Code screen shows beyond what needs you: your repositories and
//! their checks, a check that keeps failing, sixteen weeks of contributions
//! and pull requests merged each week. Each look replaces the whole picture,
//! so it is kept as one snapshot in the settings table, like the account.

use crate::StoreError;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

const KEY: &str = "github_stats";

/// How a repository's recent checks went.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Checks {
    /// The latest runs passed.
    Passing,
    /// The latest run failed, and so did the one before.
    Failing,
    /// Some recent runs failed and others passed on the same branch.
    Flaky,
}

/// One of your repositories, most recently active first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repo {
    pub full_name: String,
    pub language: Option<String>,
    pub stars: u32,
    pub private: bool,
    pub open_prs: u32,
    pub open_issues: u32,
    /// `None` when the repository runs no checks Bindle can see.
    pub checks: Option<Checks>,
    pub url: String,
}

/// A check that failed more than once lately.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailingCheck {
    pub repo: String,
    pub workflow: String,
    pub failed: u32,
    pub of: u32,
    /// The latest failed run, to open its logs.
    pub url: String,
}

/// Everything in one look.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GithubStats {
    pub repos: Vec<Repo>,
    pub failing: Option<FailingCheck>,
    /// Contributions per day, oldest first: ("2026-06-24", 3).
    pub days: Vec<(String, u32)>,
    /// Pull requests of yours merged in each of the last eight weeks, oldest first.
    pub merged_weeks: Vec<u32>,
}

impl GithubStats {
    /// Days in a row up to today with at least one contribution.
    pub fn streak(&self) -> u32 {
        let mut days = self.days.iter().rev().peekable();
        // Today may not have one yet; that does not break yesterday's run.
        if days.peek().is_some_and(|(_, n)| *n == 0) {
            days.next();
        }
        days.take_while(|(_, n)| *n > 0).count() as u32
    }

    /// True when this week merged more than any of the seven before.
    pub fn best_week(&self) -> bool {
        match self.merged_weeks.split_last() {
            Some((last, before)) => *last > 0 && before.iter().all(|w| last > w),
            None => false,
        }
    }
}

/// The last saved look, or nothing yet.
pub fn load(conn: &Connection) -> Result<GithubStats, StoreError> {
    let saved: Option<String> = conn
        .query_row("SELECT value FROM setting WHERE key = ?1", [KEY], |r| {
            r.get(0)
        })
        .optional()?;
    Ok(saved
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default())
}

pub fn save(conn: &Connection, stats: &GithubStats) -> Result<(), StoreError> {
    let json = serde_json::to_string(stats).expect("stats serialise");
    conn.execute(
        "INSERT INTO setting (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [KEY, json.as_str()],
    )?;
    Ok(())
}

/// Forgets the snapshot, as when GitHub is disconnected.
pub fn forget(conn: &Connection) -> Result<(), StoreError> {
    conn.execute("DELETE FROM setting WHERE key = ?1", [KEY])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn days(counts: &[u32]) -> Vec<(String, u32)> {
        counts.iter().map(|n| (String::new(), *n)).collect()
    }

    #[test]
    fn a_streak_forgives_today_but_not_yesterday() {
        let mut stats = GithubStats {
            days: days(&[1, 0, 2, 3, 1, 0]),
            ..Default::default()
        };
        assert_eq!(stats.streak(), 3);
        stats.days = days(&[1, 2, 0]);
        assert_eq!(stats.streak(), 2);
        stats.days = days(&[1, 0, 0]);
        assert_eq!(stats.streak(), 0);
    }

    #[test]
    fn the_best_week_beats_every_other() {
        let mut stats = GithubStats {
            merged_weeks: vec![2, 4, 3, 7],
            ..Default::default()
        };
        assert!(stats.best_week());
        stats.merged_weeks = vec![2, 7, 3, 7];
        assert!(!stats.best_week());
    }

    #[test]
    fn a_snapshot_is_saved_and_forgotten() {
        let conn = crate::open_in_memory().unwrap();
        assert_eq!(load(&conn).unwrap(), GithubStats::default());
        let stats = GithubStats {
            merged_weeks: vec![1, 2],
            ..Default::default()
        };
        save(&conn, &stats).unwrap();
        assert_eq!(load(&conn).unwrap(), stats);
        forget(&conn).unwrap();
        assert_eq!(load(&conn).unwrap(), GithubStats::default());
    }
}
