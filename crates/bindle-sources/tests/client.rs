// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The GitHub client against a tiny local server: it sends the token, asks
//! the three searches, and turns GitHub's errors into plain words.

use bindle_sources::github::{Client, GithubError};
use bindle_store::work::WorkKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const REVIEWS: &str = include_str!("fixtures/search_reviews.json");

/// Answers each request with `answer(request_line, headers)` until stopped.
async fn serve(
    answer: fn(&str) -> (u16, String),
) -> (String, tokio::task::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        let mut seen = Vec::new();
        while let Ok(Ok((mut socket, _))) =
            tokio::time::timeout(std::time::Duration::from_millis(500), listener.accept()).await
        {
            let mut buf = vec![0; 8192];
            let n = socket.read(&mut buf).await.unwrap();
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let (status, body) = answer(&request);
            seen.push(request);
            let reply = format!(
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(reply.as_bytes()).await.unwrap();
        }
        seen
    });
    (base, handle)
}

#[tokio::test]
async fn the_client_signs_in_and_reads_all_three_searches() {
    let (base, server) = serve(|request| {
        if request.starts_with("GET /user ") {
            (200, r#"{"login":"aspershupadhyay"}"#.into())
        } else if request.contains("review-requested") {
            (200, REVIEWS.into())
        } else {
            (200, r#"{"items":[]}"#.into())
        }
    })
    .await;
    let client = Client::with_base(" secret-token \n", &base).unwrap();
    assert_eq!(client.login().await.unwrap(), "aspershupadhyay");
    let work = client.work().await.unwrap();
    assert_eq!(work.len(), 2);
    assert!(work.iter().all(|i| i.kind == WorkKind::Review));

    let seen = server.await.unwrap();
    assert_eq!(seen.len(), 4, "one sign-in and three searches");
    for request in &seen {
        let lower = request.to_lowercase();
        assert!(
            lower.contains("authorization: bearer secret-token\r\n"),
            "{request}"
        );
        assert!(lower.contains("user-agent: bindle/"));
    }
}

#[tokio::test]
async fn a_refusal_passes_on_what_github_said() {
    let (base, server) = serve(|_| {
        (
            403,
            r#"{"message":"Resource protected by organization SAML enforcement."}"#.into(),
        )
    })
    .await;
    let client = Client::with_base("t", &base).unwrap();
    assert_eq!(
        client.login().await,
        Err(GithubError::Refused(
            "Resource protected by organization SAML enforcement.".into()
        ))
    );
    server.abort();
}

#[tokio::test]
async fn a_rejected_token_says_so() {
    let (base, server) = serve(|_| (401, r#"{"message":"Bad credentials"}"#.into())).await;
    let client = Client::with_base("old", &base).unwrap();
    assert_eq!(client.login().await, Err(GithubError::BadToken));
    server.abort();
}

#[tokio::test]
async fn no_server_means_offline() {
    let client = Client::with_base("t", "http://127.0.0.1:9").unwrap();
    assert_eq!(client.work().await, Err(GithubError::Offline));
}
