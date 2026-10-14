// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Any mailbox, read over IMAP with an app password: Gmail, iCloud,
//! Fastmail, Yahoo, Zoho, Outlook (where the account still allows app
//! passwords) or a server the person types in.
//!
//! An app password is a password the mail service makes for one app (most
//! need 2-Step Verification on the account first). The app keeps it in the
//! system keychain. Bindle opens the inbox and sent mail read-only
//! (`EXAMINE`) and fetches with `BODY.PEEK`, so nothing is marked as read,
//! moved, labelled or sent.

use crate::mail::{read_mail, Mail, LOOK_BACK_DAYS};
use async_imap::types::NameAttribute;
use chrono::{DateTime, Duration, Utc};
use futures_util::TryStreamExt;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

/// A mail service Bindle knows how to reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Provider {
    /// What the account remembers: "gmail", "icloud", … or "other".
    pub id: &'static str,
    pub name: &'static str,
    pub host: &'static str,
    pub port: u16,
    /// Where a person makes an app password for Bindle.
    pub password_page: &'static str,
    /// The service's web mail, opened by "Open in …".
    pub webmail: &'static str,
    /// Anything a person should know before connecting, in plain words.
    pub note: &'static str,
}

/// The services with known settings, Gmail first. "other" takes a server
/// the person types in.
pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "gmail",
        name: "Gmail",
        host: "imap.gmail.com",
        port: 993,
        password_page: "https://myaccount.google.com/apppasswords",
        webmail: "https://mail.google.com/",
        note: "Needs 2-Step Verification on your Google account. Then make an app password named Bindle.",
    },
    Provider {
        id: "icloud",
        name: "iCloud Mail",
        host: "imap.mail.me.com",
        port: 993,
        password_page: "https://account.apple.com/account/manage",
        webmail: "https://www.icloud.com/mail/",
        note: "In your Apple Account, open Sign-In and Security, then App-Specific Passwords. Use your full iCloud address.",
    },
    Provider {
        id: "fastmail",
        name: "Fastmail",
        host: "imap.fastmail.com",
        port: 993,
        password_page: "https://app.fastmail.com/settings/security/apps",
        webmail: "https://app.fastmail.com/",
        note: "Make an app password with IMAP access only.",
    },
    Provider {
        id: "yahoo",
        name: "Yahoo Mail",
        host: "imap.mail.yahoo.com",
        port: 993,
        password_page: "https://login.yahoo.com/account/security",
        webmail: "https://mail.yahoo.com/",
        note: "In Account Security, choose Generate app password.",
    },
    Provider {
        id: "zoho",
        name: "Zoho Mail",
        host: "imap.zoho.com",
        port: 993,
        password_page: "https://accounts.zoho.com/home#security/app_password",
        webmail: "https://mail.zoho.com/",
        note: "Turn on IMAP access in Zoho Mail's settings first. Accounts outside the US may use imap.zoho.eu or imap.zoho.in: choose Another mailbox for those.",
    },
    Provider {
        id: "outlook",
        name: "Outlook",
        host: "outlook.office365.com",
        port: 993,
        password_page: "https://account.live.com/proofs/AppPassword",
        webmail: "https://outlook.live.com/mail/",
        note: "Microsoft has turned off app passwords for most Outlook.com and many work accounts. If yours refuses, Outlook needs Sign in with Microsoft, which is not in Bindle yet.",
    },
    Provider {
        id: "other",
        name: "Another mailbox",
        host: "",
        port: 993,
        password_page: "",
        webmail: "",
        note: "Type your mail service's IMAP server. Bindle only uses encrypted connections (usually port 993).",
    },
];

/// The provider with this id; Gmail when the id is unknown or missing,
/// which is what accounts saved before other mailboxes existed mean.
pub fn provider(id: Option<&str>) -> &'static Provider {
    PROVIDERS
        .iter()
        .find(|p| Some(p.id) == id)
        .unwrap_or(&PROVIDERS[0])
}

/// At most this many recent emails are read from each folder.
const MAX_PER_FOLDER: usize = 200;
/// Only the start of each email is read: enough for the new words.
const TEXT_BYTES: usize = 12_000;
/// A look that takes longer than this is given up.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(90);

/// Everything that can go wrong reading a mailbox, in plain words.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MailError {
    #[error("the mailbox did not accept the sign-in: {0}. Check the address, and use an app password, not your usual password.")]
    SignIn(String),
    #[error(
        "could not reach the mail server. Check the internet connection and the server's name."
    )]
    Offline,
    #[error("the mail server took too long to answer. Bindle will try again later.")]
    TimedOut,
    #[error("the mail server answered with an error: {0}")]
    Server(String),
}

