// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Ours: promises shared in a space, kept in step through Bindle Cloud
//! (handbook P15).
//!
//! Sync works field by field. [`collect_changes`] compares each shared
//! promise with what was last agreed ([`sync_field`]) and turns every
//! difference into a [`FieldOp`] stamped by a hybrid logical clock; those go
//! into an outbox until the relay has them. [`apply_changes`] takes changes
//! from other devices and keeps a field's new value only when its stamp is
//! later than the one already agreed, so every device ends up the same
//! whatever order changes arrive in. Nothing here talks to the network or
//! sees a key: sealing happens in `bindle-cloud`.
//!
//! [`sync_field`]: https://github.com/aspershupadhyay/bindle/blob/main/crates/bindle-store/migrations/0010_cloud_sync.sql

use crate::{people, StoreError};
use bindle_sync::{Clock, FieldOp, Stamp};
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

/// A shared space on this device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub relay: Option<String>,
    pub promises: u32,
    pub synced_at: Option<String>,
    /// Changes waiting to go up.
    pub waiting: u32,
}

/// Someone who carries a space, as their devices announce themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Member {
    pub account: String,
    pub name: String,
    pub joined_at: String,
}

/// The promise columns that travel, by their field name. Each is plain
/// data; the person travels by name and is matched on arrival.
const COLUMNS: &[&str] = &[
    "title",
    "state",
    "direction",
    "strength",
    "mode",
    "first_step",
    "do_on",
    "do_on_has_time",
    "due",
    "due_has_time",
    "amount_minor",
    "currency",
    "note",
    "created_at",
    "closed_at",
    "created_by",
    "done_by",
];

