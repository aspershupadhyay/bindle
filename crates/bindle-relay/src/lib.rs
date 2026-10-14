// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Bindle Cloud's relay (handbook P15).
//!
//! The relay is deliberately simple and deliberately blind. It stores sealed
//! batches it cannot open, numbered in the order they arrived, and hands
//! them back to devices that prove they hold the space's key. It knows no
//! names, emails or passwords: a space is a random id, and the first device
//! to write to it sets its write token (only a hash of which is kept).
//!
//! - `POST /v1/spaces/{id}/batches` adds a sealed batch; `GET` reads those after a number.
//! - `PUT /v1/invites/{id}` keeps a sealed invite for 7 days; `POST /v1/invites/{id}/take`
//!   hands it out once and forgets it.
//! - `PUT` and `GET /v1/kits/{id}` keep one sealed recovery backup per kit.

use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// The largest batch, invite or backup the relay accepts.
pub const MAX_BODY: usize = 256 * 1024;
/// The most batches one space may hold.
pub const MAX_BATCHES: i64 = 200_000;
/// The most batches one read returns.
pub const PAGE: i64 = 500;
/// How long an invite waits to be taken.
pub const INVITE_SECONDS: i64 = 7 * 24 * 60 * 60;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS space (id TEXT PRIMARY KEY, token_hash TEXT NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS batch (space_id TEXT NOT NULL, seq INTEGER NOT NULL, sealed TEXT NOT NULL, at INTEGER NOT NULL,
                                  PRIMARY KEY (space_id, seq));
CREATE TABLE IF NOT EXISTS invite (id TEXT PRIMARY KEY, sealed TEXT NOT NULL, expires_at INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS kit (id TEXT PRIMARY KEY, token_hash TEXT NOT NULL, sealed TEXT NOT NULL, at INTEGER NOT NULL);
";

type Db = Arc<Mutex<Connection>>;

/// The relay's routes over an open database.
pub fn router(conn: Connection) -> Router {
    conn.pragma_update(None, "journal_mode", "WAL").ok();
    conn.execute_batch(SCHEMA)
        .expect("the relay schema applies");
    let db: Db = Arc::new(Mutex::new(conn));
    Router::new()
        .route("/v1/health", get(|| async { "ok" }))
        .route("/v1/spaces/{id}/batches", post(add_batch).get(read_batches))
        .route("/v1/invites/{id}", put(put_invite))
        .route("/v1/invites/{id}/take", post(take_invite))
        .route("/v1/kits/{id}", put(put_kit).get(get_kit))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .with_state(db)
}

/// Serves the relay on `listener` until stopped.
pub async fn serve(listener: tokio::net::TcpListener, conn: Connection) -> std::io::Result<()> {
    axum::serve(listener, router(conn)).await
}

type Reply<T> = Result<T, (StatusCode, &'static str)>;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// Ids are 32 lower-case hex characters, as the device derives them.
fn check_id(id: &str) -> Reply<()> {
    if id.len() == 32
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "not a Bindle id"))
    }
}

fn token_hash(headers: &HeaderMap) -> Reply<String> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|t| t.len() >= 32)
        .ok_or((StatusCode::UNAUTHORIZED, "a write token is needed"))?;
    Ok(Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// A sealed value must look like one: an object with a nonce and data.
fn check_sealed(sealed: &Value) -> Reply<String> {
    let ok = sealed.get("nonce").is_some_and(Value::is_string)
        && sealed.get("data").is_some_and(Value::is_string);
    if ok {
        Ok(sealed.to_string())
    } else {
        Err((StatusCode::BAD_REQUEST, "not a sealed value"))
    }
}

fn db_error<E>(_: E) -> (StatusCode, &'static str) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "the relay's storage failed",
    )
}

