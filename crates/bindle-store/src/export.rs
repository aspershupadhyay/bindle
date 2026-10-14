// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Export: everything the person has put in Bindle, in open formats they can
//! take anywhere (handbook principle 5). JSON holds every table that is
//! theirs; CSV holds the promises for a spreadsheet.

use crate::StoreError;
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::types::ValueRef;
use rusqlite::Connection;
use serde_json::{json, Map, Value};

/// The tables in a JSON export, in the order they are written.
const TABLES: &[&str] = &[
    "promise",
    "promise_event",
    "person",
    "source",
    "top3",
    "day_log",
    "setting",
];

/// Every table that is the person's, as one JSON value.
pub fn to_json(conn: &Connection, now: DateTime<Utc>) -> Result<Value, StoreError> {
    let mut tables = Map::new();
    for table in TABLES {
        tables.insert((*table).to_string(), Value::Array(rows(conn, table)?));
    }
    Ok(json!({
        "format": "bindle-export",
        "version": 1,
        "schema_version": crate::latest_version(),
        "exported_at": now.to_rfc3339_opts(SecondsFormat::Secs, true),
        "tables": tables,
    }))
}

/// Every row of `table` as JSON objects keyed by column name.
fn rows(conn: &Connection, table: &str) -> Result<Vec<Value>, StoreError> {
    // `table` only ever comes from TABLES, never from outside.
    let mut stmt = conn.prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))?;
    let names: Vec<String> = stmt.column_names().iter().map(|n| n.to_string()).collect();
    let mut out = Vec::new();
    let mut result = stmt.query([])?;
    while let Some(row) = result.next()? {
        let mut object = Map::new();
        for (i, name) in names.iter().enumerate() {
            object.insert(name.clone(), value(row.get_ref(i)?));
        }
        out.push(Value::Object(object));
    }
    Ok(out)
}

fn value(raw: ValueRef) -> Value {
    match raw {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(f) => json!(f),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(b) => json!(b),
    }
}

const CSV_COLUMNS: &str = "title,state,direction,strength,person,first_step,do_on,due,amount,currency,created_at,closed_at";

/// Every promise as CSV, one row each, oldest first.
pub fn promises_csv(conn: &Connection) -> Result<String, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT p.title, p.state, p.direction, p.strength, person.display_name, p.first_step,
                p.do_on, p.due, p.amount_minor, p.currency, p.created_at, p.closed_at
         FROM promise p LEFT JOIN person ON person.id = p.person_id
         ORDER BY p.created_at, p.id",
    )?;
    let mut out = format!("{CSV_COLUMNS}\n");
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let mut cells = Vec::with_capacity(12);
        for i in 0..12 {
            let cell = match row.get_ref(i)? {
                ValueRef::Null => String::new(),
                // Amounts are stored in minor units: 50000 paise is 500.00.
                ValueRef::Integer(minor) if i == 8 => {
                    format!("{}.{:02}", minor / 100, minor.abs() % 100)
                }
                ValueRef::Integer(n) => n.to_string(),
                ValueRef::Real(f) => f.to_string(),
                ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                ValueRef::Blob(_) => String::new(),
            };
            cells.push(csv_cell(&cell));
        }
        out.push_str(&cells.join(","));
        out.push('\n');
    }
    Ok(out)
}

/// Quotes a cell when it needs it. A cell starting with `=`, `+`, `-` or `@`
/// gets a leading `'` so a spreadsheet shows it rather than running it.
fn csv_cell(text: &str) -> String {
    let safe = if text.starts_with(['=', '+', '-', '@']) {
        format!("'{text}")
    } else {
        text.to_string()
    };
    if safe.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", safe.replace('"', "\"\""))
    } else {
        safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;
    use crate::promises::{jot_down, Jot};
    use bindle_core::Money;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
    }

    #[test]
    fn a_json_export_holds_every_promise_and_its_history() {
        let mut conn = open_in_memory().expect("open");
        jot_down(&mut conn, &Jot::plain("Call mom"), now()).expect("jot");
        let export = to_json(&conn, now()).expect("export");
        assert_eq!(export["format"], "bindle-export");
        assert_eq!(export["tables"]["promise"][0]["title"], "Call mom");
        assert_eq!(export["tables"]["promise_event"][0]["kind"], "created");
    }

    #[test]
    fn csv_quotes_commas_and_defuses_formulas() {
        let mut conn = open_in_memory().expect("open");
        let mut jot = Jot::plain("Pay Rohan, for dinner");
        jot.amount = Some(Money {
            minor: 50050,
            currency: "INR".into(),
        });
        jot_down(&mut conn, &jot, now()).expect("jot");
        jot_down(&mut conn, &Jot::plain("=HYPERLINK(\"x\")"), now()).expect("jot");
        let csv = promises_csv(&conn).expect("csv");
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], CSV_COLUMNS);
        assert!(lines[1].starts_with("\"Pay Rohan, for dinner\",unsorted,"));
        assert!(lines[1].contains(",500.50,INR,"));
        assert!(lines[2].starts_with("\"'=HYPERLINK(\"\"x\"\")\""));
    }
}