fn stamp_text(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Adds a shared space this device holds the key to.
pub fn add_space(
    conn: &Connection,
    id: &str,
    name: &str,
    relay: &str,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO space (id, kind, name, created_at, relay) VALUES (?1, 'ours', ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, relay = excluded.relay",
        params![id, name.trim(), stamp_text(now), relay],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO sync_cursor (space_id, pulled) VALUES (?1, 0)",
        [id],
    )?;
    Ok(())
}

/// Every shared space on this device.
pub fn spaces(conn: &Connection) -> Result<Vec<Space>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.relay,
                (SELECT count(*) FROM promise p WHERE p.space_id = s.id AND p.state <> 'let_go'),
                c.synced_at,
                (SELECT count(*) FROM sync_outbox o WHERE o.space_id = s.id)
         FROM space s LEFT JOIN sync_cursor c ON c.space_id = s.id
         WHERE s.kind = 'ours' ORDER BY s.created_at, s.id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Space {
            id: r.get(0)?,
            name: r.get(1)?,
            relay: r.get(2)?,
            promises: r.get(3)?,
            synced_at: r.get(4)?,
            waiting: r.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn check_space(conn: &Connection, id: &str) -> Result<(), StoreError> {
    let found: Option<String> = conn
        .query_row(
            "SELECT id FROM space WHERE id = ?1 AND kind = 'ours'",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    found.map(|_| ()).ok_or(StoreError::NoSuchSpace)
}

/// Puts a promise in a shared space, or (`None`) back in Mine. Taking one
/// back keeps the others' copies as they are: nothing is deleted anywhere.
pub fn share(
    conn: &Connection,
    promise_id: &str,
    space: Option<&str>,
    account: &str,
) -> Result<(), StoreError> {
    if let Some(space) = space {
        check_space(conn, space)?;
    }
    let changed = conn.execute(
        "UPDATE promise SET space_id = ?2, created_by = coalesce(created_by, ?3) WHERE id = ?1",
        params![promise_id, space, account],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    if space.is_none() {
        conn.execute(
            "DELETE FROM sync_field WHERE record = ?1",
            [format!("promise:{promise_id}")],
        )?;
    }
    Ok(())
}

/// Leaves a space on this device: its promises stay here, in Mine.
pub fn leave(conn: &Connection, space: &str) -> Result<(), StoreError> {
    check_space(conn, space)?;
    conn.execute(
        "UPDATE promise SET space_id = NULL WHERE space_id = ?1",
        [space],
    )?;
    for table in ["sync_field", "sync_outbox", "sync_cursor"] {
        conn.execute(&format!("DELETE FROM {table} WHERE space_id = ?1"), [space])?;
    }
    conn.execute("DELETE FROM member WHERE space_id = ?1", [space])?;
    conn.execute("DELETE FROM space WHERE id = ?1", [space])?;
    Ok(())
}

/// The people in a space, by when they joined.
pub fn members(conn: &Connection, space: &str) -> Result<Vec<Member>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT account_id, display_name, joined_at FROM member WHERE space_id = ?1 ORDER BY joined_at, account_id",
    )?;
    let rows = stmt.query_map([space], |r| {
        Ok(Member {
            account: r.get(0)?,
            name: r.get(1)?,
            joined_at: r.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// A promise in a space, with the accounts that added it and kept it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedRow {
    pub promise: bindle_core::Promise,
    pub created_by: Option<String>,
    pub done_by: Option<String>,
}

/// Every promise in a space that is not let go, soonest first.
pub fn shared_promises(conn: &Connection, space: &str) -> Result<Vec<SharedRow>, StoreError> {
    check_space(conn, space)?;
    let mut stmt = conn.prepare(
        "SELECT id, created_by, done_by FROM promise WHERE space_id = ?1 AND state <> 'let_go'
         ORDER BY coalesce(due, do_on, '9999'), created_at",
    )?;
    let rows: Vec<(String, Option<String>, Option<String>)> = stmt
        .query_map([space], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|(id, created_by, done_by)| {
            Ok(SharedRow {
                promise: crate::promises::get(conn, &id)?,
                created_by,
                done_by,
            })
        })
        .collect()
}

/// A clock for this device that has already seen every stamp in the space.
fn clock(conn: &Connection, space: &str, device: &str, now_ms: u64) -> Result<Clock, StoreError> {
    let mut clock = Clock::new(device);
    let newest: Option<(u64, u32, String)> = conn
        .query_row(
            "SELECT millis, counter, device FROM sync_field WHERE space_id = ?1
             ORDER BY millis DESC, counter DESC LIMIT 1",
            [space],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    if let Some((millis, counter, device)) = newest {
        clock.observe(
            &Stamp {
                millis,
                counter,
                device,
            },
            now_ms,
        );
    }
    Ok(clock)
}

/// The current value of every shared field of every promise in the space,
/// plus this person's member record.
fn current(
    conn: &Connection,
    space: &str,
    account: &str,
    name: &str,
    joined_at: &str,
) -> Result<BTreeMap<String, BTreeMap<String, Option<String>>>, StoreError> {
    let columns = COLUMNS
        .iter()
        .map(|c| format!("p.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT p.id, {columns}, person.display_name FROM promise p
         LEFT JOIN person ON person.id = p.person_id WHERE p.space_id = ?1"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut records = BTreeMap::new();
    let mut rows = stmt.query([space])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let mut fields = BTreeMap::new();
        for (i, column) in COLUMNS.iter().enumerate() {
            fields.insert(column.to_string(), json_of(row.get_ref(i + 1)?));
        }
        fields.insert("person".into(), json_of(row.get_ref(COLUMNS.len() + 1)?));
        records.insert(format!("promise:{id}"), fields);
    }
    if !account.is_empty() {
        let mut me = BTreeMap::new();
        me.insert("name".into(), Some(Value::from(name).to_string()));
        me.insert("joined_at".into(), Some(Value::from(joined_at).to_string()));
        records.insert(format!("member:{account}"), me);
    }
    Ok(records)
}

fn json_of(value: rusqlite::types::ValueRef) -> Option<String> {
    use rusqlite::types::ValueRef;
    match value {
        ValueRef::Null => None,
        ValueRef::Integer(i) => Some(i.to_string()),
        ValueRef::Real(f) => Some(Value::from(f).to_string()),
        ValueRef::Text(t) => Some(Value::from(String::from_utf8_lossy(t).into_owned()).to_string()),
        ValueRef::Blob(_) => None,
    }
}

/// Finds what changed in the space since it was last agreed, stamps each
/// change, remembers it as agreed, and puts the batch in the outbox.
/// Returns how many field changes were found.
pub fn collect_changes(
    conn: &mut Connection,
    space: &str,
    device: &str,
    account: &str,
    name: &str,
    now: DateTime<Utc>,
) -> Result<usize, StoreError> {
    check_space(conn, space)?;
    let now_ms = now.timestamp_millis().max(0) as u64;
    // Who kept a shared promise: whoever marked it done on this device.
    conn.execute(
        "UPDATE promise SET done_by = ?2 WHERE space_id = ?1 AND state = 'done' AND done_by IS NULL",
        params![space, account],
    )?;
    conn.execute(
        "UPDATE promise SET done_by = NULL WHERE space_id = ?1 AND state <> 'done' AND done_by IS NOT NULL",
        [space],
    )?;
    let joined_at: String =
        conn.query_row("SELECT created_at FROM space WHERE id = ?1", [space], |r| {
            r.get(0)
        })?;
    let records = current(conn, space, account, name, &joined_at)?;
    let mut known: HashMap<(String, String), Option<String>> = HashMap::new();
    {
        let mut stmt =
            conn.prepare("SELECT record, field, value FROM sync_field WHERE space_id = ?1")?;
        let rows = stmt.query_map([space], |r| Ok(((r.get(0)?, r.get(1)?), r.get(2)?)))?;
        for row in rows {
            let (key, value) = row?;
            known.insert(key, value);
        }
    }
    let mut clock = clock(conn, space, device, now_ms)?;
    let mut ops = Vec::new();
    for (record, fields) in &records {
        for (field, value) in fields {
            let key = (record.clone(), field.clone());
            if known.get(&key) == Some(value) {
                continue;
            }
            // A field that was never set and is still empty is not news.
            if value.is_none() && !known.contains_key(&key) {
                continue;
            }
            ops.push(FieldOp {
                record: record.clone(),
                field: field.clone(),
                value: value.clone(),
                stamp: clock.tick(now_ms),
            });
        }
    }
    if ops.is_empty() {
        return Ok(0);
    }
    let tx = conn.transaction()?;
    for op in &ops {
        remember(&tx, space, op)?;
    }
    tx.execute(
        "INSERT INTO sync_outbox (space_id, batch) VALUES (?1, ?2)",
        params![space, serde_json::to_string(&ops).expect("ops serialise")],
    )?;
    tx.commit()?;
    Ok(ops.len())
}

fn remember(conn: &Connection, space: &str, op: &FieldOp) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO sync_field (space_id, record, field, value, millis, counter, device)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(space_id, record, field) DO UPDATE SET
           value = excluded.value, millis = excluded.millis,
           counter = excluded.counter, device = excluded.device",
        params![
            space,
            op.record,
            op.field,
            op.value,
            op.stamp.millis,
            op.stamp.counter,
            op.stamp.device
        ],
    )?;
    Ok(())
}

/// The batches waiting to go up, oldest first, as (outbox id, JSON ops).
pub fn outbox(conn: &Connection, space: &str) -> Result<Vec<(i64, String)>, StoreError> {
    let mut stmt =
        conn.prepare("SELECT id, batch FROM sync_outbox WHERE space_id = ?1 ORDER BY id")?;
    let rows = stmt.query_map([space], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Forgets a batch the relay now has.
pub fn sent(conn: &Connection, outbox_id: i64) -> Result<(), StoreError> {
    conn.execute("DELETE FROM sync_outbox WHERE id = ?1", [outbox_id])?;
    Ok(())
}

/// The number of the last batch read from the relay.
pub fn pulled(conn: &Connection, space: &str) -> Result<u64, StoreError> {
    Ok(conn
        .query_row(
            "SELECT pulled FROM sync_cursor WHERE space_id = ?1",
            [space],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0))
}

/// Remembers how far the relay was read, and when.
pub fn set_pulled(
    conn: &Connection,
    space: &str,
    seq: u64,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO sync_cursor (space_id, pulled, synced_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(space_id) DO UPDATE SET pulled = max(pulled, excluded.pulled), synced_at = excluded.synced_at",
        params![space, seq, stamp_text(now)],
    )?;
    Ok(())
}

/// Takes changes from other devices. A change wins only over an older
/// stamp. Promises that arrive for the first time are added. Returns how
/// many fields changed here.
pub fn apply_changes(
    conn: &mut Connection,
    space: &str,
    ops: &[FieldOp],
    now: DateTime<Utc>,
) -> Result<usize, StoreError> {
    check_space(conn, space)?;
    let at = stamp_text(now);
    let tx = conn.transaction()?;
    let mut changed = 0;
    let mut touched = std::collections::BTreeSet::new();
    for op in ops {
        let agreed: Option<Stamp> = tx
            .query_row(
                "SELECT millis, counter, device FROM sync_field WHERE space_id = ?1 AND record = ?2 AND field = ?3",
                params![space, op.record, op.field],
                |r| Ok(Stamp { millis: r.get(0)?, counter: r.get(1)?, device: r.get(2)? }),
            )
            .optional()?;
        if agreed.is_some_and(|stamp| op.stamp <= stamp) {
            continue;
        }
        let written = match op.record.split_once(':') {
            Some(("promise", id)) => {
                write_promise_field(&tx, space, id, &op.field, op.value.as_deref(), &at)?;
                touched.insert(id.to_string());
                true
            }
            Some(("member", account)) => {
                write_member_field(&tx, space, account, &op.field, op.value.as_deref(), &at)?;
                true
            }
            _ => false,
        };
        if written {
            remember(&tx, space, op)?;
            changed += 1;
        }
    }
    for id in touched {
        tx.execute(
            "INSERT INTO promise_event (id, promise_id, at, actor, kind, data)
             VALUES (?1, ?2, ?3, 'cloud', 'received', '{}')",
            params![uuid::Uuid::now_v7().to_string(), id, at],
        )?;
    }
    tx.commit()?;
    Ok(changed)
}

fn write_promise_field(
    conn: &Connection,
    space: &str,
    id: &str,
    field: &str,
    value: Option<&str>,
    at: &str,
) -> Result<(), StoreError> {
    // First news of this promise: add it, then fill it in field by field.
    conn.execute(
        "INSERT OR IGNORE INTO promise (id, title, space_id, created_at, updated_at)
         VALUES (?1, '…', ?2, ?3, ?3)",
        params![id, space, at],
    )?;
    let parsed: Option<Value> = value.and_then(|v| serde_json::from_str(v).ok());
    if field == "person" {
        let person = match parsed.as_ref().and_then(Value::as_str) {
            Some(name) if !name.trim().is_empty() => Some(people::find_or_add(conn, name, at)?),
            _ => None,
        };
        conn.execute(
            "UPDATE promise SET person_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, person, at],
        )?;
        return Ok(());
    }
    let Some(column) = COLUMNS.iter().find(|c| **c == field) else {
        return Ok(());
    };
    let sql_value: rusqlite::types::Value = match parsed {
        None | Some(Value::Null) => rusqlite::types::Value::Null,
        Some(Value::Number(n)) => n
            .as_i64()
            .map(rusqlite::types::Value::Integer)
            .unwrap_or_else(|| rusqlite::types::Value::Real(n.as_f64().unwrap_or_default())),
        Some(Value::String(s)) => rusqlite::types::Value::Text(s),
        Some(other) => rusqlite::types::Value::Text(other.to_string()),
    };
    // Values that break the schema's rules (an unknown state, say) are refused
    // by its CHECKs; such a change is skipped rather than stopping the rest.
    let result = conn.execute(
        &format!("UPDATE promise SET {column} = ?2, updated_at = ?3 WHERE id = ?1"),
        params![id, sql_value, at],
    );
    match result {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(e, _))
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

fn write_member_field(
    conn: &Connection,
    space: &str,
    account: &str,
    field: &str,
    value: Option<&str>,
    at: &str,
) -> Result<(), StoreError> {
    let text = value
        .and_then(|v| serde_json::from_str::<Value>(v).ok())
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default();
    conn.execute(
        "INSERT OR IGNORE INTO member (space_id, account_id, display_name, role, joined_at)
         VALUES (?1, ?2, '', 'member', ?3)",
        params![space, account, at],
    )?;
    match field {
        "name" => conn.execute(
            "UPDATE member SET display_name = ?3 WHERE space_id = ?1 AND account_id = ?2",
            params![space, account, text],
        )?,
        "joined_at" if !text.is_empty() => conn.execute(
            "UPDATE member SET joined_at = ?3 WHERE space_id = ?1 AND account_id = ?2",
            params![space, account, text],
        )?,
        _ => 0,
    };
    Ok(())
}

/// The person's own Bindle Cloud details, kept in settings. The recovery
/// key and space keys live in the system keychain, never here.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CloudAccount {
    /// The relay's address, such as `https://cloud.example.com`.
    pub relay: Option<String>,
    /// The name others see in Ours.
    pub name: Option<String>,
    /// Stable across this person's devices: derived from their recovery key.
    pub account: Option<String>,
    /// This device, so its own changes can be told apart.
    pub device: Option<String>,
    /// When the person said they saved the recovery kit.
    pub kit_saved_at: Option<String>,
}

const CLOUD_KEY: &str = "cloud";

pub fn cloud_account(conn: &Connection) -> Result<CloudAccount, StoreError> {
    let saved: Option<String> = conn
        .query_row(
            "SELECT value FROM setting WHERE key = ?1",
            [CLOUD_KEY],
            |r| r.get(0),
        )
        .optional()?;
    Ok(saved
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default())
}

pub fn save_cloud_account(conn: &Connection, account: &CloudAccount) -> Result<(), StoreError> {
    let json = serde_json::to_string(account).expect("account serialises");
    conn.execute(
        "INSERT INTO setting (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [CLOUD_KEY, json.as_str()],
    )?;
    Ok(())
}

#[cfg(test)]
#[path = "cloud_tests.rs"]
mod tests;
