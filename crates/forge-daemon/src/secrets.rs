//! RAII zeroize-on-drop wrapper for the Alpaca API secret key.
//! Ported from `F:\...\forge-daemon-door\src\mma_nostr.rs::SovereignActivations`
//! (ADR-0026). Heap-backed — belongs at this std daemon layer, never in the
//! `#![no_std]` forge-gate crate.

use zeroize::Zeroize;

/// A secret byte buffer that scrubs itself from RAM on drop.
pub struct SecureSecret(Vec<u8>);

impl SecureSecret {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn expose(&self) -> &[u8] {
        &self.0
    }

    /// Scrub now, without waiting for drop — e.g. right after a signed
    /// request is built and the raw key is no longer needed this tick.
    pub fn zeroize_now(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for SecureSecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeroize_now_scrubs_the_buffer() {
        let mut s = SecureSecret::new(b"alpaca-api-secret-key".to_vec());
        assert_eq!(s.expose(), b"alpaca-api-secret-key");
        s.zeroize_now();
        assert!(s.expose().iter().all(|&b| b == 0));
    }

    #[test]
    fn expose_returns_the_original_bytes_before_scrub() {
        let s = SecureSecret::new(vec![1, 2, 3, 4]);
        assert_eq!(s.expose(), &[1, 2, 3, 4]);
    }
}
