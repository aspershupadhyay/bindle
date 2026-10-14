// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The YouTube client against a tiny local server: it sends the key, reads
//! the channel, its uploads and the comments waiting, and turns YouTube's
//! errors into plain words.

use bindle_sources::youtube::{Client, YoutubeError};
use chrono::{TimeZone, Utc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const CHANNEL: &str = include_str!("fixtures/youtube_channel.json");
const UPLOADS: &str = include_str!("fixtures/youtube_uploads.json");
const VIDEOS: &str = include_str!("fixtures/youtube_videos.json");
const COMMENTS: &str = include_str!("fixtures/youtube_comments.json");

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
            seen.push(request.lines().next().unwrap_or_default().to_string());
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

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

#[tokio::test]
async fn a_look_reads_the_channel_uploads_and_waiting_comments() {
    let (base, server) = serve(|request| {
        let body = if request.starts_with("GET /channels") {
            CHANNEL
        } else if request.starts_with("GET /playlistItems") {
            UPLOADS
        } else if request.starts_with("GET /videos") {
            VIDEOS
        } else {
            COMMENTS
        };
        (200, body.into())
    })
    .await;
    let client = Client::with_base(" my-key \n", &base).unwrap();
    let look = client.look("@aspersh", now()).await.unwrap();

    assert_eq!(look.channel.title, "Aspersh");
    assert_eq!(look.channel.handle.as_deref(), Some("@aspersh"));
    assert_eq!(
        look.channel.description,
        "Long builds and walkthroughs. Code you can follow along with."
    );
    assert_eq!(look.channel.followers, Some(642));
    assert_eq!(look.channel.posts, 41);

    assert_eq!(look.posts.len(), 2);
    assert!(!look.posts[0].short && look.posts[1].short);
    assert_eq!(look.posts[1].comments, 0, "a missing count reads as none");

    // Only Kunal waits: Neha was answered, the channel's own comment, an
    // old one and one on a video not read this look are left out.
    assert_eq!(look.comments.len(), 1);
    let kunal = &look.comments[0];
    assert_eq!(kunal.author, "kunal");
    assert_eq!(kunal.post_title, "I let an AI drive my Android phone");
    assert_eq!(kunal.url, "https://www.youtube.com/watch?v=vid1&lc=t1");

    let seen = server.await.unwrap();
    assert_eq!(seen.len(), 4);
    assert!(
        seen[0].contains("forHandle=%40aspersh") && seen[0].contains("key=my-key"),
        "{seen:?}"
    );
    assert!(seen[2].contains("id=vid1%2Cvid2"), "{seen:?}");
    assert!(seen[3].contains("allThreadsRelatedToChannelId=UCaspershaspershaspersh1"));
    assert!(
        seen.iter().all(|line| line.starts_with("GET ")),
        "only reads: {seen:?}"
    );
}

#[tokio::test]
async fn comments_turned_off_are_not_an_error() {
    let (base, _server) = serve(|request| {
        if request.starts_with("GET /commentThreads") {
            (403, r#"{"error":{"code":403,"message":"The channel has disabled comments.","errors":[{"reason":"commentsDisabled"}]}}"#.into())
        } else if request.starts_with("GET /channels") {
            (200, CHANNEL.into())
        } else if request.starts_with("GET /playlistItems") {
            (200, UPLOADS.into())
        } else {
            (200, VIDEOS.into())
        }
    })
    .await;
    let look = Client::with_base("k", &base)
        .unwrap()
        .look("@aspersh", now())
        .await
        .unwrap();
    assert!(look.comments.is_empty());
    assert_eq!(look.posts.len(), 2);
}

#[tokio::test]
async fn a_wrong_key_or_channel_says_so() {
    let (base, _server) = serve(|request| {
        if request.contains("forHandle=%40nobody") {
            (200, r#"{"kind":"youtube#channelListResponse","pageInfo":{"totalResults":0}}"#.into())
        } else {
            (400, r#"{"error":{"code":400,"message":"API key not valid. Please pass a valid API key.","errors":[{"reason":"badRequest"}]}}"#.into())
        }
    })
    .await;
    let client = Client::with_base("k", &base).unwrap();
    assert_eq!(
        client.channel("@nobody").await.unwrap_err(),
        YoutubeError::NoChannel
    );
    assert_eq!(
        client.channel("@aspersh").await.unwrap_err(),
        YoutubeError::BadKey
    );
}

#[tokio::test]
async fn no_server_means_offline() {
    let client = Client::with_base("k", "http://127.0.0.1:9").unwrap();
    assert_eq!(
        client.channel("@aspersh").await.unwrap_err(),
        YoutubeError::Offline
    );
}
