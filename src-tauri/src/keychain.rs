//! Secure credential storage via the platform's native keychain.
//!
//! | Platform | Backend |
//! |----------|---------|
//! | macOS    | macOS Keychain (Keychain.app) |
//! | Linux    | D-Bus Secret Service (GNOME Keyring / KWallet) |
//! | Windows  | Windows Credential Manager |
//!
//! All secrets are stored under the service name `"link-mage"`.
//! The *account* name is the service identifier (e.g. `"virustotal"`).

use keyring::{Entry, Error as KeyringError};

const SERVICE: &str = "link-mage";

// ── Internal helpers ──────────────────────────────────────────────────────────

fn entry(account: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, account).map_err(|e| format!("Keychain entry error: {e}"))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Retrieve the stored key for `account`.
///
/// Returns `Ok(Some(key))` if the key exists, `Ok(None)` if it has never been
/// set, or `Err(msg)` on a platform keychain error.
pub fn get_key(account: &str) -> Result<Option<String>, String> {
    match entry(account)?.get_password() {
        Ok(pw) => Ok(Some(pw)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(e) => Err(format!("Keychain read error: {e}")),
    }
}

/// Store `key` for `account`, creating or overwriting any existing entry.
pub fn set_key(account: &str, key: &str) -> Result<(), String> {
    entry(account)?
        .set_password(key)
        .map_err(|e| format!("Keychain write error: {e}"))
}

/// Delete the key for `account`.  Deleting a non-existent entry is a no-op.
pub fn delete_key(account: &str) -> Result<(), String> {
    match entry(account)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(format!("Keychain delete error: {e}")),
    }
}

/// Check whether a key is stored for `account` without returning the value.
pub fn has_key(account: &str) -> Result<bool, String> {
    match entry(account)?.get_password() {
        Ok(_) => Ok(true),
        Err(KeyringError::NoEntry) => Ok(false),
        Err(e) => Err(format!("Keychain check error: {e}")),
    }
}

// ── Known service identifiers ─────────────────────────────────────────────────

/// The keychain account name for the VirusTotal API key.
pub const VIRUSTOTAL: &str = "virustotal";

/// All service IDs that the settings UI should display.
pub const ALL_SERVICES: &[&str] = &[VIRUSTOTAL];
