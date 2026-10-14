// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Bindle's AI connection (handbook P8): a Model Context Protocol server that
//! speaks JSON-RPC over stdin and stdout, one message per line.
//!
//! An AI app such as Claude starts it with `bindle --mcp --db <path>`. It
//! opens the same database as the app; SQLite's WAL lets both work at once.
//! What the AI may do is decided by the person in Settings, and checked on
//! every call (see `bindle_store::ai`), so a change there applies at once.

mod text;
mod tools;

use rusqlite::Connection;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;

/// The protocol versions this server speaks, newest first.
const PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

/// What the AI is told about Bindle when it connects.
const INSTRUCTIONS: &str =
    "Bindle keeps the person's promises: what they owe others, what others owe them, \
and what they promised themselves. You can read Home, promises and people. You can suggest a new \
promise, which waits in Sort for the person to keep or let go; you can never change, finish or \
remove one, and Bindle never sends messages for anyone. Use Bindle's words: a promise is \
'running late', never 'overdue'. Be kind about late promises.";

/// One connection to one AI app.
pub struct Server {
    conn: Connection,
    /// The AI app's name, from `initialize`: "Claude", for example.
    client: String,
}

impl Server {
    /// A server reading and writing through `conn`.
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            client: "AI".into(),
        }
    }

    /// Answers one JSON-RPC message. Notifications get no answer.
    pub fn handle(&mut self, message: &Value) -> Option<Value> {
        let id = message.get("id").cloned()?;
        let method = message.get("method").and_then(Value::as_str).unwrap_or("");
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        let result = match method {
            "initialize" => Ok(self.initialize(&params)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tools::list() })),
            "tools/call" => Ok(tools::call(&mut self.conn, &self.client, &params)),
            _ => Err((-32601, format!("Bindle does not know {method:?}"))),
        };
        Some(match result {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, message)) => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
            }
        })
    }

    fn initialize(&mut self, params: &Value) -> Value {
        if let Some(name) = params.pointer("/clientInfo/name").and_then(Value::as_str) {
            self.client = client_name(name);
        }
        let asked = params.get("protocolVersion").and_then(Value::as_str);
        let version = PROTOCOL_VERSIONS
            .iter()
            .find(|v| Some(**v) == asked)
            .unwrap_or(&PROTOCOL_VERSIONS[0]);
        json!({
            "protocolVersion": version,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "bindle", "title": "Bindle", "version": env!("CARGO_PKG_VERSION") },
            "instructions": INSTRUCTIONS,
        })
    }
}

/// A short, readable name for the AI app: "claude-ai" becomes "Claude".
fn client_name(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("claude") {
        return "Claude".into();
    }
    let trimmed: String = raw.trim().chars().take(40).collect();
    if trimmed.is_empty() {
        "AI".into()
    } else {
        trimmed
    }
}

/// Reads messages from `input` until it closes, answering each on `output`.
pub fn serve(
    server: &mut Server,
    input: impl BufRead,
    mut output: impl Write,
) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(Value::Array(batch)) => {
                let replies: Vec<Value> = batch.iter().filter_map(|m| server.handle(m)).collect();
                (!replies.is_empty()).then_some(Value::Array(replies))
            }
            Ok(message) => server.handle(&message),
            Err(_) => Some(json!({
                "jsonrpc": "2.0", "id": null,
                "error": { "code": -32700, "message": "that was not JSON" }
            })),
        };
        if let Some(reply) = reply {
            writeln!(output, "{reply}")?;
            output.flush()?;
        }
    }
    Ok(())
}

/// Runs the server from command-line arguments: `--db <path>`. Used by the
/// `bindle-mcp` binary and by the app when started with `--mcp`.
pub fn run_from_args(args: impl IntoIterator<Item = String>) -> ExitCode {
    let mut args = args.into_iter();
    let mut db: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        if arg == "--db" {
            db = args.next().map(PathBuf::from);
        }
    }
    let Some(db) = db else {
        eprintln!("bindle mcp: pass --db <path to bindle.db>. Settings > AI connection shows the line to use.");
        return ExitCode::from(2);
    };
    let conn = match bindle_store::open(&db) {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("bindle mcp: could not open {}: {e}", db.display());
            return ExitCode::FAILURE;
        }
    };
    let mut server = Server::new(conn);
    let stdin = std::io::stdin();
    match serve(&mut server, stdin.lock(), std::io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("bindle mcp: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests;