/// Checks the token against the space's, setting it on first use.
fn authorise(conn: &Connection, table: &str, id: &str, hash: &str, create: bool) -> Reply<bool> {
    let known: Option<String> = conn
        .query_row(
            &format!("SELECT token_hash FROM {table} WHERE id = ?1"),
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(db_error)?;
    match known {
        Some(known) if known == hash => Ok(true),
        Some(_) => Err((StatusCode::FORBIDDEN, "that token does not open this")),
        None if create => Ok(false),
        None => Err((StatusCode::NOT_FOUND, "nothing here yet")),
    }
}

#[derive(Serialize)]
struct Added {
    seq: i64,
}

async fn add_batch(
    State(db): State<Db>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(sealed): Json<Value>,
) -> Reply<Json<Added>> {
    check_id(&id)?;
    let hash = token_hash(&headers)?;
    let sealed = check_sealed(&sealed)?;
    let mut conn = db.lock().map_err(db_error)?;
    let tx = conn.transaction().map_err(db_error)?;
    if !authorise(&tx, "space", &id, &hash, true)? {
        tx.execute(
            "INSERT INTO space (id, token_hash, created_at) VALUES (?1, ?2, ?3)",
            params![id, hash, now()],
        )
        .map_err(db_error)?;
    }
    let last: i64 = tx
        .query_row(
            "SELECT coalesce(max(seq), 0) FROM batch WHERE space_id = ?1",
            [&id],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    if last >= MAX_BATCHES {
        return Err((StatusCode::INSUFFICIENT_STORAGE, "this space is full"));
    }
    tx.execute(
        "INSERT INTO batch (space_id, seq, sealed, at) VALUES (?1, ?2, ?3, ?4)",
        params![id, last + 1, sealed, now()],
    )
    .map_err(db_error)?;
    tx.commit().map_err(db_error)?;
    Ok(Json(Added { seq: last + 1 }))
}

#[derive(Deserialize)]
struct After {
    #[serde(default)]
    after: i64,
}

#[derive(Serialize)]
struct Batch {
    seq: i64,
    sealed: Value,
}

#[derive(Serialize)]
struct Batches {
    batches: Vec<Batch>,
    more: bool,
}

async fn read_batches(
    State(db): State<Db>,
    Path(id): Path<String>,
    Query(after): Query<After>,
    headers: HeaderMap,
) -> Reply<Json<Batches>> {
    check_id(&id)?;
    let hash = token_hash(&headers)?;
    let conn = db.lock().map_err(db_error)?;
    if !authorise(&conn, "space", &id, &hash, true)? {
        // A space nobody wrote to yet is simply empty.
        return Ok(Json(Batches {
            batches: Vec::new(),
            more: false,
        }));
    }
    let mut stmt = conn
        .prepare(
            "SELECT seq, sealed FROM batch WHERE space_id = ?1 AND seq > ?2 ORDER BY seq LIMIT ?3",
        )
        .map_err(db_error)?;
    let mut batches: Vec<Batch> = stmt
        .query_map(params![id, after.after, PAGE + 1], |r| {
            let sealed: String = r.get(1)?;
            Ok(Batch {
                seq: r.get(0)?,
                sealed: serde_json::from_str(&sealed).unwrap_or(Value::Null),
            })
        })
        .map_err(db_error)?
        .collect::<Result<_, _>>()
        .map_err(db_error)?;
    let more = batches.len() as i64 > PAGE;
    batches.truncate(PAGE as usize);
    Ok(Json(Batches { batches, more }))
}

async fn put_invite(
    State(db): State<Db>,
    Path(id): Path<String>,
    Json(sealed): Json<Value>,
) -> Reply<StatusCode> {
    check_id(&id)?;
    let sealed = check_sealed(&sealed)?;
    let conn = db.lock().map_err(db_error)?;
    conn.execute("DELETE FROM invite WHERE expires_at < ?1", [now()])
        .map_err(db_error)?;
    let added = conn
        .execute(
            "INSERT OR IGNORE INTO invite (id, sealed, expires_at) VALUES (?1, ?2, ?3)",
            params![id, sealed, now() + INVITE_SECONDS],
        )
        .map_err(db_error)?;
    if added == 0 {
        return Err((StatusCode::CONFLICT, "that invite already exists"));
    }
    Ok(StatusCode::CREATED)
}

async fn take_invite(State(db): State<Db>, Path(id): Path<String>) -> Reply<Json<Value>> {
    check_id(&id)?;
    let conn = db.lock().map_err(db_error)?;
    let found: Option<(String, i64)> = conn
        .query_row(
            "SELECT sealed, expires_at FROM invite WHERE id = ?1",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(db_error)?;
    conn.execute("DELETE FROM invite WHERE id = ?1", [&id])
        .map_err(db_error)?;
    match found {
        Some((sealed, expires)) if expires >= now() => {
            Ok(Json(serde_json::from_str(&sealed).map_err(db_error)?))
        }
        _ => Err((
            StatusCode::GONE,
            "this invite was used, or it is more than 7 days old",
        )),
    }
}

async fn put_kit(
    State(db): State<Db>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(sealed): Json<Value>,
) -> Reply<StatusCode> {
    check_id(&id)?;
    let hash = token_hash(&headers)?;
    let sealed = check_sealed(&sealed)?;
    let conn = db.lock().map_err(db_error)?;
    if authorise(&conn, "kit", &id, &hash, true)? {
        conn.execute(
            "UPDATE kit SET sealed = ?2, at = ?3 WHERE id = ?1",
            params![id, sealed, now()],
        )
    } else {
        conn.execute(
            "INSERT INTO kit (id, token_hash, sealed, at) VALUES (?1, ?2, ?3, ?4)",
            params![id, hash, sealed, now()],
        )
    }
    .map_err(db_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_kit(
    State(db): State<Db>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Reply<Json<Value>> {
    check_id(&id)?;
    let hash = token_hash(&headers)?;
    let conn = db.lock().map_err(db_error)?;
    authorise(&conn, "kit", &id, &hash, false)?;
    let sealed: String = conn
        .query_row("SELECT sealed FROM kit WHERE id = ?1", [&id], |r| r.get(0))
        .map_err(db_error)?;
    Ok(Json(serde_json::from_str(&sealed).map_err(db_error)?))
}
