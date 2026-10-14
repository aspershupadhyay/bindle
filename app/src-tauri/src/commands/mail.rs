// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Commands for Inbox (handbook P14): the mailbox connection (Gmail,
//! iCloud, Fastmail, Yahoo, Zoho, Outlook or any IMAP server), what needs a
//! reply, and Make it a promise. Bindle reads mail over IMAP with an app
//! password kept in the system keychain. It opens folders read-only, never
//! marks anything as read, and never sends email (handbook principle 6).

use super::{announce, lock, CommandError, Reply};
use crate::Db;
use bindle_core::Promise;
use bindle_sources::imap::{self, MailError, Mailbox, Provider};
use bindle_sources::mail::{gmail_link, reply_waits, triage};
use bindle_store::mail::{self, InboxView, MailAccount};
use chrono::{Local, Offset, SecondsFormat, Utc};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State as Managed};

/// Sent to every window after a look at the mailbox.
pub const MAIL_CHANGED: &str = "mail-changed";

/// The first look waits until Bindle has settled; then every 10 minutes.
const FIRST_LOOK: Duration = Duration::from_secs(30);
const LOOK_EVERY: Duration = Duration::from_secs(10 * 60);

/// The app password's name in the system keychain (named when Gmail was
/// the only mailbox; kept so connected accounts stay connected).
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

/// The mail services Bindle knows, with their servers and where to make
/// an app password.
#[tauri::command]
pub fn mail_providers() -> &'static [Provider] {
    imap::PROVIDERS
}

/// The mailbox an account points at.
fn mailbox(account: &MailAccount, address: &str, password: &str) -> Mailbox {
    let provider = imap::provider(account.provider.as_deref());
    match (provider.id, account.host.as_deref()) {
        ("other", Some(host)) => Mailbox::new(host, account.port.unwrap_or(993), address, password),
        _ => Mailbox::new(provider.host, provider.port, address, password),
    }
}

/// A server name as people type it: letters, digits, dots and dashes.
fn is_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host.contains('.')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

/// Checks the address and app password with the mail service, keeps the
/// password in the keychain, and has a first look.
#[tauri::command]
pub async fn gmail_connect(
    app: AppHandle,
    address: String,
    password: String,
    provider: Option<String>,
    host: Option<String>,
    port: Option<u16>,
) -> Reply<MailAccount> {
    let address = address.trim().to_lowercase();
    let known = imap::provider(provider.as_deref());
    if !address.contains('@') {
        return Err(plain(format!("type your {} address first", known.name)));
    }
    if password.trim().is_empty() {
        return Err(plain("paste the app password first".into()));
    }
    let (host, port) = if known.id == "other" {
        let host = host.unwrap_or_default().trim().to_lowercase();
        if !is_host(&host) {
            return Err(plain(
                "type the IMAP server's name, such as imap.example.com".into(),
            ));
        }
        (Some(host), Some(port.filter(|p| *p > 0).unwrap_or(993)))
    } else {
        (None, None)
    };
    let account = MailAccount {
        address: Some(address.clone()),
        provider: Some(known.id.to_string()),
        host,
        port,
        ..Default::default()
    };
    mailbox(&account, &address, &password).check().await?;
    crate::secrets::set(PASSWORD, &password).map_err(plain)?;
    {
        let db = app.state::<Db>();
        let conn = lock(&db)?;
        mail::save_gmail_account(&conn, &account)?;
    }
    refresh(&app).await
}

/// Forgets the app password and what the mailbox showed. Promises made stay.
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

/// Looks at the mailbox now.
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

/// "Not needed": hides an email in Bindle. The mailbox is not touched.
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

/// Opens an email in its service's web mail, or the page for making an
/// app password. Only the known services' pages are opened.
#[tauri::command]
pub fn open_in_gmail(app: AppHandle, url: String) -> Reply<()> {
    use tauri_plugin_opener::OpenerExt;
    if !is_mail_link(&url) {
        return Err(plain(
            "Bindle only opens the web mail and account pages of the mail services it knows".into(),
        ));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| plain(format!("could not open the browser: {e}")))
}

fn is_mail_link(url: &str) -> bool {
    imap::PROVIDERS
        .iter()
        .flat_map(|p| [p.webmail, p.password_page])
        .filter(|page| !page.is_empty())
        .any(|page| {
            // The page's own address, up to and including the first "/" after the host.
            let start = page
                .match_indices('/')
                .nth(2)
                .map_or(page, |(at, _)| &page[..=at]);
            url.strip_prefix(start)
                .is_some_and(|rest| !rest.contains(['\\', '@', ' ']))
        })
}

/// Looks at the mailbox and saves what needs you. A failed look is saved as the
/// account's error, so Inbox can say what went wrong.
pub(crate) async fn refresh(app: &AppHandle) -> Reply<MailAccount> {
    let db = app.state::<Db>();
    let saved = mail::gmail_account(&*lock(&db)?)?;
    let (Some(address), Some(password)) = (saved.address.clone(), crate::secrets::get(PASSWORD))
    else {
        return Ok(saved);
    };
    let now = Utc::now();
    let looked = mailbox(&saved, &address, &password).recent(now).await;
    let provider = imap::provider(saved.provider.as_deref());
    let zone = Local::now().offset().fix();
    let account = {
        let mut conn = lock(&db)?;
        let mut account = mail::gmail_account(&conn)?;
        match looked {
            Ok(recent) => {
                let webmail = provider.webmail.to_string();
                let open_in_webmail = move |_: &str| webmail.clone();
                let link: &dyn Fn(&str) -> String = if provider.id == "gmail" {
                    &gmail_link
                } else {
                    &open_in_webmail
                };
                let found = triage(&recent.inbox, &recent.sent, &address, now, zone, link);
                mail::sync(&mut conn, "gmail", &found, now)?;
                let waits = reply_waits(&recent.inbox, &recent.sent, &address);
                bindle_store::insights::record_waits(&mut conn, &waits)?;
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

/// Looks at the mailbox in the background while Bindle runs, when connected.
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
                    eprintln!("Bindle: could not look at the mailbox: {}", error.message);
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
    fn only_known_mail_pages_are_opened() {
        assert!(is_mail_link(
            "https://mail.google.com/mail/u/0/#search/rfc822msgid%3Aa"
        ));
        assert!(is_mail_link("https://myaccount.google.com/apppasswords"));
        assert!(is_mail_link("https://www.icloud.com/mail/"));
        assert!(is_mail_link("https://outlook.live.com/mail/"));
        assert!(!is_mail_link("https://mail.google.com.evil.example/"));
        assert!(!is_mail_link("http://mail.google.com/"));
        assert!(!is_mail_link("https://mail.google.com/@evil"));
        assert!(!is_mail_link("https://evil.example/"));
        assert!(!is_mail_link(""));
    }

    #[test]
    fn server_names_are_checked() {
        assert!(is_host("imap.zoho.eu"));
        assert!(!is_host("https://imap.example.com"));
        assert!(!is_host("localhost"));
        assert!(!is_host("imap example.com"));
    }
}
