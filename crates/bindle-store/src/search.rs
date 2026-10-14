// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Search across every promise, in every state (handbook P6). Words match
//! from their start, in any case and without accents: "elec" finds
//! "Electricity". Every word must match. The best matches come first.

use crate::promises::{read_row, COLUMNS, FROM};
use crate::StoreError;
use bindle_core::Promise;
use rusqlite::{params, Connection};

/// Finds promises whose title, first step, note or person match `query`.
/// An empty query finds nothing.
pub fn search(conn: &Connection, query: &str, limit: usize) -> Result<Vec<Promise>, StoreError> {
    let Some(fts) = to_fts(query) else {
        return Ok(Vec::new());
    };
    let sql = format!(
        "SELECT {COLUMNS} {FROM}
         JOIN search_index s ON s.rowid = p.rowid
         WHERE search_index MATCH ?1
         ORDER BY bm25(search_index, 0.0, 0.0, 10.0, 1.0), p.created_at DESC
         LIMIT ?2"
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let rows = stmt.query_map(params![fts, limit as i64], read_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Turns typed words into an FTS5 query: each word quoted (so symbols are
/// never read as query syntax) and matched as a prefix, all required.
fn to_fts(query: &str) -> Option<String> {
    let words: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{w}\"*"))
        .collect();
    (!words.is_empty()).then(|| words.join(" "))
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
