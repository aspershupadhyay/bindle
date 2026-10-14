// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! YouTube, read only: a channel's public numbers, its latest uploads, and
//! the comments on them that are still waiting on the channel's reply.
//!
//! Bindle reads the YouTube Data API with an API key the person makes in
//! Google Cloud (the key only reads public data and lives in the system
//! keychain). Watch time, click-through and audience need YouTube
//! Analytics, which needs "Sign in with Google" and an app the owner
//! registers; they are not read yet. Nothing is ever posted or replied.

use bindle_store::creator::{Channel, FoundComment, FoundPost};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use thiserror::Error;

/// Where the YouTube Data API lives. Tests point the client somewhere else.
pub const API: &str = "https://www.googleapis.com/youtube/v3";

/// Where a person makes an API key (then turns on "YouTube Data API v3").
pub const KEY_PAGE: &str = "https://console.cloud.google.com/apis/credentials";

/// At most this many uploads are read per look.
const UPLOADS: usize = 50;
/// Comments older than this no longer count as waiting.
const COMMENT_DAYS: i64 = 14;
/// Videos this short or shorter are counted as Shorts (YouTube allows up to 3 minutes).
const SHORT_SECONDS: u32 = 180;

/// Everything that can go wrong talking to YouTube, in plain words.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum YoutubeError {
    #[error("YouTube did not accept the API key. Check it, and that YouTube Data API v3 is turned on for it.")]
    BadKey,
    #[error(
        "no YouTube channel by that name. Type its @handle, or the channel id that starts with UC."
    )]
    NoChannel,
    #[error("YouTube's daily allowance for this key is used up. Bindle will look again tomorrow.")]
    QuotaUsed,
    #[error("YouTube said no: {0}")]
    Refused(String),
    #[error("could not reach YouTube. Check the internet connection.")]
    Offline,
    #[error("YouTube answered with an error ({0}). Bindle will try again later.")]
    Server(u16),
    #[error("YouTube's answer was not what Bindle expected: {0}")]
    Unexpected(String),
}

fn unexpected(error: serde_json::Error) -> YoutubeError {
    YoutubeError::Unexpected(error.to_string())
}

/// What one look found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Look {
    pub channel: Channel,
    pub posts: Vec<FoundPost>,
    pub comments: Vec<FoundComment>,
}

