// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for Creator (handbook P14): the YouTube connection, the Creator
//! screen, and Make it a promise from a comment. Bindle only reads public
//! data with an API key kept in the system keychain; it never replies,
//! posts or changes anything on YouTube.

use super::{announce, lock, CommandError, Reply};
use crate::Db;
use bindle_core::Promise;
use bindle_sources::youtube::{self, Client, YoutubeError};
use bindle_store::creator::{self, CreatorAccount, CreatorView};
use chrono::{SecondsFormat, Utc};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State as Managed};

/// Sent to every window after a look at YouTube.
pub const CREATOR_CHANGED: &str = "creator-changed";

/// The first look waits until Bindle has settled; then every hour, which
/// keeps well inside the API key's free daily allowance.
const FIRST_LOOK: Duration = Duration::from_secs(40);
const LOOK_EVERY: Duration = Duration::from_secs(60 * 60);

/// The API key's name in the system keychain.
const KEY: &str = "youtube-api-key";

impl From<YoutubeError> for CommandError {
    fn from(error: YoutubeError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

fn plain(message: String) -> CommandError {
    CommandError { message }
}

/// Everything the Creator screen shows.
#[tauri::command]
pub fn creator_view(db: Managed<Db>) -> Reply<CreatorView> {
    Ok(creator::creator_view(&*lock(&db)?, Utc::now())?)
}

/// How many comments wait on your reply, for the sidebar.
#[tauri::command]
pub fn comments_waiting(db: Managed<Db>) -> Reply<u32> {
    Ok(creator::comments_waiting(&*lock(&db)?)?)
}

/// The page where a person makes an API key.
#[tauri::command]
pub fn youtube_key_page() -> &'static str {
    youtube::KEY_PAGE
}

/// Checks the channel and key with YouTube, keeps the key in the keychain,
/// and has a first look.
#[tauri::command]
pub async fn youtube_connect(
    app: AppHandle,
    channel: String,
    key: String,
) -> Reply<CreatorAccount> {
    super::real_only("connect YouTube")?;
    let (channel, key) = (channel.trim().to_string(), key.trim().to_string());
    if channel.is_empty() {
        return Err(plain("type your channel's @handle first".into()));
    }
    if key.is_empty() {
        return Err(plain("paste the API key first".into()));
    }
    let (found, _) = Client::new(&key)?.channel(&channel).await?;
    crate::secrets::set(KEY, &key).map_err(plain)?;
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        creator::save_youtube_account(
            &conn,
            &CreatorAccount {
                channel: Some(found.id.clone()),
                found: Some(found),
                ..Default::default()
            },
        )?;
    }
    refresh(&app).await
}

/// Forgets the key and what YouTube showed. Promises made stay.
#[tauri::command]
pub async fn youtube_disconnect(app: AppHandle) -> Reply<()> {
    super::real_only("change YouTube")?;
    crate::secrets::forget(KEY).map_err(plain)?;
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        creator::save_youtube_account(&conn, &CreatorAccount::default())?;
        creator::forget(&conn, "youtube")?;
    }
    let _ = app.emit(CREATOR_CHANGED, ());
    Ok(())
}

/// Looks at YouTube now.
#[tauri::command]
pub async fn youtube_refresh(app: AppHandle) -> Reply<CreatorAccount> {
    super::real_only("look at your own channel")?;
    refresh(&app).await
}

/// Makes a work promise to reply to a comment. It waits in Sort.
#[tauri::command]
pub fn comment_make_promise(app: AppHandle, db: Managed<Db>, comment_id: String) -> Reply<Promise> {
    let promise = creator::make_promise(&mut *lock(&db)?, &comment_id, Utc::now())?;
    announce(&app);
    let _ = app.emit(CREATOR_CHANGED, ());
    Ok(promise)
}

/// "Not needed": hides a comment in Bindle. YouTube is not touched.
#[tauri::command]
pub fn comment_hide(app: AppHandle, db: Managed<Db>, comment_id: String) -> Reply<()> {
    creator::hide(&*lock(&db)?, &comment_id, Utc::now())?;
    let _ = app.emit(CREATOR_CHANGED, ());
    Ok(())
}

/// Shows a hidden comment again.
#[tauri::command]
pub fn comment_unhide(app: AppHandle, db: Managed<Db>, comment_id: String) -> Reply<()> {
    creator::unhide(&*lock(&db)?, &comment_id)?;
    let _ = app.emit(CREATOR_CHANGED, ());
    Ok(())
}

/// Opens a YouTube page, or the Google Cloud page for making a key.
#[tauri::command]
pub fn open_on_youtube(app: AppHandle, url: String) -> Reply<()> {
    use tauri_plugin_opener::OpenerExt;
    if !is_youtube_link(&url) {
        return Err(plain(
            "Bindle only opens YouTube and Google Cloud links".into(),
        ));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| plain(format!("could not open the browser: {e}")))
}

fn is_youtube_link(url: &str) -> bool {
    [
        "https://www.youtube.com/",
        "https://studio.youtube.com/",
        "https://console.cloud.google.com/",
    ]
    .iter()
    .any(|start| {
        url.strip_prefix(start)
            .is_some_and(|rest| !rest.contains(['\\', '@', ' ']) || rest.starts_with('@'))
    })
}

/// Looks at YouTube and saves what it shows. A failed look is saved as the
/// account's error, so the Creator screen can say what went wrong.
pub(crate) async fn refresh(app: &AppHandle) -> Reply<CreatorAccount> {
    let db = app.state::<Db>();
    let saved = creator::youtube_account(&*lock(&db)?)?;
    let (Some(channel), Some(key)) = (saved.channel.clone(), crate::secrets::get(KEY)) else {
        return Ok(saved);
    };
    let now = Utc::now();
    let looked = match Client::new(&key) {
        Ok(client) => client.look(&channel, now).await,
        Err(error) => Err(error),
    };
    let account = {
        let mut conn = lock(&db)?;
        let mut account = creator::youtube_account(&conn)?;
        match looked {
            Ok(look) => {
                creator::sync(
                    &mut conn,
                    "youtube",
                    &look.channel,
                    &look.posts,
                    &look.comments,
                    now,
                )?;
                account.found = Some(look.channel);
                account.synced_at = Some(now.to_rfc3339_opts(SecondsFormat::Millis, true));
                account.error = None;
            }
            Err(error) => account.error = Some(error.to_string()),
        }
        creator::save_youtube_account(&conn, &account)?;
        account
    };
    let _ = app.emit(CREATOR_CHANGED, ());
    Ok(account)
}

/// Looks at YouTube in the background while Bindle runs, when connected.
pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_LOOK).await;
        loop {
            let connected = {
                let db = app.state::<Db>();
                lock(&db)
                    .ok()
                    .and_then(|conn| creator::youtube_account(&conn).ok())
                    .is_some_and(|a| a.channel.is_some())
            };
            if connected && !crate::showing_sample() {
                if let Err(error) = refresh(&app).await {
                    eprintln!("Bindle: could not look at YouTube: {}", error.message);
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
    fn only_youtube_and_cloud_pages_are_opened() {
        assert!(is_youtube_link(
            "https://www.youtube.com/watch?v=vid1&lc=t1"
        ));
        assert!(is_youtube_link("https://www.youtube.com/@aspersh"));
        assert!(is_youtube_link(
            "https://console.cloud.google.com/apis/credentials"
        ));
        assert!(!is_youtube_link("https://www.youtube.com.evil.example/"));
        assert!(!is_youtube_link("http://www.youtube.com/"));
        assert!(!is_youtube_link("https://evil.example/"));
    }
}
