// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The Board (v2 design, Board; handbook P14): work promises in lanes.
//!
//! The lanes come from what Bindle already knows, so nothing is kept twice:
//! Ideas are promises kept for later, Next are open ones you will do,
//! Making are open ones you are working on now, Waiting are ones someone
//! else owes you, and Shipped are the ones kept this week.

use crate::promises::{self, COLUMNS, FROM};
use crate::StoreError;
use bindle_core::Promise;
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection};
use serde::Serialize;

/// A promise on the Board, with where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Card {
    pub promise: Promise,
    /// "GitHub kunal-m/latch#42" or "Gmail" when made from Code or Inbox.
    pub from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoardView {
    pub ideas: Vec<Card>,
    pub next: Vec<Card>,
    pub making: Vec<Card>,
    pub waiting: Vec<Card>,
    /// Kept since the start of the week, newest first.
    pub shipped: Vec<Card>,
}

/// Reads the Board: work promises only. `week_start` decides what counts
/// as shipped this week.
pub fn board_view(conn: &Connection, week_start: DateTime<Utc>) -> Result<BoardView, StoreError> {
    let cards = |filter: &str,
                 order: &str,
                 args: &[&dyn rusqlite::ToSql]|
     -> Result<Vec<Card>, StoreError> {
        let sql = format!(
            "SELECT {COLUMNS}, CASE WHEN s.kind = 'link' THEN s.label END
             {FROM} LEFT JOIN source s ON s.id = p.source_id
             WHERE p.mode = 'work' AND {filter} ORDER BY {order}"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(args, |row| {
            Ok(Card {
                promise: promises::read_row(row)?,
                from: row.get(17)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    };
    let soonest = "coalesce(p.due, p.do_on, '9999'), p.created_at";
    let since = week_start.to_rfc3339_opts(SecondsFormat::Millis, true);
    Ok(BoardView {
        ideas: cards("p.state = 'later'", "p.created_at", &[])?,
        next: cards(
            "p.state = 'open' AND p.board_lane IS NULL AND coalesce(p.direction, '') <> 'theirs'",
            soonest,
            &[],
        )?,
        making: cards("p.state = 'open' AND p.board_lane = 'making'", soonest, &[])?,
        waiting: cards(
            "p.state = 'open' AND p.board_lane IS NULL AND p.direction = 'theirs'",
            soonest,
            &[],
        )?,
        shipped: cards(
            "p.state = 'done' AND p.closed_at >= ?1",
            "p.closed_at DESC",
            &[&since],
        )?,
    })
}

/// Moves an open promise into Making, or back out to Next or Waiting.
pub fn set_making(conn: &Connection, id: &str, making: bool) -> Result<(), StoreError> {
    let state: Option<String> = conn
        .query_row("SELECT state FROM promise WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            e => Err(e),
        })?;
    match state.as_deref() {
        None => Err(StoreError::NotFound),
        Some("open") => {
            conn.execute(
                "UPDATE promise SET board_lane = ?2 WHERE id = ?1",
                params![id, making.then_some("making")],
            )?;
            Ok(())
        }
        Some(_) => Err(StoreError::NotForMaking),
    }
}

#[cfg(test)]
#[path = "board_tests.rs"]
mod tests;
