// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for Work (handbook P14): the GitHub connection, the Code screen
//! and Make it a promise. Bindle only reads from GitHub; it never comments,
//! merges or closes anything.
//!
//! The token lives in the system keychain (see [`crate::secrets`]).

use super::{announce, lock, CommandError, Reply};
use crate::Db;
use bindle_core::Promise;
use bindle_sources::github::{self, Client, GithubError};
use bindle_store::work::{self, CodeView, GithubAccount};
use chrono::{SecondsFormat, Utc};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State as Managed};

/// Sent to every window after a look at GitHub, so the Code screen and the
/// sidebar refresh.
pub const WORK_CHANGED: &str = "work-changed";

/// The first look waits until Bindle has settled; then every 15 minutes.
const FIRST_LOOK: Duration = Duration::from_secs(20);
const LOOK_EVERY: Duration = Duration::from_secs(15 * 60);

/// The GitHub token's name in the system keychain.
const TOKEN: &str = "github-token";

impl From<GithubError> for CommandError {
    fn from(error: GithubError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

/// Everything the Code screen shows.
#[tauri::command]
pub fn code_view(db: Managed<Db>) -> Reply<CodeView> {
    Ok(work::code_view(&*lock(&db)?, Utc::now())?)
}

/// How many reviews you owe, for the sidebar.
#[tauri::command]
pub fn reviews_owed(db: Managed<Db>) -> Reply<u32> {
    Ok(work::reviews_owed(&*lock(&db)?)?)
}

/// The page where a person makes a token for Bindle.
#[tauri::command]
pub fn github_token_page() -> &'static str {
    github::NEW_TOKEN_URL
}

/// Checks the token with GitHub, keeps it in the keychain, and has a first look.
#[tauri::command]
pub async fn github_connect(app: AppHandle, token: String) -> Reply<GithubAccount> {
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(CommandError {
            message: "paste the token from GitHub first".into(),
        });
    }
    let login = Client::new(&token)?.login().await?;
    crate::secrets::set(TOKEN, &token).map_err(|message| CommandError { message })?;
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        work::save_github_account(
            &conn,
            &GithubAccount {
                login: Some(login),
                ..Default::default()
            },
        )?;
    }
    refresh(&app).await
}

/// Forgets the token and what GitHub listed. Promises already made stay.
#[tauri::command]
pub async fn github_disconnect(app: AppHandle) -> Reply<()> {
    crate::secrets::forget(TOKEN).map_err(|message| CommandError { message })?;
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        work::save_github_account(&conn, &GithubAccount::default())?;
        work::forget(&conn, "github")?;
    }
    let _ = app.emit(WORK_CHANGED, ());
    Ok(())
}

/// Looks at GitHub now.
#[tauri::command]
pub async fn github_refresh(app: AppHandle) -> Reply<GithubAccount> {
    refresh(&app).await
}

/// Makes a work promise from an item. It waits in Sort.
#[tauri::command]
pub fn make_promise(app: AppHandle, db: Managed<Db>, item_id: String) -> Reply<Promise> {
    let promise = work::make_promise(&mut *lock(&db)?, &item_id, Utc::now())?;
    announce(&app);
    let _ = app.emit(WORK_CHANGED, ());
    Ok(promise)
}

/// Opens a GitHub page in the browser. Only github.com links are opened.
#[tauri::command]
pub fn open_on_github(app: AppHandle, url: String) -> Reply<()> {
    use tauri_plugin_opener::OpenerExt;
    if !is_github_link(&url) {
        return Err(CommandError {
            message: "Bindle only opens github.com links".into(),
        });
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| CommandError {
            message: format!("could not open the browser: {e}"),
        })
}

fn is_github_link(url: &str) -> bool {
    url.strip_prefix("https://github.com/")
        .is_some_and(|rest| !rest.is_empty() && !rest.contains(['\\', '@', ' ']))
}

/// Looks at GitHub and saves what it lists. A failed look is saved as the
/// account's error, so the Code screen can say what went wrong.
pub(crate) async fn refresh(app: &AppHandle) -> Reply<GithubAccount> {
    let db = app.state::<Db>();
    let Some(token) = crate::secrets::get(TOKEN) else {
        return Ok(work::github_account(&*lock(&db)?)?);
    };
    let found = match Client::new(&token) {
        Ok(client) => client.work().await,
        Err(e) => Err(e),
    };
    let account = {
        let mut conn = lock(&db)?;
        let mut account = work::github_account(&conn)?;
        let now = Utc::now();
        match found {
            Ok(items) => {
                work::sync(&mut conn, "github", &items, now)?;
                account.synced_at = Some(now.to_rfc3339_opts(SecondsFormat::Millis, true));
                account.error = None;
            }
            Err(error) => account.error = Some(error.to_string()),
        }
        work::save_github_account(&conn, &account)?;
        account
    };
    let _ = app.emit(WORK_CHANGED, ());
    Ok(account)
}

/// Looks at GitHub in the background while Bindle runs, when connected.
pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_LOOK).await;
        loop {
            let connected = {
                let db = app.state::<Db>();
                lock(&db)
                    .ok()
                    .and_then(|conn| work::github_account(&conn).ok())
                    .is_some_and(|a| a.login.is_some())
            };
            if connected {
                if let Err(error) = refresh(&app).await {
                    eprintln!("Bindle: could not look at GitHub: {}", error.message);
                }
            }
            tokio::time::sleep(LOOK_EVERY).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_github_pages_are_opened() {
        assert!(is_github_link("https://github.com/kunal-m/latch/pull/42"));
        assert!(!is_github_link("https://github.com/"));
        assert!(!is_github_link("http://github.com/a"));
        assert!(!is_github_link("https://github.com.evil.example/a"));
        assert!(!is_github_link("https://github.com/@evil.example"));
        assert!(!is_github_link("file:///etc/passwd"));
    }

    #[test]
    fn github_errors_become_plain_messages() {
        let error = CommandError::from(GithubError::BadToken);
        assert!(error.message.starts_with("GitHub did not accept the token"));
    }
}
