// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Everything Life Home shows, in one read (handbook P6), plus the top 3 and
//! Nudge. The rules for what counts as late or full live in `bindle-core`.

use crate::promises::{read_row, COLUMNS, FROM};
use crate::StoreError;
use bindle_core::{week_of, Direction, Promise, State, DAY_CAPACITY};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::BTreeSet;
use uuid::Uuid;

/// What Home shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HomeView {
    /// Today's top 3 in order, including ones already done today.
    pub top3: Vec<Promise>,
    /// Up to three promises for today or running late, offered while the top
    /// 3 has room. Only suggestions: nothing is added without the person.
    pub suggestions: Vec<Promise>,
    /// Your open firm promises past their day, oldest first. Theirs are in
    /// `waiting_on_others`; ones in the top 3 show there.
    pub running_late: Vec<Promise>,
    /// Open promises someone owes you, oldest first.
    pub waiting_on_others: Vec<Waiting>,
    /// The rest of your open promises, soonest first, ones with no day last.
    pub coming_up: Vec<Promise>,
    /// Monday to Sunday: how full each day is.
    pub week: Vec<DayLoad>,
    /// How many promises fill a day ([`DAY_CAPACITY`]), for drawing the strip.
    pub day_capacity: usize,
    pub this_week: WeekCounts,
    /// Your open promises for today.
    pub today_count: usize,
    /// Different people your open promises for today or earlier are with.
    pub people_waiting_on_you: usize,
    /// Today has been wrapped up.
    pub wrapped_up: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Waiting {
    pub promise: Promise,
    /// When it was last nudged, if ever.
    pub last_nudged: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DayLoad {
    pub day: NaiveDate,
    pub count: usize,
    /// At or over [`DAY_CAPACITY`].
    pub full: bool,
}

/// What happened since Monday, not counting anything undone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WeekCounts {
    pub kept: u32,
    pub moved: u32,
    pub let_go: u32,
}

/// Reads Home for `today`. `week_start` is Monday 00:00 local, as UTC.
pub fn home(
    conn: &Connection,
    today: NaiveDate,
    week_start: DateTime<Utc>,
) -> Result<HomeView, StoreError> {
    let week = week_of(today);
    let top3_ids = top3_ids(conn, today)?;
    let promises = load(conn, week[0], week[6], today)?;
    let open: Vec<&Promise> = promises.iter().filter(|p| p.state == State::Open).collect();
    // Yours: everything except what someone else owes you.
    let yours: Vec<&Promise> = open
        .iter()
        .copied()
        .filter(|p| p.direction != Some(Direction::Theirs))
        .collect();
    let in_top3 = |p: &Promise| top3_ids.contains(&p.id);
    let pick = |keep: &dyn Fn(&Promise) -> bool| -> Vec<Promise> {
        yours.iter().copied().filter(|p| keep(p)).cloned().collect()
    };

    let top3: Vec<Promise> = top3_ids
        .iter()
        .filter_map(|id| {
            promises
                .iter()
                .find(|p| &p.id == id && matches!(p.state, State::Open | State::Done))
        })
        .cloned()
        .collect();

    let mut running_late = pick(&|p| p.is_running_late(today) && !in_top3(p));
    running_late.sort_by_key(|p| p.day());

    let mut coming_up = pick(&|p| !p.is_running_late(today) && !in_top3(p));
    coming_up.sort_by_key(|p| {
        (
            p.day().is_none(),
            p.day(),
            p.due.or(p.do_on).and_then(|w| w.time),
        )
    });

    let mut suggestions = pick(&|p| !in_top3(p) && p.day().is_some_and(|d| d <= today));
    suggestions.sort_by_key(|p| p.day());
    suggestions.truncate(3usize.saturating_sub(top3.len()));

    let today_count = yours.iter().filter(|p| p.day() == Some(today)).count();
    let people: BTreeSet<&str> = yours
        .iter()
        .filter(|p| p.direction == Some(Direction::Mine) && p.day().is_some_and(|d| d <= today))
        .filter_map(|p| p.person.as_deref())
        .collect();

    Ok(HomeView {
        top3,
        suggestions,
        running_late,
        waiting_on_others: waiting(conn, &open)?,
        coming_up,
        week: week
            .iter()
            .map(|&day| {
                let count = promises
                    .iter()
                    .filter(|p| {
                        p.direction != Some(Direction::Theirs)
                            && matches!(p.state, State::Open | State::Done)
                    })
                    .filter(|p| p.day() == Some(day))
                    .count();
                DayLoad {
                    day,
                    count,
                    full: count >= DAY_CAPACITY,
                }
            })
            .collect(),
        day_capacity: DAY_CAPACITY,
        this_week: week_counts(conn, week_start)?,
        today_count,
        people_waiting_on_you: people.len(),
        wrapped_up: crate::wrap_up::is_wrapped_up(conn, today)?,
    })
}

/// Your open promises for `day`: the count Home and the morning note use.
pub fn today_count(conn: &Connection, day: NaiveDate) -> Result<usize, StoreError> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM promise WHERE state = 'open'
           AND (direction IS NULL OR direction != 'theirs')
           AND substr(coalesce(due, do_on), 1, 10) = ?1",
        [day_text(day)],
        |r| r.get(0),
    )?)
}

