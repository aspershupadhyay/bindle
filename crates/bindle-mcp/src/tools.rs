// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The tools an AI can call. Four read; one suggests. None changes, finishes
//! or removes a promise (handbook P8).

use crate::text;
use bindle_core::{week_of, State};
use bindle_store::{ai, home, people, promises, search, StoreError};
use chrono::{Local, NaiveDate, TimeZone, Utc};
use rusqlite::Connection;
use serde_json::{json, Value};

/// The most promises one call returns.
const MAX_LIMIT: usize = 100;

/// Every tool, as `tools/list` describes them.
pub fn list() -> Value {
    let read_only = json!({ "readOnlyHint": true, "openWorldHint": false });
    json!([
        {
            "name": "read_home",
            "title": "Read Home",
            "description": "Today in Bindle: the top 3, promises running late, what others owe the person, what is coming up this week, and how full each day is. Start here.",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": read_only,
        },
        {
            "name": "list_promises",
            "title": "List promises",
            "description": "Promises in one state: open (being kept), later (someday), unsorted (waiting in Sort), done or let_go.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "state": { "type": "string", "enum": ["open", "later", "unsorted", "done", "let_go"] },
                    "limit": { "type": "integer", "minimum": 1, "maximum": MAX_LIMIT, "default": 30 }
                },
                "required": ["state"]
            },
            "annotations": read_only,
        },
        {
            "name": "search_promises",
            "title": "Search promises",
            "description": "Finds promises by words in their title, person or first step, in any state.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "minLength": 1 },
                    "limit": { "type": "integer", "minimum": 1, "maximum": MAX_LIMIT, "default": 20 }
                },
                "required": ["query"]
            },
            "annotations": read_only,
        },
        {
            "name": "read_people",
            "title": "Read people",
            "description": "Everyone the person has promises with. Give a name to see both sides with that one person: what the person owes them, what they owe the person, and what was kept lately.",
            "inputSchema": {
                "type": "object",
                "properties": { "name": { "type": "string" } }
            },
            "annotations": read_only,
        },
        {
            "name": "suggest_promise",
            "title": "Suggest a promise",
            "description": "Suggests a new promise. It waits in Sort until the person keeps it or lets it go; nothing is added to their day without them. Write it the way they would jot it down, with the person, day and time in plain words, for example 'Call Rohan about the rent Friday 6pm' or 'Aman owes me ₹500 by Sunday'. Suggest only what the conversation clearly points to.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string", "minLength": 1, "maxLength": 300 },
                    "reason": { "type": "string", "maxLength": 300, "description": "One short line on why, shown to the person in Sort." }
                },
                "required": ["text"]
            },
            "annotations": { "readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false },
        },
    ])
}

/// Runs one tool. Failures come back as a tool result with `isError`, in
/// plain words the AI can pass on.
pub fn call(conn: &mut Connection, client: &str, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let result = match name {
        "read_home" => read_home(conn),
        "list_promises" => list_promises(conn, &args),
        "search_promises" => search_promises(conn, &args),
        "read_people" => read_people(conn, &args),
        "suggest_promise" => suggest_promise(conn, client, &args),
        _ => Err(Failure::Asked(format!(
            "Bindle has no tool called {name:?}."
        ))),
    };
    let detail = args
        .get("query")
        .or_else(|| args.get("name"))
        .or_else(|| args.get("text"))
        .or_else(|| args.get("state"))
        .and_then(Value::as_str)
        .unwrap_or("");
    // A refused call is logged too: the person should see every attempt.
    let outcome = if result.is_ok() {
        detail.to_string()
    } else {
        format!("refused {detail}").trim().to_string()
    };
    let _ = ai::log(conn, client, name, &outcome, Utc::now());
    match result {
        Ok(text) => json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
        Err(failure) => {
            json!({ "content": [{ "type": "text", "text": failure.to_string() }], "isError": true })
        }
    }
}

/// Why a tool call did not work.
enum Failure {
    Store(StoreError),
    Asked(String),
}

impl From<StoreError> for Failure {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => write!(f, "{e}"),
            Self::Asked(message) => f.write_str(message),
        }
    }
}

type Reply = Result<String, Failure>;

fn today() -> NaiveDate {
    Local::now().date_naive()
}

