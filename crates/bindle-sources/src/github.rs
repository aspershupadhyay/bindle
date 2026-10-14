// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! GitHub: the pull requests waiting on your review, your own pull requests
//! waiting on others, and the issues assigned to you.
//!
//! Bindle signs in with a personal access token the person makes on GitHub.
//! The token is kept in the system keychain by the app, never in the
//! database. Three searches cover everything; each reads at most 100 items.

use bindle_store::work::{FoundItem, WorkKind};
use serde::Deserialize;
use thiserror::Error;

/// Where GitHub's API lives. Tests point the client somewhere else.
pub const API: &str = "https://api.github.com";

/// The page where a person makes a token for Bindle, with the right
/// permissions ticked: read access to repositories, including private ones.
pub const NEW_TOKEN_URL: &str =
    "https://github.com/settings/tokens/new?description=Bindle&scopes=repo";

/// Everything that can go wrong talking to GitHub, in plain words.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GithubError {
    #[error("GitHub did not accept the token. Make a new one and connect again.")]
    BadToken,
    #[error("GitHub said no: {0}")]
    Refused(String),
    #[error("GitHub asked Bindle to slow down. It will look again later.")]
    RateLimited,
    #[error("could not reach GitHub. Check the internet connection.")]
    Offline,
    #[error("GitHub answered with an error ({0}). Bindle will try again later.")]
    Server(u16),
    #[error("GitHub's answer was not what Bindle expected: {0}")]
    Unexpected(String),
}

/// The searches, one per kind of work.
fn query(kind: WorkKind) -> &'static str {
    match kind {
        WorkKind::Review => "is:open is:pr review-requested:@me archived:false",
        WorkKind::Mine => "is:open is:pr author:@me archived:false",
        WorkKind::Issue => "is:open is:issue assignee:@me archived:false",
    }
}

#[derive(Deserialize)]
struct SearchPage {
    items: Vec<SearchItem>,
}

#[derive(Deserialize)]
struct SearchItem {
    node_id: String,
    number: u32,
    title: String,
    html_url: String,
    repository_url: String,
    user: Option<User>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    labels: Vec<Label>,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize)]
struct User {
    login: String,
}

#[derive(Deserialize)]
struct Label {
    name: String,
}

