// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Talking to a relay. Every body sent is sealed first; every body received
//! is opened here. The relay sees ids, write tokens and sealed bytes only.

use crate::codes::{Backup, Invite};
use crate::crypto::{CryptoError, Key, Sealed};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CloudError {
    #[error("could not reach Bindle Cloud. Changes stay on this device and go up when you are back online.")]
    Offline,
    #[error("this invite was already used, or it is more than 7 days old. Ask for a new one.")]
    InviteGone,
    #[error("nothing was found for that recovery kit on this server")]
    NotFound,
    #[error("Bindle Cloud refused: {0}")]
    Refused(String),
    #[error("Bindle Cloud answered with an error ({0}). Bindle will try again later.")]
    Server(u16),
    #[error("{0}")]
    Crypto(#[from] CryptoError),
    #[error("Bindle Cloud's answer was not what Bindle expected: {0}")]
    Unexpected(String),
}

/// A relay at a base URL, such as `https://cloud.example.com`.
#[derive(Clone)]
pub struct Relay {
    http: reqwest::Client,
    base: String,
}

#[derive(Deserialize)]
struct Added {
    seq: u64,
}

#[derive(Deserialize)]
struct Page {
    batches: Vec<PageBatch>,
    more: bool,
}

#[derive(Deserialize)]
struct PageBatch {
    seq: u64,
    sealed: Sealed,
}

impl Relay {
    pub fn new(base: &str) -> Result<Self, CloudError> {
        if rustls::crypto::CryptoProvider::get_default().is_none() {
            let _ = rustls::crypto::ring::default_provider().install_default();
        }
        let http = reqwest::Client::builder()
            .user_agent(concat!("Bindle/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| CloudError::Unexpected(e.to_string()))?;
        Ok(Self {
            http,
            base: base.trim().trim_end_matches('/').to_string(),
        })
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    async fn send<T: DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<Option<T>, CloudError> {
        let response = request.send().await.map_err(|_| CloudError::Offline)?;
        let status = response.status().as_u16();
        match status {
            200 | 201 => {
                let body = response.text().await.map_err(|_| CloudError::Offline)?;
                if body.trim().is_empty() {
                    return Ok(None);
                }
                serde_json::from_str(&body)
                    .map(Some)
                    .map_err(|e| CloudError::Unexpected(e.to_string()))
            }
            204 => Ok(None),
            404 => Err(CloudError::NotFound),
            410 => Err(CloudError::InviteGone),
            400..=499 => Err(CloudError::Refused(
                response.text().await.unwrap_or_default(),
            )),
            _ => Err(CloudError::Server(status)),
        }
    }

    /// Seals `plain` with the space key and adds it to the space. Returns its number.
    pub async fn push(&self, space: &Key, plain: &[u8]) -> Result<u64, CloudError> {
        let request = self
            .http
            .post(format!("{}/v1/spaces/{}/batches", self.base, space.id()))
            .bearer_auth(space.write_token())
            .json(&space.seal(plain));
        let added: Added = self
            .send(request)
            .await?
            .ok_or_else(|| CloudError::Unexpected("no number".into()))?;
        Ok(added.seq)
    }

    /// Every batch after `after`, opened, in order, with its number.
    pub async fn pull(
        &self,
        space: &Key,
        mut after: u64,
    ) -> Result<Vec<(u64, Vec<u8>)>, CloudError> {
        let mut all = Vec::new();
        loop {
            let request = self
                .http
                .get(format!("{}/v1/spaces/{}/batches", self.base, space.id()))
                .query(&[("after", after)])
                .bearer_auth(space.write_token());
            let page: Page = self
                .send(request)
                .await?
                .ok_or_else(|| CloudError::Unexpected("no page".into()))?;
            for batch in page.batches {
                all.push((batch.seq, space.open(&batch.sealed)?));
                after = batch.seq;
            }
            if !page.more {
                return Ok(all);
            }
        }
    }

    /// Leaves an invite on the relay, sealed with its own secret.
    pub async fn put_invite(&self, secret: &Key, invite: &Invite) -> Result<(), CloudError> {
        let plain = serde_json::to_vec(invite).expect("an invite serialises");
        let request = self
            .http
            .put(format!("{}/v1/invites/{}", self.base, secret.id()))
            .json(&secret.seal(&plain));
        self.send::<serde_json::Value>(request).await.map(|_| ())
    }

    /// Takes an invite: it works once, so no one else can use it after.
    pub async fn take_invite(&self, secret: &Key) -> Result<Invite, CloudError> {
        let request = self
            .http
            .post(format!("{}/v1/invites/{}/take", self.base, secret.id()));
        let sealed: Sealed = self.send(request).await?.ok_or(CloudError::InviteGone)?;
        open_json(secret, &sealed)
    }

    /// Keeps the list of spaces on the relay, sealed with the recovery key.
    pub async fn put_backup(&self, kit: &Key, backup: &Backup) -> Result<(), CloudError> {
        let plain = serde_json::to_vec(backup).expect("a backup serialises");
        let request = self
            .http
            .put(format!("{}/v1/kits/{}", self.base, kit.id()))
            .bearer_auth(kit.write_token())
            .json(&kit.seal(&plain));
        self.send::<serde_json::Value>(request).await.map(|_| ())
    }

    /// Brings back the list of spaces with a recovery kit.
    pub async fn get_backup(&self, kit: &Key) -> Result<Backup, CloudError> {
        let request = self
            .http
            .get(format!("{}/v1/kits/{}", self.base, kit.id()))
            .bearer_auth(kit.write_token());
        let sealed: Sealed = self.send(request).await?.ok_or(CloudError::NotFound)?;
        open_json(kit, &sealed)
    }
}

fn open_json<T: DeserializeOwned>(key: &Key, sealed: &Sealed) -> Result<T, CloudError> {
    serde_json::from_slice(&key.open(sealed)?).map_err(|e| CloudError::Unexpected(e.to_string()))
}

/// What one sealed batch holds: field changes from one device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Batch {
    pub ops: Vec<bindle_sync::FieldOp>,
}
