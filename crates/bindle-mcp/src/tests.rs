// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

use super::*;
use bindle_store::ai::AiAccess;
use bindle_store::promises::{jot_down, sort_queue, Jot};
use bindle_store::{lifecycle, settings};

fn server(access: AiAccess) -> Server {
    let conn = bindle_store::open_in_memory().expect("open");
    let mut s = settings::load(&conn).expect("load");
    s.ai_access = access;
    s.name = Some("Sparsh".into());
    settings::save(&conn, &s).expect("save");
    Server::new(conn)
}

fn call(server: &mut Server, tool: &str, arguments: Value) -> (bool, String) {
    let reply = server
        .handle(&json!({ "jsonrpc": "2.0", "id": 7, "method": "tools/call",
                         "params": { "name": tool, "arguments": arguments } }))
        .expect("a reply");
    let result = &reply["result"];
    (
        result["isError"].as_bool().expect("isError"),
        result["content"][0]["text"]
            .as_str()
            .expect("text")
            .to_string(),
    )
}

/// Runs a whole conversation through `serve`, as an AI app would.
fn conversation(server: &mut Server, lines: &[Value]) -> Vec<Value> {
    let input: String = lines.iter().map(|l| format!("{l}\n")).collect();
    let mut output = Vec::new();
    serve(server, input.as_bytes(), &mut output).expect("serve");
    String::from_utf8(output)
        .expect("utf8")
        .lines()
        .map(|l| serde_json::from_str(l).expect("json"))
        .collect()
}

#[test]
fn an_ai_app_connects_and_sees_five_tools() {
    let mut s = server(AiAccess::Read);
    let replies = conversation(
        &mut s,
        &[
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2025-06-18", "capabilities": {},
                "clientInfo": { "name": "claude-ai", "version": "1" } } }),
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        ],
    );
    assert_eq!(replies.len(), 2, "notifications get no reply");
    assert_eq!(replies[0]["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(replies[0]["result"]["serverInfo"]["name"], "bindle");
    let tools: Vec<&str> = replies[1]["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|t| t["name"].as_str().expect("name"))
        .collect();
    assert_eq!(
        tools,
        [
            "read_home",
            "list_promises",
            "search_promises",
            "read_people",
            "suggest_promise"
        ]
    );
    assert_eq!(s.client, "Claude");
}

#[test]
fn bad_json_and_unknown_methods_get_errors_not_a_crash() {
    let mut s = server(AiAccess::Read);
    let input = "not json\n{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"resources/list\"}\n";
    let mut output = Vec::new();
    serve(&mut s, input.as_bytes(), &mut output).expect("serve");
    let text = String::from_utf8(output).expect("utf8");
    assert!(text.contains("-32700"));
    assert!(text.contains("-32601"));
}

#[test]
fn with_the_connection_off_every_tool_says_so() {
    let mut s = server(AiAccess::Off);
    for tool in [
        "read_home",
        "list_promises",
        "search_promises",
        "read_people",
    ] {
        let (is_error, text) = call(&mut s, tool, json!({ "state": "open", "query": "x" }));
        assert!(is_error, "{tool}");
        assert!(text.contains("Turn it on in Settings"), "{tool}: {text}");
    }
}

#[test]
fn home_reads_in_bindles_words() {
    let mut s = server(AiAccess::Read);
    let id = jot_down(
        &mut s.conn,
        &Jot::plain("Pay electricity bill"),
        chrono::Utc::now(),
    )
    .expect("jot")
    .id;
    let sorting = lifecycle::Sorting {
        title: "Pay electricity bill".into(),
        direction: Some(bindle_core::Direction::SelfPromise),
        strength: bindle_core::Strength::Firm,
        person: None,
        first_step: None,
        when: Some(bindle_core::When {
            day: chrono::Local::now().date_naive() - chrono::Duration::days(2),
            time: None,
        }),
        is_deadline: true,
    };
    lifecycle::sort(
        &mut s.conn,
        &id,
        &sorting,
        bindle_core::State::Open,
        chrono::Utc::now(),
    )
    .expect("sort");
    let (is_error, text) = call(&mut s, "read_home", json!({}));
    assert!(!is_error, "{text}");
    assert!(text.contains("for Sparsh"));
    assert!(text.contains("Running late"));
    assert!(text.contains("- Pay electricity bill (for yourself; was due"));
    assert!(!text.to_lowercase().contains("overdue"));
}

#[test]
fn a_suggestion_lands_in_sort_read_like_a_jot() {
    let mut s = server(AiAccess::Suggest);
    s.client = "Claude".into();
    let (is_error, text) = call(
        &mut s,
        "suggest_promise",
        json!({ "text": "Call Rohan about the rent tomorrow 6pm", "reason": "Rohan asked in chat" }),
    );
    assert!(!is_error, "{text}");
    assert!(text.contains("waiting in Sort"));
    let queue = sort_queue(&s.conn).expect("queue");
    assert_eq!(queue[0].suggested_by.as_deref(), Some("Claude"));
    assert_eq!(queue[0].promise.person.as_deref(), Some("Rohan"));
    assert!(queue[0].promise.do_on.is_some_and(|w| w.time.is_some()));
}

#[test]
fn read_only_access_cannot_suggest() {
    let mut s = server(AiAccess::Read);
    let (is_error, text) = call(&mut s, "suggest_promise", json!({ "text": "Book dentist" }));
    assert!(is_error);
    assert!(text.contains("can read but not suggest"));
    assert!(sort_queue(&s.conn).expect("queue").is_empty());
}

#[test]
fn people_kept_from_ai_are_not_there_for_it() {
    let mut s = server(AiAccess::Read);
    let mut jot = Jot::plain("Surprise gift");
    jot.person = Some("Ananya".into());
    jot_down(&mut s.conn, &jot, chrono::Utc::now()).expect("jot");
    let id = bindle_store::people::find_or_add(&s.conn, "Ananya", "x").expect("find");
    bindle_store::ai::set_person_hidden(&s.conn, &id, true).expect("hide");
    let (_, everyone) = call(&mut s, "read_people", json!({}));
    assert!(!everyone.contains("Ananya"));
    let (is_error, _) = call(&mut s, "read_people", json!({ "name": "Ananya" }));
    assert!(is_error);
    let (_, found) = call(&mut s, "search_promises", json!({ "query": "gift" }));
    assert!(found.contains("none"), "{found}");
}

#[test]
fn every_call_is_logged_even_a_refused_one() {
    let mut s = server(AiAccess::Read);
    s.client = "Claude".into();
    call(&mut s, "search_promises", json!({ "query": "rent" }));
    call(&mut s, "suggest_promise", json!({ "text": "Book dentist" }));
    let log = bindle_store::ai::recent(&s.conn, 10).expect("log");
    let rows: Vec<(&str, &str)> = log
        .iter()
        .map(|e| (e.tool.as_str(), e.detail.as_str()))
        .collect();
    assert_eq!(
        rows,
        [
            ("suggest_promise", "refused Book dentist"),
            ("search_promises", "rent")
        ]
    );
}
