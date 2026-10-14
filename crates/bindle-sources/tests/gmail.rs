// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The Gmail client against a tiny local IMAP server: it signs in, opens
//! both folders read-only, reads emails without marking them, and turns a
//! refused sign-in into plain words.

use bindle_sources::gmail::{Gmail, MailError};
use chrono::{TimeZone, Utc};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

const INBOX_MAIL: &str = "From: Priya Sharma <priya@brand.example>\r\nTo: sparsh@gmail.com\r\nSubject: Sponsor brief\r\nDate: Mon, 12 Oct 2026 10:12:00 +0530\r\nMessage-ID: <brief-1@brand.example>\r\nContent-Type: text/plain\r\n\r\n";
const INBOX_TEXT: &str = "Could you share your rate card?\r\n";
const SENT_MAIL: &str = "From: Sparsh <sparsh@gmail.com>\r\nTo: Neha <neha@gymbros.example>\r\nSubject: Timings\r\nDate: Mon, 12 Oct 2026 18:00:00 +0530\r\nMessage-ID: <ask-1@gmail.com>\r\nContent-Type: text/plain\r\n\r\n";
const SENT_TEXT: &str = "I'll send the build by Friday.\r\n";

fn fetch_line(uid: u32, header: &str, text: &str) -> String {
    format!(
        "* {uid} FETCH (UID {uid} BODY[HEADER] {{{}}}\r\n{header} BODY[TEXT]<0> {{{}}}\r\n{text})\r\n",
        header.len(),
        text.len()
    )
}

/// Serves one IMAP session and records every command it got.
async fn serve(password_ok: bool) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let (read, mut write) = socket.into_split();
        let mut lines = BufReader::new(read).lines();
        write.write_all(b"* OK Gimap ready\r\n").await.unwrap();
        let mut folder = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            log.lock().unwrap().push(line.clone());
            let (tag, command) = line.split_once(' ').unwrap();
            let upper = command.to_uppercase();
            let reply = if upper.starts_with("LOGIN") {
                if password_ok {
                    format!("{tag} OK sparsh@gmail.com authenticated (Success)\r\n")
                } else {
                    format!("{tag} NO [AUTHENTICATIONFAILED] Invalid credentials (Failure)\r\n")
                }
            } else if upper.starts_with("LIST") {
                format!("* LIST (\\HasNoChildren) \"/\" \"INBOX\"\r\n* LIST (\\HasNoChildren \\Sent) \"/\" \"[Gmail]/Sent Mail\"\r\n{tag} OK Success\r\n")
            } else if upper.starts_with("EXAMINE") {
                folder = command.to_string();
                format!("* FLAGS (\\Seen)\r\n* 1 EXISTS\r\n{tag} OK [READ-ONLY] Success\r\n")
            } else if upper.starts_with("UID SEARCH") {
                format!("* SEARCH 1\r\n{tag} OK SEARCH completed\r\n")
            } else if upper.starts_with("UID FETCH") {
                let body = if folder.contains("Sent") {
                    fetch_line(1, SENT_MAIL, SENT_TEXT)
                } else {
                    fetch_line(1, INBOX_MAIL, INBOX_TEXT)
                };
                format!("{body}{tag} OK Success\r\n")
            } else if upper.starts_with("LOGOUT") {
                format!("* BYE LOGOUT Requested\r\n{tag} OK 73 good day\r\n")
            } else {
                format!("{tag} BAD unknown\r\n")
            };
            write.write_all(reply.as_bytes()).await.unwrap();
            if upper.starts_with("LOGOUT") || !password_ok {
                break;
            }
        }
    });
    (port, seen)
}

#[tokio::test]
async fn reads_both_folders_read_only() {
    let (port, seen) = serve(true).await;
    let gmail = Gmail::on_test_server("sparsh@gmail.com", "abcd efgh ijkl mnop", port);
    let now = Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap();
    let recent = gmail.recent(now).await.unwrap();

    assert_eq!(recent.inbox.len(), 1);
    assert_eq!(recent.inbox[0].from_name, "Priya Sharma");
    assert!(recent.inbox[0].text.contains("rate card"));
    assert_eq!(recent.sent.len(), 1);
    assert_eq!(recent.sent[0].to_name, "Neha");

    let seen = seen.lock().unwrap().join("\n");
    assert!(
        seen.contains("LOGIN \"sparsh@gmail.com\" \"abcdefghijklmnop\""),
        "{seen}"
    );
    assert!(seen.contains("EXAMINE \"INBOX\""));
    assert!(seen.contains("EXAMINE \"[Gmail]/Sent Mail\""));
    assert!(seen.contains("UID SEARCH SINCE 30-Sep-2026"));
    assert!(seen.contains("BODY.PEEK[HEADER]"));
    // Nothing that could change the mailbox is ever sent.
    for word in [
        "SELECT", "STORE", "COPY", "MOVE", "EXPUNGE", "APPEND", "DELETE",
    ] {
        assert!(!seen.contains(word), "sent {word}: {seen}");
    }
}

#[tokio::test]
async fn a_refused_sign_in_says_so() {
    let (port, _) = serve(false).await;
    let gmail = Gmail::on_test_server("sparsh@gmail.com", "wrong", port);
    assert_eq!(
        gmail.check().await,
        Err(MailError::SignIn("Invalid credentials".into()))
    );
}

#[tokio::test]
async fn no_server_means_offline() {
    let gmail = Gmail::on_test_server("sparsh@gmail.com", "x", 9);
    assert_eq!(gmail.check().await, Err(MailError::Offline));
}