/// Open promises, plus done ones from this week or in today's top 3.
fn load(
    conn: &Connection,
    monday: NaiveDate,
    sunday: NaiveDate,
    today: NaiveDate,
) -> Result<Vec<Promise>, StoreError> {
    let sql = format!(
        "SELECT {COLUMNS} {FROM}
         WHERE p.state = 'open'
            OR (p.state = 'done' AND (substr(coalesce(p.due, p.do_on), 1, 10) BETWEEN ?1 AND ?2
                                      OR p.id IN (SELECT promise_id FROM top3 WHERE day = ?3)))
         ORDER BY p.created_at, p.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        params![day_text(monday), day_text(sunday), day_text(today)],
        read_row,
    )?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn waiting(conn: &Connection, open: &[&Promise]) -> Result<Vec<Waiting>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT max(at) FROM promise_event WHERE promise_id = ?1 AND kind = 'nudged'")?;
    open.iter()
        .filter(|p| p.direction == Some(Direction::Theirs))
        .map(|p| {
            let last_nudged = stmt.query_row([&p.id], |row| row.get(0))?;
            Ok(Waiting {
                promise: (*p).clone(),
                last_nudged,
            })
        })
        .collect()
}

fn week_counts(conn: &Connection, week_start: DateTime<Utc>) -> Result<WeekCounts, StoreError> {
    let count = |kind: &str| -> Result<u32, StoreError> {
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM promise_event e WHERE kind = ?1 AND at >= ?2
               AND NOT EXISTS (SELECT 1 FROM promise_event u
                               WHERE u.kind = 'undone' AND json_extract(u.data, '$.undoes') = e.id)",
            params![kind, stamp(week_start)],
            |row| row.get(0),
        )?)
    };
    Ok(WeekCounts {
        kept: count("done")?,
        moved: count("moved")?,
        let_go: count("let_go")?,
    })
}

/// Today's top 3 promise ids, in order.
fn top3_ids(conn: &Connection, day: NaiveDate) -> Result<Vec<String>, StoreError> {
    let mut stmt = conn.prepare("SELECT promise_id FROM top3 WHERE day = ?1 ORDER BY position")?;
    let rows = stmt.query_map([day_text(day)], |row| row.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Puts an open promise in the day's top 3, in the first free place.
pub fn add_to_top3(
    conn: &mut Connection,
    day: NaiveDate,
    id: &str,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    let tx = conn.transaction()?;
    let state: Option<String> = tx
        .query_row("SELECT state FROM promise WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?;
    match state.as_deref() {
        None => return Err(StoreError::NotFound),
        Some("open") => {}
        Some(_) => return Err(StoreError::NotForTopThree),
    }
    let ids = top3_ids(&tx, day)?;
    if ids.iter().any(|i| i == id) {
        return Ok(());
    }
    if ids.len() >= 3 {
        return Err(StoreError::TopThreeFull);
    }
    tx.execute(
        "INSERT INTO top3 (day, position, promise_id) VALUES (?1, ?2, ?3)",
        params![day_text(day), ids.len() + 1, id],
    )?;
    event(
        &tx,
        id,
        "top3_added",
        now,
        &serde_json::json!({ "day": day_text(day) }),
    )?;
    tx.commit()?;
    Ok(())
}

/// Takes a promise out of the day's top 3; the others close up the gap.
pub fn remove_from_top3(
    conn: &mut Connection,
    day: NaiveDate,
    id: &str,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    let tx = conn.transaction()?;
    let ids = top3_ids(&tx, day)?;
    if !ids.iter().any(|i| i == id) {
        return Ok(());
    }
    tx.execute("DELETE FROM top3 WHERE day = ?1", [day_text(day)])?;
    for (position, kept) in ids.iter().filter(|i| *i != id).enumerate() {
        tx.execute(
            "INSERT INTO top3 (day, position, promise_id) VALUES (?1, ?2, ?3)",
            params![day_text(day), position + 1, kept],
        )?;
    }
    event(
        &tx,
        id,
        "top3_removed",
        now,
        &serde_json::json!({ "day": day_text(day) }),
    )?;
    tx.commit()?;
    Ok(())
}

/// Records a nudge on a promise someone owes you. Bindle never sends it: the
/// person sends their own message (handbook principle 6).
pub fn nudge(conn: &Connection, id: &str, now: DateTime<Utc>) -> Result<(), StoreError> {
    let row: Option<(String, Option<String>)> = conn
        .query_row(
            "SELECT state, direction FROM promise WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    match row {
        None => Err(StoreError::NotFound),
        Some((state, Some(direction))) if state == "open" && direction == "theirs" => {
            event(conn, id, "nudged", now, &serde_json::json!({}))
        }
        Some(_) => Err(StoreError::NotNudgeable),
    }
}

fn event(
    conn: &Connection,
    id: &str,
    kind: &str,
    now: DateTime<Utc>,
    data: &serde_json::Value,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO promise_event (id, promise_id, at, kind, data) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            Uuid::now_v7().to_string(),
            id,
            stamp(now),
            kind,
            data.to_string()
        ],
    )?;
    Ok(())
}

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn day_text(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
#[path = "home_tests.rs"]
mod tests;