#[derive(Deserialize)]
struct Page<T> {
    #[serde(default = "Vec::new")]
    items: Vec<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChannelItem {
    id: String,
    snippet: ChannelSnippet,
    statistics: ChannelStats,
    content_details: ChannelDetails,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChannelSnippet {
    title: String,
    #[serde(default)]
    description: String,
    custom_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChannelStats {
    #[serde(default)]
    view_count: Option<String>,
    #[serde(default)]
    subscriber_count: Option<String>,
    #[serde(default)]
    hidden_subscriber_count: bool,
    #[serde(default)]
    video_count: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChannelDetails {
    related_playlists: Playlists,
}

#[derive(Deserialize)]
struct Playlists {
    uploads: String,
}

fn count(text: &Option<String>) -> u64 {
    text.as_deref().and_then(|t| t.parse().ok()).unwrap_or(0)
}

/// Reads the channel, and its uploads playlist, from a `channels` answer.
pub fn parse_channel(body: &str) -> Result<(Channel, String), YoutubeError> {
    let page: Page<ChannelItem> = serde_json::from_str(body).map_err(unexpected)?;
    let item = page
        .items
        .into_iter()
        .next()
        .ok_or(YoutubeError::NoChannel)?;
    let stats = &item.statistics;
    Ok((
        Channel {
            id: item.id,
            title: item.snippet.title.trim().to_string(),
            handle: item.snippet.custom_url.map(|h| {
                if h.starts_with('@') {
                    h
                } else {
                    format!("@{h}")
                }
            }),
            description: first_line(&item.snippet.description),
            followers: (!stats.hidden_subscriber_count).then(|| count(&stats.subscriber_count)),
            views: count(&stats.view_count),
            posts: count(&stats.video_count),
        },
        item.content_details.related_playlists.uploads,
    ))
}

fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or_default()
        .chars()
        .take(200)
        .collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistItem {
    content_details: PlaylistDetails,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaylistDetails {
    video_id: String,
}

/// The video ids in a `playlistItems` answer, newest first as listed.
pub fn parse_uploads(body: &str) -> Result<Vec<String>, YoutubeError> {
    let page: Page<PlaylistItem> = serde_json::from_str(body).map_err(unexpected)?;
    Ok(page
        .items
        .into_iter()
        .map(|i| i.content_details.video_id)
        .collect())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoItem {
    id: String,
    snippet: VideoSnippet,
    #[serde(default)]
    statistics: VideoStats,
    content_details: VideoDetails,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoSnippet {
    title: String,
    published_at: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct VideoStats {
    #[serde(default)]
    view_count: Option<String>,
    #[serde(default)]
    like_count: Option<String>,
    #[serde(default)]
    comment_count: Option<String>,
}

#[derive(Deserialize)]
struct VideoDetails {
    duration: String,
}

/// Reads posts from a `videos` answer.
pub fn parse_videos(body: &str) -> Result<Vec<FoundPost>, YoutubeError> {
    let page: Page<VideoItem> = serde_json::from_str(body).map_err(unexpected)?;
    Ok(page
        .items
        .into_iter()
        .map(|v| {
            let seconds = duration_seconds(&v.content_details.duration);
            FoundPost {
                url: format!("https://www.youtube.com/watch?v={}", v.id),
                id: format!("youtube:{}", v.id),
                title: v.snippet.title.trim().to_string(),
                short: seconds > 0 && seconds <= SHORT_SECONDS,
                duration_seconds: seconds,
                published_at: v.snippet.published_at,
                views: count(&v.statistics.view_count),
                likes: count(&v.statistics.like_count),
                comments: count(&v.statistics.comment_count),
            }
        })
        .collect())
}

/// Seconds in an ISO 8601 duration such as "PT18M4S" or "P1DT2H". Zero when
/// it cannot be read (a live stream says "P0D").
pub fn duration_seconds(text: &str) -> u32 {
    let Some(rest) = text.strip_prefix('P') else {
        return 0;
    };
    let (mut total, mut number, mut in_time) = (0u32, String::new(), false);
    for c in rest.chars() {
        match c {
            'T' => in_time = true,
            '0'..='9' => number.push(c),
            unit => {
                let n: u32 = number.parse().unwrap_or(0);
                number.clear();
                total += n * match (unit, in_time) {
                    ('D', false) => 86_400,
                    ('W', false) => 604_800,
                    ('H', true) => 3_600,
                    ('M', true) => 60,
                    ('S', true) => 1,
                    _ => 0,
                };
            }
        }
    }
    total
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadItem {
    id: String,
    snippet: ThreadSnippet,
    replies: Option<Replies>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadSnippet {
    video_id: Option<String>,
    top_level_comment: CommentItem,
    #[serde(default)]
    total_reply_count: u32,
}

#[derive(Deserialize)]
struct Replies {
    #[serde(default)]
    comments: Vec<CommentItem>,
}

#[derive(Deserialize)]
struct CommentItem {
    snippet: CommentSnippet,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommentSnippet {
    author_display_name: String,
    author_channel_id: Option<AuthorId>,
    #[serde(default)]
    text_original: String,
    #[serde(default)]
    like_count: u64,
    published_at: String,
}

#[derive(Deserialize)]
struct AuthorId {
    value: String,
}

fn by(snippet: &CommentSnippet, channel_id: &str) -> bool {
    snippet
        .author_channel_id
        .as_ref()
        .is_some_and(|a| a.value == channel_id)
}

/// Comments on the channel's videos that the channel has not answered,
/// from the last two weeks, from a `commentThreads` answer. `titles` names
/// the videos read this look; comments on other videos are left out.
pub fn parse_waiting_comments(
    body: &str,
    channel_id: &str,
    titles: &[(String, String)],
    now: DateTime<Utc>,
) -> Result<Vec<FoundComment>, YoutubeError> {
    let page: Page<ThreadItem> = serde_json::from_str(body).map_err(unexpected)?;
    let since = now - Duration::days(COMMENT_DAYS);
    Ok(page
        .items
        .into_iter()
        .filter_map(|thread| {
            let top = &thread.snippet.top_level_comment.snippet;
            let video = thread.snippet.video_id.as_deref()?;
            let title = titles.iter().find(|(id, _)| id == video)?.1.clone();
            let at = DateTime::parse_from_rfc3339(&top.published_at).ok()?;
            let answered = thread
                .replies
                .as_ref()
                .is_some_and(|r| r.comments.iter().any(|c| by(&c.snippet, channel_id)));
            // Replies beyond the first few are not listed; a long thread is
            // someone else's conversation by then.
            let busy = thread.snippet.total_reply_count > 5;
            (!by(top, channel_id) && !answered && !busy && at.with_timezone(&Utc) >= since).then(
                || FoundComment {
                    id: format!("youtube:{}", thread.id),
                    post_id: format!("youtube:{video}"),
                    post_title: title,
                    author: top
                        .author_display_name
                        .trim()
                        .trim_start_matches('@')
                        .to_string(),
                    text: top.text_original.trim().chars().take(500).collect(),
                    url: format!("https://www.youtube.com/watch?v={video}&lc={}", thread.id),
                    likes: top.like_count,
                    published_at: top.published_at.clone(),
                },
            )
        })
        .collect())
}

#[derive(Deserialize)]
struct ErrorBody {
    error: ErrorInner,
}

#[derive(Deserialize)]
struct ErrorInner {
    #[serde(default)]
    message: String,
    #[serde(default)]
    errors: Vec<ErrorReason>,
}

#[derive(Deserialize)]
struct ErrorReason {
    #[serde(default)]
    reason: String,
}

/// Turns an HTTP status and Google's error body into an error, or `None`.
fn status_error(status: u16, body: &str) -> Option<YoutubeError> {
    if (200..300).contains(&status) {
        return None;
    }
    let parsed = serde_json::from_str::<ErrorBody>(body).ok();
    let reason = parsed
        .as_ref()
        .and_then(|b| b.error.errors.first())
        .map(|e| e.reason.as_str())
        .unwrap_or_default();
    Some(match (status, reason) {
        (_, "quotaExceeded" | "dailyLimitExceeded" | "rateLimitExceeded") => {
            YoutubeError::QuotaUsed
        }
        (400, "keyInvalid" | "badRequest") if body.contains("API key") => YoutubeError::BadKey,
        (400, "keyInvalid") | (401, _) => YoutubeError::BadKey,
        (403, "accessNotConfigured" | "ipRefererBlocked") => YoutubeError::BadKey,
        (403 | 400, _) => YoutubeError::Refused(
            parsed
                .map(|b| b.error.message)
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| "the key cannot read this".into()),
        ),
        (404, _) => YoutubeError::NoChannel,
        _ => YoutubeError::Server(status),
    })
}

/// A YouTube Data API reader with one API key.
pub struct Client {
    http: reqwest::Client,
    base: String,
    key: String,
}

impl Client {
    pub fn new(key: &str) -> Result<Self, YoutubeError> {
        Self::with_base(key, API)
    }

    /// A client for an API at `base`, such as a test server.
    pub fn with_base(key: &str, base: &str) -> Result<Self, YoutubeError> {
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }
        let http = reqwest::Client::builder()
            .user_agent(concat!("Bindle/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|e| YoutubeError::Unexpected(e.to_string()))?;
        Ok(Self {
            http,
            base: base.trim_end_matches('/').to_string(),
            key: key.trim().to_string(),
        })
    }

    async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<String, YoutubeError> {
        let response = self
            .http
            .get(format!("{}{path}", self.base))
            .query(query)
            .query(&[("key", self.key.as_str())])
            .send()
            .await
            .map_err(|_| YoutubeError::Offline)?;
        let status = response.status().as_u16();
        let body = response.text().await.map_err(|_| YoutubeError::Offline)?;
        match status_error(status, &body) {
            Some(error) => Err(error),
            None => Ok(body),
        }
    }

    /// The channel for "@handle", a channel id ("UC…") or a channel link.
    pub async fn channel(&self, name: &str) -> Result<(Channel, String), YoutubeError> {
        let (field, value) = channel_query(name).ok_or(YoutubeError::NoChannel)?;
        let body = self
            .get(
                "/channels",
                &[
                    ("part", "snippet,statistics,contentDetails"),
                    (field, &value),
                ],
            )
            .await?;
        parse_channel(&body)
    }

    /// One look: the channel, its latest uploads, and comments waiting.
    pub async fn look(&self, name: &str, now: DateTime<Utc>) -> Result<Look, YoutubeError> {
        let (channel, uploads) = self.channel(name).await?;
        let max = UPLOADS.to_string();
        let ids = parse_uploads(
            &self
                .get(
                    "/playlistItems",
                    &[
                        ("part", "contentDetails"),
                        ("playlistId", &uploads),
                        ("maxResults", &max),
                    ],
                )
                .await
                .or_else(|e| match e {
                    // A channel with no uploads has no uploads playlist yet.
                    YoutubeError::NoChannel => Ok(r#"{"items":[]}"#.into()),
                    other => Err(other),
                })?,
        )?;
        let posts = if ids.is_empty() {
            Vec::new()
        } else {
            parse_videos(
                &self
                    .get(
                        "/videos",
                        &[
                            ("part", "snippet,statistics,contentDetails"),
                            ("id", &ids.join(",")),
                        ],
                    )
                    .await?,
            )?
        };
        let titles: Vec<(String, String)> = posts
            .iter()
            .map(|p| {
                (
                    p.id.trim_start_matches("youtube:").to_string(),
                    p.title.clone(),
                )
            })
            .collect();
        let comments = match self
            .get(
                "/commentThreads",
                &[
                    ("part", "snippet,replies"),
                    ("allThreadsRelatedToChannelId", &channel.id),
                    ("order", "time"),
                    ("maxResults", "100"),
                ],
            )
            .await
        {
            Ok(body) => parse_waiting_comments(&body, &channel.id, &titles, now)?,
            // Comments turned off on the channel: nothing is waiting.
            Err(YoutubeError::Refused(_)) => Vec::new(),
            Err(other) => return Err(other),
        };
        Ok(Look {
            channel,
            posts,
            comments,
        })
    }
}

/// How the API is asked for a channel: by handle or by id.
pub fn channel_query(name: &str) -> Option<(&'static str, String)> {
    let name = name.trim().trim_end_matches('/');
    let name = name
        .rsplit_once("youtube.com/")
        .map_or(name, |(_, rest)| rest)
        .trim_start_matches("channel/");
    let name = name.split(['?', '/']).next().unwrap_or_default();
    if name.is_empty() {
        None
    } else if name.starts_with("UC") && name.len() == 24 && !name.contains('@') {
        Some(("id", name.to_string()))
    } else {
        let handle = name.trim_start_matches('@');
        let ok = !handle.is_empty()
            && handle
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_'));
        ok.then(|| ("forHandle", format!("@{handle}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_read_as_seconds() {
        assert_eq!(duration_seconds("PT18M4S"), 1084);
        assert_eq!(duration_seconds("PT58S"), 58);
        assert_eq!(duration_seconds("PT1H2M"), 3720);
        assert_eq!(duration_seconds("P1DT1S"), 86_401);
        assert_eq!(duration_seconds("P0D"), 0);
        assert_eq!(duration_seconds("nonsense"), 0);
    }

    #[test]
    fn channels_are_asked_for_by_handle_or_id() {
        assert_eq!(
            channel_query("@aspersh"),
            Some(("forHandle", "@aspersh".into()))
        );
        assert_eq!(
            channel_query("aspersh"),
            Some(("forHandle", "@aspersh".into()))
        );
        assert_eq!(
            channel_query("https://www.youtube.com/@aspersh/videos"),
            Some(("forHandle", "@aspersh".into()))
        );
        assert_eq!(
            channel_query("https://youtube.com/channel/UCabcdefghijklmnopqrstuv"),
            Some(("id", "UCabcdefghijklmnopqrstuv".into()))
        );
        assert_eq!(channel_query("  "), None);
        assert_eq!(channel_query("bad name!"), None);
    }

    #[test]
    fn google_errors_become_plain_words() {
        let quota =
            r#"{"error":{"code":403,"message":"quota","errors":[{"reason":"quotaExceeded"}]}}"#;
        assert_eq!(status_error(403, quota), Some(YoutubeError::QuotaUsed));
        let key = r#"{"error":{"code":400,"message":"API key not valid. Please pass a valid API key.","errors":[{"reason":"badRequest"}]}}"#;
        assert_eq!(status_error(400, key), Some(YoutubeError::BadKey));
        let off = r#"{"error":{"code":403,"message":"YouTube Data API v3 has not been used","errors":[{"reason":"accessNotConfigured"}]}}"#;
        assert_eq!(status_error(403, off), Some(YoutubeError::BadKey));
        let disabled = r#"{"error":{"code":403,"message":"The video has disabled comments.","errors":[{"reason":"commentsDisabled"}]}}"#;
        assert_eq!(
            status_error(403, disabled),
            Some(YoutubeError::Refused(
                "The video has disabled comments.".into()
            ))
        );
        assert_eq!(status_error(500, ""), Some(YoutubeError::Server(500)));
        assert_eq!(status_error(200, ""), None);
    }
}
