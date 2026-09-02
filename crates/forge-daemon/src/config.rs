//! Alpaca credential/endpoint config. Base URLs are public, not secret — the
//! key/secret pair is, and never leaves an env var for a `SecureSecret`.

use crate::secrets::SecureSecret;

/// Paper-trading API base — the only endpoint this hackathon submission
/// trades against (`CLAUDE.md` submission_checklist: fresh paper account,
/// $100,000 reset balance).
pub const ALPACA_PAPER_BASE_URL: &str = "https://paper-api.alpaca.markets/v2";

/// Live-trading API base. Present for completeness only — nothing in this
/// repo is authorized to point at it; the daemon must default to paper.
pub const ALPACA_LIVE_BASE_URL: &str = "https://api.alpaca.markets/v2";

pub struct AlpacaCredentials {
    pub key_id: SecureSecret,
    pub secret_key: SecureSecret,
    pub base_url: String,
}

/// Load credentials from `APCA_API_KEY_ID`/`APCA_API_SECRET_KEY`. Always
/// defaults `base_url` to paper — going live is not this function's call to
/// make.
pub fn load_from_env() -> Result<AlpacaCredentials, String> {
    load_from(|name| std::env::var(name))
}

/// Injectable-lookup variant so tests never touch real process env vars.
fn load_from(lookup: impl Fn(&str) -> Result<String, std::env::VarError>) -> Result<AlpacaCredentials, String> {
    let key_id = lookup("APCA_API_KEY_ID").map_err(|_| "APCA_API_KEY_ID not set".to_string())?;
    let secret_key = lookup("APCA_API_SECRET_KEY").map_err(|_| "APCA_API_SECRET_KEY not set".to_string())?;

    if key_id.trim().is_empty() || secret_key.trim().is_empty() {
        return Err("APCA_API_KEY_ID/APCA_API_SECRET_KEY must not be empty".to_string());
    }

    Ok(AlpacaCredentials {
        key_id: SecureSecret::new(key_id.into_bytes()),
        secret_key: SecureSecret::new(secret_key.into_bytes()),
        base_url: ALPACA_PAPER_BASE_URL.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::env::VarError;

    fn fake_env(vars: &[(&str, &str)]) -> impl Fn(&str) -> Result<String, VarError> {
        let map: HashMap<String, String> = vars.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |name: &str| map.get(name).cloned().ok_or(VarError::NotPresent)
    }

    #[test]
    fn loads_both_vars_and_defaults_to_paper_url() {
        let creds = load_from(fake_env(&[
            ("APCA_API_KEY_ID", "PKTESTKEY"),
            ("APCA_API_SECRET_KEY", "test-secret-value"),
        ])).expect("both vars present");

        assert_eq!(creds.key_id.expose(), b"PKTESTKEY");
        assert_eq!(creds.secret_key.expose(), b"test-secret-value");
        assert_eq!(creds.base_url, ALPACA_PAPER_BASE_URL);
    }

    #[test]
    fn missing_key_id_is_a_named_error_not_a_panic() {
        match load_from(fake_env(&[("APCA_API_SECRET_KEY", "x")])) {
            Err(e) => assert!(e.contains("APCA_API_KEY_ID")),
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn missing_secret_key_is_a_named_error_not_a_panic() {
        match load_from(fake_env(&[("APCA_API_KEY_ID", "x")])) {
            Err(e) => assert!(e.contains("APCA_API_SECRET_KEY")),
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn empty_string_vars_are_rejected() {
        match load_from(fake_env(&[("APCA_API_KEY_ID", ""), ("APCA_API_SECRET_KEY", "x")])) {
            Err(e) => assert!(e.contains("must not be empty")),
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn never_defaults_to_the_live_url() {
        let creds = load_from(fake_env(&[
            ("APCA_API_KEY_ID", "k"),
            ("APCA_API_SECRET_KEY", "s"),
        ])).unwrap();
        assert_ne!(creds.base_url, ALPACA_LIVE_BASE_URL);
    }
}
