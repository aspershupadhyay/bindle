// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! What people copy, paste, print and type: invite links and recovery kits.
//!
//! An invite link carries its secret after the `#`. Browsers and apps never
//! send that part to a server, so the relay holding the invite cannot open
//! it. A recovery kit is the recovery key written in 4-letter groups of
//! Crockford base32, which has no letters that look alike (no I, L, O, U),
//! plus a check letter pair so a typo is caught before anything is fetched.

use crate::crypto::{CryptoError, Key};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Where invite links point. The page there explains how to open one in
/// Bindle; the secret after `#` never reaches it.
pub const INVITE_BASE: &str = "https://aspershupadhyay.github.io/bindle/join/#";

/// What an invite seals: everything needed to join one Ours space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invite {
    pub space_key: String,
    pub space_name: String,
    /// Who sent it, so the person joining knows: "Sparsh invites you".
    pub from: String,
    /// The relay the space lives on.
    pub relay: String,
}

/// The link for an invite secret on `relay`. Both ride after the `#`: the
/// secret so no server sees it, the relay so the app knows where to ask.
pub fn invite_link(secret: &Key, relay: &str) -> String {
    use base64::Engine;
    let relay = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(relay);
    format!("{INVITE_BASE}{}.{relay}", secret.to_text())
}

/// Reads an invite secret, and the relay when the link names one, from a
/// pasted link or the bare code.
pub fn read_invite_link(text: &str) -> Result<(Key, Option<String>), CryptoError> {
    use base64::Engine;
    let text = text.trim();
    let code = text.rsplit_once('#').map_or(text, |(_, code)| code);
    let (secret, relay) = match code.split_once('.') {
        Some((secret, relay)) => {
            let relay = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(relay)
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .filter(|r| r.starts_with("https://") || r.starts_with("http://"))
                .ok_or(CryptoError::BadKey)?;
            (secret, Some(relay))
        }
        None => (code, None),
    };
    Ok((Key::from_text(secret)?, relay))
}

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// The recovery kit for a recovery key: "7KQ2-…", 14 groups of up to 4.
pub fn recovery_text(key: &Key) -> String {
    let mut bytes = key.as_bytes().to_vec();
    bytes.push(Sha256::digest(key.as_bytes())[0]);
    let mut letters = String::new();
    let (mut buffer, mut bits) = (0u32, 0u32);
    for byte in bytes {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            letters.push(ALPHABET[((buffer >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        letters.push(ALPHABET[((buffer << (5 - bits)) & 31) as usize] as char);
    }
    letters
        .as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).expect("ASCII"))
        .collect::<Vec<_>>()
        .join("-")
}

/// Reads a typed recovery kit. Case, spaces and dashes do not matter, and
/// the look-alikes I, L (one) and O (zero) are read as meant.
pub fn read_recovery_text(text: &str) -> Result<Key, CryptoError> {
    let (mut buffer, mut bits) = (0u32, 0u32);
    let mut bytes = Vec::new();
    for c in text.chars().filter(|c| !c.is_whitespace() && *c != '-') {
        let c = match c.to_ascii_uppercase() {
            'I' | 'L' => '1',
            'O' => '0',
            c => c,
        };
        let value = ALPHABET
            .iter()
            .position(|&a| a as char == c)
            .ok_or(CryptoError::BadKey)? as u32;
        buffer = (buffer << 5) | value;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    if bytes.len() != 33 {
        return Err(CryptoError::BadKey);
    }
    let check = bytes.pop().expect("33 bytes");
    let key: [u8; 32] = bytes.try_into().expect("32 bytes");
    if Sha256::digest(key)[0] != check {
        return Err(CryptoError::BadKey);
    }
    Ok(Key::from_bytes(key))
}

/// What a recovery kit seals on the relay: every space the person is in.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backup {
    pub name: String,
    pub spaces: Vec<BackupSpace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupSpace {
    pub key: String,
    pub name: String,
    pub relay: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_invite_link_keeps_its_secret_after_the_hash() {
        let secret = Key::random();
        let link = invite_link(&secret, "https://cloud.example.com");
        let (page, code) = link.split_once('#').unwrap();
        assert!(!page.contains(&secret.to_text()));
        assert!(code.starts_with(&secret.to_text()));
        assert_eq!(
            read_invite_link(&link).unwrap(),
            (secret.clone(), Some("https://cloud.example.com".into()))
        );
        assert_eq!(
            read_invite_link(&format!("  {}\n", secret.to_text())).unwrap(),
            (secret, None)
        );
        assert!(read_invite_link("https://example.com/#nope").is_err());
        assert!(
            read_invite_link(&format!("#{}.bm90IGEgdXJs", Key::random().to_text())).is_err(),
            "relay must be a web address"
        );
    }

    #[test]
    fn a_recovery_kit_reads_back_however_it_is_typed() {
        let key = Key::random();
        let kit = recovery_text(&key);
        assert_eq!(kit.split('-').count(), 14);
        assert!(kit
            .chars()
            .all(|c| c == '-' || ALPHABET.contains(&(c as u8))));
        assert_eq!(read_recovery_text(&kit).unwrap(), key);
        let sloppy = kit
            .to_lowercase()
            .replace('-', " ")
            .replace('1', "l")
            .replace('0', "o");
        assert_eq!(read_recovery_text(&sloppy).unwrap(), key);
    }

    #[test]
    fn a_typo_in_a_recovery_kit_is_caught() {
        let kit = recovery_text(&Key::from_bytes([42; 32]));
        let first = kit.chars().next().unwrap();
        let other = if first == 'A' { 'B' } else { 'A' };
        let typo = format!("{other}{}", &kit[1..]);
        assert_eq!(read_recovery_text(&typo), Err(CryptoError::BadKey));
        assert_eq!(read_recovery_text("ABCD-EFGH"), Err(CryptoError::BadKey));
        assert_eq!(read_recovery_text("UUUU"), Err(CryptoError::BadKey));
    }
}