fn read_home(conn: &Connection) -> Reply {
    ai::check_read(conn)?;
    let monday = week_of(today())[0]
        .and_hms_opt(0, 0, 0)
        .expect("midnight exists");
    let week_start = Local
        .from_local_datetime(&monday)
        .earliest()
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    let mut view = home::home(conn, today(), week_start)?;
    let hidden = ai::hidden_ids(conn)?;
    for list in [
        &mut view.top3,
        &mut view.suggestions,
        &mut view.running_late,
        &mut view.coming_up,
    ] {
        ai::without_hidden(list, &hidden);
    }
    view.waiting_on_others
        .retain(|w| !hidden.contains(&w.promise.id));
    let name = bindle_store::settings::load(conn)?.name;
    Ok(text::home(&view, today(), name.as_deref()))
}

fn limit(args: &Value, default: usize) -> usize {
    args.get("limit")
        .and_then(Value::as_u64)
        .map_or(default, |n| (n as usize).clamp(1, MAX_LIMIT))
}

fn list_promises(conn: &Connection, args: &Value) -> Reply {
    ai::check_read(conn)?;
    let word = args.get("state").and_then(Value::as_str).unwrap_or("open");
    let state = State::from_word(word).ok_or_else(|| {
        Failure::Asked(format!(
            "{word:?} is not a state. Use open, later, unsorted, done or let_go."
        ))
    })?;
    let hidden = ai::hidden_ids(conn)?;
    let wanted = limit(args, 30);
    let mut list = promises::list_for_screen(conn, state, wanted + hidden.len())?;
    ai::without_hidden(&mut list, &hidden);
    list.truncate(wanted);
    Ok(text::list(
        &format!("{} promises", text::state_word(state)),
        &list,
        today(),
    ))
}

fn search_promises(conn: &Connection, args: &Value) -> Reply {
    ai::check_read(conn)?;
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if query.is_empty() {
        return Err(Failure::Asked("Give a few words to search for.".into()));
    }
    let hidden = ai::hidden_ids(conn)?;
    let wanted = limit(args, 20);
    let mut list = search::search(conn, query, wanted + hidden.len())?;
    ai::without_hidden(&mut list, &hidden);
    list.truncate(wanted);
    Ok(text::list(
        &format!("Promises matching {query:?}"),
        &list,
        today(),
    ))
}

fn read_people(conn: &Connection, args: &Value) -> Reply {
    ai::check_read(conn)?;
    let hidden = ai::hidden_people(conn)?;
    let everyone: Vec<_> = people::summaries(conn)?
        .into_iter()
        .filter(|p| !hidden.contains(&p.id))
        .collect();
    let Some(name) = args
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|n| !n.is_empty())
    else {
        return Ok(text::people(&everyone));
    };
    let Some(person) = everyone.iter().find(|p| p.name.eq_ignore_ascii_case(name)) else {
        return Err(Failure::Asked(format!(
            "There is no one called {name:?} in Bindle."
        )));
    };
    let since = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let mut detail = people::detail(conn, &person.id, &since)?;
    let hidden = ai::hidden_ids(conn)?;
    for list in [
        &mut detail.you_owe,
        &mut detail.they_owe,
        &mut detail.kept_lately,
    ] {
        ai::without_hidden(list, &hidden);
    }
    Ok(text::person(&detail, today()))
}

fn suggest_promise(conn: &mut Connection, client: &str, args: &Value) -> Reply {
    ai::check_suggest(conn)?;
    let typed: String = args
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .chars()
        .take(300)
        .collect();
    let reason: Option<String> = args
        .get("reason")
        .and_then(Value::as_str)
        .map(|r| r.chars().take(300).collect());
    let known = people::names(conn)?;
    let reading = bindle_parse::read(&typed, Local::now().naive_local(), &known);
    let jot = promises::Jot {
        unsure_words: reading.unsure_words,
        text: typed.clone(),
        title: reading.title,
        when: reading.when,
        is_deadline: reading.is_deadline,
        amount: reading.amount,
        person: reading.person,
        direction: reading.direction,
        strength: reading.strength,
        tz: iana_time_zone::get_timezone().ok(),
    };
    let promise = ai::suggest(conn, &jot, client, reason.as_deref(), Utc::now())?;
    Ok(format!(
        "Suggested. \"{}\" is waiting in Sort{}. The person decides whether to keep it; tell them it is there.",
        promise.title,
        text::when_phrase(&promise, today()).map(|w| format!(" ({w})")).unwrap_or_default(),
    ))
}
