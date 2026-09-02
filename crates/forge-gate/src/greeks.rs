//! Black-Scholes European option pricing and Greeks.
//!
//! Net-new — confirmed absent from every reachable primitive repo before
//! writing this (drain per T\<net-new\>: grepped `black_scholes|implied_vol|
//! greeks` case-insensitive across F:\v3, F:\13forge-super, and the Nistam
//! repo, zero hits). Unlike the ported modules, this file's correctness is
//! verified against independent textbook reference values (Hull), not
//! inherited trust from a proven source primitive.
//!
//! Zero Generative Cree Law analog: these Greeks are the ONLY source of
//! truth for strike selection — an LLM oracle is never permitted to emit a
//! delta value directly; `strategy.rs` must call into this module against a
//! real chain snapshot instead.

use libm::{erf, exp, log, sqrt};

/// Option type for pricing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Call,
    Put,
}

/// Standard normal cumulative distribution function via `erf`.
#[inline]
fn norm_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / core::f64::consts::SQRT_2))
}

/// Black-Scholes `d1` term.
#[inline]
fn d1(spot: f64, strike: f64, rate: f64, vol: f64, t_years: f64) -> f64 {
    (log(spot / strike) + (rate + 0.5 * vol * vol) * t_years) / (vol * sqrt(t_years))
}

/// Black-Scholes European option price.
///
/// `spot`/`strike` in dollars, `rate` and `vol` annualized decimals (e.g.
/// 0.05 = 5%), `t_years` time to expiration in years.
pub fn price(kind: OptionKind, spot: f64, strike: f64, rate: f64, vol: f64, t_years: f64) -> f64 {
    let d1v = d1(spot, strike, rate, vol, t_years);
    let d2v = d1v - vol * sqrt(t_years);
    let disc = exp(-rate * t_years);

    match kind {
        OptionKind::Call => spot * norm_cdf(d1v) - strike * disc * norm_cdf(d2v),
        OptionKind::Put => strike * disc * norm_cdf(-d2v) - spot * norm_cdf(-d1v),
    }
}

/// Black-Scholes delta: dPrice/dSpot. Call delta in `(0,1)`, put delta in `(-1,0)`.
pub fn delta(kind: OptionKind, spot: f64, strike: f64, rate: f64, vol: f64, t_years: f64) -> f64 {
    let d1v = d1(spot, strike, rate, vol, t_years);
    match kind {
        OptionKind::Call => norm_cdf(d1v),
        OptionKind::Put => norm_cdf(d1v) - 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hull, "Options, Futures, and Other Derivatives" worked ATM example:
    /// S=K=100, r=5%, sigma=20%, T=1yr. Reference: call ~10.4506, put ~5.5735,
    /// call delta ~0.6368.
    #[test]
    fn matches_hull_atm_reference_values() {
        let call = price(OptionKind::Call, 100.0, 100.0, 0.05, 0.2, 1.0);
        let put = price(OptionKind::Put, 100.0, 100.0, 0.05, 0.2, 1.0);
        assert!((call - 10.4506).abs() < 0.01, "call={call}");
        assert!((put - 5.5735).abs() < 0.01, "put={put}");

        let call_delta = delta(OptionKind::Call, 100.0, 100.0, 0.05, 0.2, 1.0);
        assert!((call_delta - 0.6368).abs() < 0.001, "call_delta={call_delta}");
    }

    /// Put-call parity: C - P = S - K*exp(-rT), an independent algebraic
    /// identity any correct implementation must satisfy exactly.
    #[test]
    fn satisfies_put_call_parity() {
        let spot = 137.50;
        let strike = 140.0;
        let rate = 0.04;
        let vol = 0.35;
        let t = 45.0 / 365.0;

        let call = price(OptionKind::Call, spot, strike, rate, vol, t);
        let put = price(OptionKind::Put, spot, strike, rate, vol, t);
        let parity_rhs = spot - strike * exp(-rate * t);

        assert!((call - put - parity_rhs).abs() < 1e-9, "call-put={}, rhs={}", call - put, parity_rhs);
    }

    /// Near-zero time-to-expiration, zero rate, ATM: d1 -> 0, delta -> 0.5.
    #[test]
    fn atm_zero_rate_near_expiry_delta_is_half() {
        let d = delta(OptionKind::Call, 100.0, 100.0, 0.0, 0.2, 0.0001);
        assert!((d - 0.5).abs() < 0.01, "delta={d}");
    }

    /// Deep out-of-the-money call delta approaches 0; deep ITM approaches 1.
    #[test]
    fn delta_bounds_at_extremes() {
        let deep_otm = delta(OptionKind::Call, 100.0, 200.0, 0.05, 0.2, 0.1);
        let deep_itm = delta(OptionKind::Call, 100.0, 20.0, 0.05, 0.2, 0.1);
        assert!(deep_otm < 0.01, "deep_otm={deep_otm}");
        assert!(deep_itm > 0.99, "deep_itm={deep_itm}");
    }

    /// Put delta is always call delta minus 1, for identical parameters.
    #[test]
    fn put_delta_equals_call_delta_minus_one() {
        let c = delta(OptionKind::Call, 137.5, 140.0, 0.04, 0.35, 45.0 / 365.0);
        let p = delta(OptionKind::Put, 137.5, 140.0, 0.04, 0.35, 45.0 / 365.0);
        assert!((c - p - 1.0).abs() < 1e-12);
    }
}
