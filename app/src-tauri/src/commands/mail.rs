// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for Inbox (handbook P14): the Gmail connection, what needs a
//! reply, and Make it a promise. Bindle reads Gmail over IMAP with an app
//! password kept in the system keychain. It opens folders read-only, never
//! marks anything as read, and never sends email (handbook principle 6).

use super::{announce, lock, CommandError, Reply};
use crate::Db;
use bindle_core::Promise;
use bindle_sources::gmail::{self, Gmail, MailError};
use bindle_sources::mail::{gmail_link, triage};
use bindle_store::mail::{self, InboxView, MailAccount};
use chrono::{Local, Offset, SecondsFormat, Utc};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State as Managed};

/// Sent to every window after a look at Gmail.
pub const MAIL_CHANGED: &str = "mail-changed";

/// The first look waits until Bindle has settled; then every 10 minutes.
const FIRST_LOOK: Duration = Duration::from_secs(30);
const LOOK_EVERY: Duration = Duration::from_secs(10 * 60);

/// The app password's name in the system keychain.
const PASSWORD: &str = "gmail-app-password";

impl From<MailError> for CommandError {
    fn from(error: MailError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

fn plain(message: String) -> CommandError {
    CommandError { message }
}

/// Everything the Inbox screen shows.
#[tauri::command]
pub fn inbox_view(db: Managed<Db>) -> Reply<InboxView> {
    Ok(mail::inbox_view(&*lock(&db)?)?)
}

/// How many emails wait on your reply, for the sidebar.
#[tauri::command]
pub fn replies_owed(db: Managed<Db>) -> Reply<u32> {
    Ok(mail::replies_owed(&*lock(&db)?)?)
}

/// The page where a person makes an app password.
#[tauri::command]
pub fn gmail_password_page() -> &'static str {
    gmail::APP_PASSWORD_URL
}

/// Checks the address and app password with Gmail, keeps the password in
/// the keychain, and has a first look.
#[tauri::command]
pub async fn gmail_connect(
    app: AppHandle,
    address: String,
    password: String,
) -> Reply<MailAccount> {
    let address = address.trim().to_lowercase();
    if !address.contains('@') {
        return Err(plain("type your Gmail address first".into()));
    }
    if password.trim().is_empty() {
        return Err(plain("paste the app password from Google first".into()));
    }
    Gmail::new(&address, &password).check().await?;
    crate::secrets::set(PASSWORD, &password).map_err(plain)?;
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        mail::save_gmail_account(
            &conn,
            &MailAccount {
                address: Some(address),
                ..Default::default()
            },
        )?;
    }
    refresh(&app).await
}

/// Forgets the app password and what Gmail showed. Promises made stay.
#[tauri::command]
pub async fn gmail_disconnect(app: AppHandle) -> Reply<()> {
    crate::secrets::forget(PASSWORD).map_err(plain)?;
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        mail::save_gmail_account(&conn, &MailAccount::default())?;
        mail::forget(&conn, "gmail")?;
    }
    let _ = app.emit(MAIL_CHANGED, ());
    Ok(())
}

/// Looks at Gmail now.
#[tauri::command]
pub async fn gmail_refresh(app: AppHandle) -> Reply<MailAccount> {
    refresh(&app).await
}

/// Makes a work promise from an email. It waits in Sort.
#[tauri::command]
pub fn mail_make_promise(app: AppHandle, db: Managed<Db>, item_id: String) -> Reply<Promise> {
    let promise = mail::make_promise(&mut *lock(&db)?, &item_id, Utc::now())?;
    announce(&app);
    let _ = app.emit(MAIL_CHANGED, ());
    Ok(promise)
}

/// "Not needed": hides an email in Bindle. Gmail is not touched.
#[tauri::command]
pub fn mail_dismiss(app: AppHandle, db: Managed<Db>, item_id: String) -> Reply<()> {
    mail::dismiss(&*lock(&db)?, &item_id, Utc::now())?;
    let _ = app.emit(MAIL_CHANGED, ());
    Ok(())
}

/// Shows a hidden email again.
#[tauri::command]
pub fn mail_restore(app: AppHandle, db: Managed<Db>, item_id: String) -> Reply<()> {
    mail::restore(&*lock(&db)?, &item_id)?;
    let _ = app.emit(MAIL_CHANGED, ());
    Ok(())
}

/// Opens an email in Gmail's web page. Only Gmail links are opened.
#[tauri::command]
pub fn open_in_gmail(app: AppHandle, url: String) -> Reply<()> {
    use tauri_plugin_opener::OpenerExt;
    if !is_gmail_link(&url) {
        return Err(plain(
            "Bindle only opens Gmail and Google account links".into(),
        ));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| plain(format!("could not open the browser: {e}")))
}

fn is_gmail_link(url: &str) -> bool {
    ["https://mail.google.com/", "https://myaccount.google.com/"]
        .iter()
        .any(|start| {
            url.strip_prefix(start)
                .is_some_and(|rest| !rest.contains(['\\', '@', ' ']))
        })
}

/// Looks at Gmail and saves what needs you. A failed look is saved as the
/// account's error, so Inbox can say what went wrong.
pub(crate) async fn refresh(app: &AppHandle) -> Reply<MailAccount> {
    let db = app.state::<Db>();
    let address = mail::gmail_account(&*lock(&db)?)?.address;
    let (Some(address), Some(password)) = (address, crate::secrets::get(PASSWORD)) else {
        return Ok(mail::gmail_account(&*lock(&db)?)?);
    };
    let now = Utc::now();
    let looked = Gmail::new(&address, &password).recent(now).await;
    let zone = Local::now().offset().fix();
    let account = {
        let mut conn = lock(&db)?;
        let mut account = mail::gmail_account(&conn)?;
        match looked {
            Ok(recent) => {
                let found = triage(&recent.inbox, &recent.sent, &address, now, zone, gmail_link);
                mail::sync(&mut conn, "gmail", &found, now)?;
                account.synced_at = Some(now.to_rfc3339_opts(SecondsFormat::Millis, true));
                account.error = None;
            }
            Err(error) => account.error = Some(error.to_string()),
        }
        mail::save_gmail_account(&conn, &account)?;
        account
    };
    let _ = app.emit(MAIL_CHANGED, ());
    Ok(account)
}

/// Looks at Gmail in the background while Bindle runs, when connected.
pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_LOOK).await;
        loop {
            let connected = {
                let db = app.state::<Db>();
                lock(&db)
                    .ok()
                    .and_then(|conn| mail::gmail_account(&conn).ok())
                    .is_some_and(|a| a.address.is_some())
            };
            if connected {
                if let Err(error) = refresh(&app).await {
                    eprintln!("Bindle: could not look at Gmail: {}", error.message);
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
    fn only_gmail_pages_are_opened() {
        assert!(is_gmail_link(
            "https://mail.google.com/mail/u/0/#search/rfc822msgid%3Aa"
        ));
        assert!(is_gmail_link("https://myaccount.google.com/apppasswords"));
        assert!(!is_gmail_link("https://mail.google.com.evil.example/"));
        assert!(!is_gmail_link("http://mail.google.com/"));
        assert!(!is_gmail_link("https://mail.google.com/@evil"));
    }
}
