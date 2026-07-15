//! Session-token storage in the OS keychain (macOS Keychain via the `keyring` crate; the Windows
//! Credential Manager and the Linux Secret Service back the follow-on targets). The token is never
//! written to a plaintext file or a log line - TASK-APP-002 clause 4.

const SERVICE: &str = "os.cyberskill.world.desktop";
const ACCOUNT: &str = "session-token";

fn entry() -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, ACCOUNT)
}

pub fn set_token(token: &str) -> Result<(), keyring::Error> {
    entry()?.set_password(token)
}

pub fn get_token() -> Result<String, keyring::Error> {
    entry()?.get_password()
}

pub fn clear_token() -> Result<(), keyring::Error> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        // Clearing a token that was never stored is a no-op, not an error.
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e),
    }
}
