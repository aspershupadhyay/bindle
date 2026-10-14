// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Tokens and app passwords in the system keychain (handbook principle 5):
//! macOS Keychain, Windows Credential Manager, or the Secret Service on
//! Linux. Never in the database, an export or a safety copy.
//!
//! Phones have no keychain the `keyring` crate can reach, so there they are
//! kept in a file in the app's private storage, which other apps cannot
//! read, and which safety copies and exports never include. A proper
//! Android Keystore or iOS Keychain plugin would be better (see STATUS).

use std::path::Path;

/// Tells the phone store where the app's private folder is. Does nothing
/// on desktops, where the keychain is used.
pub fn use_folder(dir: &Path) {
    #[cfg(mobile)]
    phone::use_folder(dir);
    #[cfg(not(mobile))]
    let _ = dir;
}

#[cfg(mobile)]
pub use phone::{forget, get, set};

#[cfg(not(mobile))]
pub use desktop::{forget, get, set, with_entry};

#[cfg(not(mobile))]
mod desktop {

    const SERVICE: &str = "app.bindle.desktop";

    /// Runs a keychain call for `name` on its own thread. Some keychains run
    /// their own event loop, which must not start inside the app's async runtime.
    pub fn with_entry<T: Send + 'static>(
        name: &str,
        call: impl FnOnce(keyring::Entry) -> keyring::Result<T> + Send + 'static,
    ) -> keyring::Result<T> {
        let name = name.to_string();
        std::thread::spawn(move || call(keyring::Entry::new(SERVICE, &name)?))
            .join()
            .unwrap_or(Err(keyring::Error::NoEntry))
    }

    /// The secret saved under `name`, if there is one.
    pub fn get(name: &str) -> Option<String> {
        with_entry(name, |entry| entry.get_password()).ok()
    }

    /// Saves `secret` under `name`.
    pub fn set(name: &str, secret: &str) -> Result<(), String> {
        let secret = secret.to_string();
        with_entry(name, move |entry| entry.set_password(&secret))
            .map_err(|e| format!("Bindle could not keep it in the system keychain: {e}"))
    }

    /// Forgets the secret under `name`. Forgetting nothing is fine.
    pub fn forget(name: &str) -> Result<(), String> {
        match with_entry(name, |entry| entry.delete_credential()) {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!(
                "Bindle could not remove it from the system keychain: {e}"
            )),
        }
    }
}

#[cfg(mobile)]
mod phone {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    static FILE: OnceLock<PathBuf> = OnceLock::new();
    static LOCK: Mutex<()> = Mutex::new(());

    pub fn use_folder(dir: &Path) {
        let _ = FILE.set(dir.join("secrets.json"));
    }

    fn read() -> BTreeMap<String, String> {
        FILE.get()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    fn write(all: &BTreeMap<String, String>) -> Result<(), String> {
        let file = FILE.get().ok_or("the app's folder is not ready yet")?;
        let text = serde_json::to_string(all).expect("secrets serialise");
        std::fs::write(file, text)
            .map_err(|e| format!("Bindle could not keep it on this phone: {e}"))
    }

    pub fn get(name: &str) -> Option<String> {
        let _held = LOCK.lock().ok()?;
        read().remove(name)
    }

    pub fn set(name: &str, secret: &str) -> Result<(), String> {
        let _held = LOCK.lock().map_err(|e| e.to_string())?;
        let mut all = read();
        all.insert(name.to_string(), secret.to_string());
        write(&all)
    }

    pub fn forget(name: &str) -> Result<(), String> {
        let _held = LOCK.lock().map_err(|e| e.to_string())?;
        let mut all = read();
        if all.remove(name).is_some() {
            write(&all)?;
        }
        Ok(())
    }
}