/// Reads one page of GitHub's issue search as items of `kind`.
pub fn parse_search(kind: WorkKind, body: &str) -> Result<Vec<FoundItem>, GithubError> {
    let page: SearchPage =
        serde_json::from_str(body).map_err(|e| GithubError::Unexpected(e.to_string()))?;
    Ok(page
        .items
        .into_iter()
        .map(|item| FoundItem {
            id: format!("github:{}", item.node_id),
            kind,
            repo: repo_from_api_url(&item.repository_url),
            number: item.number,
            title: item.title.trim().to_string(),
            author: item.user.map(|u| u.login).unwrap_or_else(|| "ghost".into()),
            url: item.html_url,
            draft: item.draft,
            labels: item.labels.into_iter().map(|l| l.name).collect(),
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
        .collect())
}

/// Reads the signed-in user's name from GitHub's `/user` answer.
pub fn parse_login(body: &str) -> Result<String, GithubError> {
    serde_json::from_str::<User>(body)
        .map(|u| u.login)
        .map_err(|e| GithubError::Unexpected(e.to_string()))
}

/// "owner/name" from "https://api.github.com/repos/owner/name".
fn repo_from_api_url(url: &str) -> String {
    url.split("/repos/")
        .nth(1)
        .unwrap_or(url)
        .trim_end_matches('/')
        .to_string()
}

/// Turns an HTTP status into an error, or `None` when it is fine. A refusal
/// passes on GitHub's own words (`body`), such as a missing permission.
fn status_error(status: u16, remaining: Option<&str>, body: &str) -> Option<GithubError> {
    match status {
        200..=299 => None,
        401 => Some(GithubError::BadToken),
        429 => Some(GithubError::RateLimited),
        403 if remaining == Some("0") => Some(GithubError::RateLimited),
        403 => Some(GithubError::Refused(
            serde_json::from_str::<Message>(body)
                .map(|m| m.message)
                .unwrap_or_else(|_| "the token cannot read this".into()),
        )),
        _ => Some(GithubError::Server(status)),
    }
}

#[derive(Deserialize)]
struct Message {
    message: String,
}

/// A signed-in connection to GitHub.
pub struct Client {
    http: reqwest::Client,
    base: String,
    token: String,
}

impl Client {
    /// A client for GitHub itself.
    pub fn new(token: &str) -> Result<Self, GithubError> {
        Self::with_base(token, API)
    }

    /// A client for an API at `base`, such as a test server.
    pub fn with_base(token: &str, base: &str) -> Result<Self, GithubError> {
        // reqwest is built without a crypto provider, so pick one once.
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }
        let http = reqwest::Client::builder()
            .user_agent(concat!("Bindle/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|e| GithubError::Unexpected(e.to_string()))?;
        Ok(Self {
            http,
            base: base.trim_end_matches('/').to_string(),
            token: token.trim().to_string(),
        })
    }

    async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<String, GithubError> {
        let response = self
            .http
            .get(format!("{}{path}", self.base))
            .query(query)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|_| GithubError::Offline)?;
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let status = response.status().as_u16();
        let body = response.text().await.map_err(|_| GithubError::Offline)?;
        match status_error(status, remaining.as_deref(), &body) {
            Some(error) => Err(error),
            None => Ok(body),
        }
    }

    /// The signed-in user's name. Also checks the token works.
    pub async fn login(&self) -> Result<String, GithubError> {
        parse_login(&self.get("/user", &[]).await?)
    }

    /// Everything that needs the person: reviews, their pull requests and
    /// their issues.
    pub async fn work(&self) -> Result<Vec<FoundItem>, GithubError> {
        let mut all = Vec::new();
        for kind in [WorkKind::Review, WorkKind::Mine, WorkKind::Issue] {
            let body = self
                .get(
                    "/search/issues",
                    &[
                        ("q", query(kind)),
                        ("sort", "updated"),
                        ("order", "desc"),
                        ("per_page", "100"),
                    ],
                )
                .await?;
            all.extend(parse_search(kind, &body)?);
        }
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REVIEWS: &str = include_str!("../tests/fixtures/search_reviews.json");

    #[test]
    fn a_search_page_becomes_work_items() {
        let items = parse_search(WorkKind::Review, REVIEWS).unwrap();
        assert_eq!(items.len(), 2);
        let first = &items[0];
        assert_eq!(first.id, "github:PR_kwDOLatch42");
        assert_eq!(first.kind, WorkKind::Review);
        assert_eq!(first.repo, "kunal-m/latch");
        assert_eq!(first.number, 42);
        assert_eq!(first.title, "Permission prompts for the share sheet");
        assert_eq!(first.author, "kunal-m");
        assert_eq!(first.url, "https://github.com/kunal-m/latch/pull/42");
        assert!(!first.draft);
        assert_eq!(first.labels, ["android", "ux"]);
        // A deleted account shows as GitHub's "ghost"; a draft stays a draft.
        assert_eq!(items[1].author, "ghost");
        assert!(items[1].draft);
    }

    #[test]
    fn a_broken_answer_is_an_error_not_a_crash() {
        assert!(matches!(
            parse_search(WorkKind::Issue, "<html>"),
            Err(GithubError::Unexpected(_))
        ));
        assert_eq!(parse_login(r#"{"login":"octo"}"#).unwrap(), "octo");
    }

    #[test]
    fn statuses_say_what_went_wrong() {
        assert_eq!(status_error(200, None, ""), None);
        assert_eq!(status_error(401, None, ""), Some(GithubError::BadToken));
        assert_eq!(
            status_error(403, Some("0"), ""),
            Some(GithubError::RateLimited)
        );
        assert_eq!(status_error(429, None, ""), Some(GithubError::RateLimited));
        assert_eq!(
            status_error(403, Some("4999"), r#"{"message":"Must have admin rights"}"#),
            Some(GithubError::Refused("Must have admin rights".into()))
        );
        assert_eq!(
            status_error(403, None, "<html>"),
            Some(GithubError::Refused("the token cannot read this".into()))
        );
        assert_eq!(status_error(502, None, ""), Some(GithubError::Server(502)));
    }

    #[test]
    fn the_searches_only_ask_for_open_work() {
        for kind in [WorkKind::Review, WorkKind::Mine, WorkKind::Issue] {
            assert!(query(kind).starts_with("is:open"));
            assert!(query(kind).contains("@me"));
        }
    }
}