/// What one look at a mailbox found: recent inbox and sent emails.
#[derive(Debug, Default)]
pub struct Recent {
    pub inbox: Vec<Mail>,
    pub sent: Vec<Mail>,
}

/// A mailbox and its app password.
pub struct Mailbox {
    address: String,
    password: String,
    host: String,
    port: u16,
    tls: bool,
}

impl Mailbox {
    /// A mailbox on `host` and `port`, always over TLS.
    pub fn new(host: &str, port: u16, address: &str, app_password: &str) -> Self {
        Self {
            address: address.trim().to_string(),
            // Services show app passwords in groups of four; spaces are not part of it.
            password: app_password
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect(),
            host: host.trim().to_lowercase(),
            port,
            tls: true,
        }
    }

    /// A Gmail mailbox.
    pub fn gmail(address: &str, app_password: &str) -> Self {
        let gmail = &PROVIDERS[0];
        Self::new(gmail.host, gmail.port, address, app_password)
    }

    /// The same account on a plain, unencrypted test server.
    pub fn on_test_server(address: &str, app_password: &str, port: u16) -> Self {
        Self {
            host: "127.0.0.1".into(),
            port,
            tls: false,
            ..Self::new("127.0.0.1", port, address, app_password)
        }
    }

    /// Signs in and out, to check the address and app password.
    pub async fn check(&self) -> Result<(), MailError> {
        self.visit(Utc::now(), false).await.map(|_| ())
    }

    /// Reads the last two weeks of inbox and sent mail.
    pub async fn recent(&self, now: DateTime<Utc>) -> Result<Recent, MailError> {
        self.visit(now, true).await
    }

    /// Signs in and, when `read` is true, reads both folders.
    async fn visit(&self, now: DateTime<Utc>, read: bool) -> Result<Recent, MailError> {
        let look = async {
            let tcp = TcpStream::connect((self.host.as_str(), self.port))
                .await
                .map_err(|_| MailError::Offline)?;
            if self.tls {
                let name = rustls::pki_types::ServerName::try_from(self.host.clone())
                    .map_err(|e| MailError::Server(e.to_string()))?;
                let tls = tls_connector()
                    .connect(name, tcp)
                    .await
                    .map_err(|_| MailError::Offline)?;
                self.look(tls, now, read).await
            } else {
                self.look(tcp, now, read).await
            }
        };
        tokio::time::timeout(TIMEOUT, look)
            .await
            .map_err(|_| MailError::TimedOut)?
    }

    async fn look<T>(&self, stream: T, now: DateTime<Utc>, read: bool) -> Result<Recent, MailError>
    where
        T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
    {
        let mut client = async_imap::Client::new(stream);
        client
            .read_response()
            .await
            .map_err(|_| MailError::Offline)?
            .ok_or(MailError::Offline)?;
        let mut session = client.login(&self.address, &self.password).await.map_err(
            |(error, _)| match error {
                async_imap::error::Error::No(text) | async_imap::error::Error::Bad(text) => {
                    MailError::SignIn(clean(&text))
                }
                async_imap::error::Error::Io(_) => MailError::Offline,
                other => MailError::SignIn(other.to_string()),
            },
        )?;
        let mut recent = Recent::default();
        if read {
            let since = imap_date(now - Duration::days(LOOK_BACK_DAYS));
            recent.inbox = read_folder(&mut session, "INBOX", &since).await?;
            if let Some(sent) = sent_folder(&mut session).await? {
                recent.sent = read_folder(&mut session, &sent, &since).await?;
            }
        }
        let _ = session.logout().await;
        Ok(recent)
    }
}

/// The sent mail folder, by its special-use mark, whatever its name is in
/// the account's language. Without the mark, the usual English names are
/// tried; a mailbox with none of them is read without its sent mail.
async fn sent_folder<T>(session: &mut async_imap::Session<T>) -> Result<Option<String>, MailError>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    let names: Vec<_> = session
        .list(Some(""), Some("*"))
        .await
        .map_err(server)?
        .try_collect()
        .await
        .map_err(server)?;
    let marked = names.iter().find(|n| {
        n.attributes()
            .iter()
            .any(|a| matches!(a, NameAttribute::Sent))
    });
    Ok(marked
        .or_else(|| {
            SENT_NAMES
                .iter()
                .find_map(|want| names.iter().find(|n| n.name().eq_ignore_ascii_case(want)))
        })
        .map(|n| n.name().to_string()))
}

