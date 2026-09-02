//! Adaptive Alpaca API polling pacer.
//!
//! Ported from `F:\v3\crates\forge-vision-v3\src\poll5d\pace.rs::Pacer`
//! (AIMD: Additive Increase, Multiplicative Decrease) — same domain-neutral
//! `{min,max,cur,step}` interval controller, renamed from motion-detection
//! polling to market-volatility-detection API polling. RAMUSPRIME spec:
//! high volatility polls at max rate, low volatility backs off toward a
//! 250ms floor.

/// Adaptive Alpaca-poll-interval pacer using AIMD.
#[derive(Debug, Clone, Copy)]
pub struct ApiPacer {
    min: u64,
    max: u64,
    cur: u64,
    step: u64,
}

impl ApiPacer {
    /// Create a new pacer with min/max interval and step size, in milliseconds.
    pub fn new(min_ms: u64, max_ms: u64, step_ms: u64) -> Self {
        let min = min_ms.max(1);
        let max = max_ms.max(min);
        Self { min, max, cur: min, step: step_ms.max(1).min(max) }
    }

    /// RAMUSPRIME default: 250ms floor, 5s idle ceiling, 250ms backoff step.
    pub fn default_market_pacer() -> Self {
        Self::new(250, 5_000, 250)
    }

    /// Called when a volatility spike is detected (high L2 hash variance);
    /// halves the polling interval toward the max-rate floor.
    pub fn on_volatility_spike(&mut self) {
        self.cur = (self.cur / 2).max(self.min);
    }

    /// Called when the market tick is quiet (static hash); increases the
    /// polling interval additively toward the ceiling.
    pub fn on_quiet_market(&mut self) {
        self.cur = (self.cur + self.step).min(self.max);
    }

    /// Route the pacer based on whether the L2 stream hash changed this tick.
    pub fn observe(&mut self, hash_changed: bool) {
        if hash_changed {
            self.on_volatility_spike();
        } else {
            self.on_quiet_market();
        }
    }

    /// Current polling interval in milliseconds.
    pub fn interval_ms(&self) -> u64 {
        self.cur
    }

    /// Get (min, max) bounds.
    pub fn bounds(&self) -> (u64, u64) {
        (self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boots_at_min() {
        assert_eq!(ApiPacer::new(100, 2000, 100).interval_ms(), 100);
    }

    #[test]
    fn quiet_market_grows_additively_and_clamps() {
        let mut p = ApiPacer::new(100, 500, 100);
        p.on_quiet_market();
        p.on_quiet_market();
        assert_eq!(p.interval_ms(), 300);
        for _ in 0..10 {
            p.on_quiet_market();
        }
        assert_eq!(p.interval_ms(), 500, "clamped at max");
    }

    #[test]
    fn volatility_spike_halves_toward_min_and_clamps() {
        let mut p = ApiPacer::new(100, 2000, 100);
        for _ in 0..30 {
            p.on_quiet_market();
        }
        assert_eq!(p.interval_ms(), 2000);
        p.on_volatility_spike();
        p.on_volatility_spike();
        assert_eq!(p.interval_ms(), 500);
        for _ in 0..10 {
            p.on_volatility_spike();
        }
        assert_eq!(p.interval_ms(), 100, "clamped at min");
    }

    #[test]
    fn observe_routes_signal() {
        let mut p = ApiPacer::new(100, 2000, 100);
        p.observe(false);
        assert_eq!(p.interval_ms(), 200);
        p.observe(true);
        assert_eq!(p.interval_ms(), 100);
    }

    #[test]
    fn default_market_pacer_has_250ms_floor() {
        let p = ApiPacer::default_market_pacer();
        assert_eq!(p.bounds(), (250, 5_000));
        assert_eq!(p.interval_ms(), 250);
    }
}
