// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Keys and sealing. A [`Key`] is 32 random bytes. Everything the relay
//! needs (an id, a write token) and the encryption key itself are derived
//! from it with HKDF-SHA256, each with its own label, so knowing the id or
//! the token says nothing about the encryption key. Sealing is
//! XChaCha20-Poly1305 with a fresh random nonce, bound to the id it belongs
//! to, so a batch cannot be moved to another space unnoticed.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

/// 32 secret bytes: a space key, an invite secret or a recovery key.
#[derive(Clone, PartialEq, Eq)]
pub struct Key([u8; 32]);

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print a secret, even in logs.
        write!(f, "Key({})", self.id())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("this was sealed with another key, or it was changed on the way")]
    CannotOpen,
    #[error("that is not a Bindle key")]
    BadKey,
}

impl Key {
    /// A new random key from the system's secure random source.
    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).expect("the system has a secure random source");
        Self(bytes)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The key as text, for the keychain and invite links.
    pub fn to_text(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }

    pub fn from_text(text: &str) -> Result<Self, CryptoError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(text.trim())
            .map_err(|_| CryptoError::BadKey)?;
        Ok(Self(bytes.try_into().map_err(|_| CryptoError::BadKey)?))
    }

    fn derive(&self, label: &str) -> [u8; 32] {
        let mut out = [0u8; 32];
        Hkdf::<Sha256>::new(None, &self.0)
            .expand(label.as_bytes(), &mut out)
            .expect("32 bytes is a valid HKDF length");
        out
    }

    /// The public name of whatever this key protects: 32 hex characters.
    pub fn id(&self) -> String {
        self.derive("bindle id v1")[..16]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Proof, for the relay, that a device holds this key. The relay keeps
    /// only a hash of it.
    pub fn write_token(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.derive("bindle write token v1"))
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new(&self.derive("bindle encryption v1").into())
    }

    /// Seals `plain` so only a holder of this key can read it.
    pub fn seal(&self, plain: &[u8]) -> Sealed {
        let mut nonce = [0u8; 24];
        getrandom::fill(&mut nonce).expect("the system has a secure random source");
        let id = self.id();
        let cipher = self
            .cipher()
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plain,
                    aad: id.as_bytes(),
                },
            )
            .expect("encryption of a byte slice cannot fail");
        Sealed {
            nonce: URL_SAFE_NO_PAD.encode(nonce),
            data: URL_SAFE_NO_PAD.encode(cipher),
        }
    }

    /// Opens something sealed with this key.
    pub fn open(&self, sealed: &Sealed) -> Result<Vec<u8>, CryptoError> {
        let nonce = URL_SAFE_NO_PAD
            .decode(&sealed.nonce)
            .map_err(|_| CryptoError::CannotOpen)?;
        let data = URL_SAFE_NO_PAD
            .decode(&sealed.data)
            .map_err(|_| CryptoError::CannotOpen)?;
        if nonce.len() != 24 {
            return Err(CryptoError::CannotOpen);
        }
        let id = self.id();
        self.cipher()
            .decrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &data,
                    aad: id.as_bytes(),
                },
            )
            .map_err(|_| CryptoError::CannotOpen)
    }
}

/// Something sealed, as it travels and as the relay stores it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sealed {
    pub nonce: String,
    pub data: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_sealed_opens_only_with_the_same_key() {
        let key = Key::random();
        let sealed = key.seal(b"Pay electricity bill");
        assert_eq!(key.open(&sealed).unwrap(), b"Pay electricity bill");
        assert_eq!(Key::random().open(&sealed), Err(CryptoError::CannotOpen));
        assert!(!sealed.data.contains("electricity"));
    }

    #[test]
    fn a_changed_batch_is_refused() {
        let key = Key::random();
        let mut sealed = key.seal(b"hello");
        let mut bytes = URL_SAFE_NO_PAD.decode(&sealed.data).unwrap();
        bytes[0] ^= 1;
        sealed.data = URL_SAFE_NO_PAD.encode(bytes);
        assert_eq!(key.open(&sealed), Err(CryptoError::CannotOpen));
    }

    #[test]
    fn each_seal_uses_a_fresh_nonce() {
        let key = Key::random();
        assert_ne!(key.seal(b"same").nonce, key.seal(b"same").nonce);
    }

    #[test]
    fn the_id_token_and_key_are_unrelated_but_stable() {
        let key = Key::from_bytes([7; 32]);
        assert_eq!(key.id().len(), 32);
        assert_eq!(key.id(), Key::from_bytes([7; 32]).id());
        assert_ne!(key.id(), Key::from_bytes([8; 32]).id());
        assert!(!key.write_token().contains(&key.to_text()));
        assert_eq!(Key::from_text(&key.to_text()).unwrap(), key);
        assert_eq!(Key::from_text("short"), Err(CryptoError::BadKey));
        assert!(!format!("{key:?}").contains(&key.to_text()));
    }
}