/// Sent folders' usual names, for servers that do not mark it.
const SENT_NAMES: &[&str] = &[
    "[Gmail]/Sent Mail",
    "Sent",
    "Sent Items",
    "Sent Messages",
    "INBOX.Sent",
    "Sent Mail",
];

async fn read_folder<T>(
    session: &mut async_imap::Session<T>,
    folder: &str,
    since: &str,
) -> Result<Vec<Mail>, MailError>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    // EXAMINE opens the folder read-only.
    session.examine(folder).await.map_err(server)?;
    let mut uids: Vec<u32> = session
        .uid_search(format!("SINCE {since}"))
        .await
        .map_err(server)?
        .into_iter()
        .collect();
    uids.sort_unstable();
    let newest = &uids[uids.len().saturating_sub(MAX_PER_FOLDER)..];
    if newest.is_empty() {
        return Ok(Vec::new());
    }
    let set = newest
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let fetches: Vec<_> = session
        .uid_fetch(
            set,
            format!("(UID BODY.PEEK[HEADER] BODY.PEEK[TEXT]<0.{TEXT_BYTES}>)"),
        )
        .await
        .map_err(server)?
        .try_collect()
        .await
        .map_err(server)?;
    Ok(fetches
        .iter()
        .filter_map(|fetch| {
            let mut raw = fetch.header()?.to_vec();
            raw.extend_from_slice(fetch.text().unwrap_or_default());
            read_mail(&raw)
        })
        .collect())
}

fn server(error: async_imap::error::Error) -> MailError {
    match error {
        async_imap::error::Error::Io(_) => MailError::Offline,
        other => MailError::Server(clean(&other.to_string())),
    }
}

/// The server's own words, without the IMAP library's wrapping (`code:
/// None, info: Some("…")`) or bracketed codes ("[AUTHENTICATIONFAILED]").
fn clean(text: &str) -> String {
    let words = match text.split_once("info: Some(\"") {
        Some((_, rest)) => rest
            .rsplit_once("\")")
            .map_or(rest, |(inner, _)| inner)
            .replace("\\\"", "\""),
        None => text.to_string(),
    };
    let words = words.trim();
    let words = match words
        .strip_prefix('[')
        .and_then(|rest| rest.split_once(']'))
    {
        Some((_, rest)) => rest.trim(),
        None => words,
    };
    words
        .split(" (Failure)")
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// "30-Sep-2026", as IMAP's SEARCH SINCE wants it.
fn imap_date(at: DateTime<Utc>) -> String {
    at.format("%d-%b-%Y").to_string()
}

fn tls_connector() -> tokio_rustls::TlsConnector {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    tokio_rustls::TlsConnector::from(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn app_passwords_lose_their_spaces() {
        let gmail = Mailbox::gmail(" me@gmail.com ", "abcd efgh ijkl mnop");
        assert_eq!(gmail.address, "me@gmail.com");
        assert_eq!(gmail.password, "abcdefghijklmnop");
        assert_eq!(gmail.host, "imap.gmail.com");
    }

    #[test]
    fn providers_are_known_by_id_and_default_to_gmail() {
        assert_eq!(provider(Some("icloud")).host, "imap.mail.me.com");
        assert_eq!(provider(None).id, "gmail");
        assert_eq!(provider(Some("nope")).id, "gmail");
        for p in PROVIDERS.iter().filter(|p| p.id != "other") {
            assert_eq!(p.port, 993, "{} uses TLS on 993", p.name);
            assert!(p.webmail.starts_with("https://"));
            assert!(p.password_page.starts_with("https://"));
        }
    }

    #[test]
    fn gmail_codes_are_dropped_from_its_words() {
        assert_eq!(
            clean("[AUTHENTICATIONFAILED] Invalid credentials (Failure)"),
            "Invalid credentials"
        );
        assert_eq!(
            clean("Application-specific password required"),
            "Application-specific password required"
        );
        assert_eq!(
            clean(r#"code: None, info: Some("[ALERT] Application-specific password required: https://support.google.com/accounts/answer/185833 (Failure)")"#),
            "Application-specific password required: https://support.google.com/accounts/answer/185833"
        );
    }

    #[test]
    fn imap_dates_are_day_month_year() {
        let at = Utc.with_ymd_and_hms(2026, 9, 30, 23, 0, 0).unwrap();
        assert_eq!(imap_date(at), "30-Sep-2026");
    }
}
