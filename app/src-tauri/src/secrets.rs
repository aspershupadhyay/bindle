// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Tokens and app passwords in the system keychain (handbook principle 5):
//! macOS Keychain, Windows Credential Manager, or the Secret Service on
//! Linux. Never in the database, an export or a safety copy.

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
