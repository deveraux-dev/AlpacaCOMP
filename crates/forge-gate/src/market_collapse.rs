//! Market-state collapse: 5 scalar option-chain dimensions -> the [i8; 512]
//! query fingerprint `regime_router` binarizes and Hamming-matches.
//! Thermometer-coded (monotonic per-dimension bands), not a ported DSP law —
//! acoustic pan/pitch formulas carry no meaning over option-chain data.

use crate::regime_router::D_MODEL;

/// Number of tracked market dimensions.
const NUM_DIMS: usize = 5;

/// Slots per dimension band. 512 / 5 = 102; the 2 leftover slots stay -1
/// (neutral), never silently folded into a band's threshold spacing.
const BAND_SLOTS: usize = D_MODEL / NUM_DIMS;

/// Target DTE horizon (days) the strategy layer trades against (45 DTE).
pub const DTE_MAX_DAYS: i32 = 45;

/// A 5D market-state point, one field per collapsed dimension:
/// spot-vs-strike, option delta, book depth, vol skew, time to expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketPoint5D {
    /// Signed permyriad distance of spot from strike. Negative = ITM side. [X]
    pub moneyness_pmy: i32,
    /// Option delta, permyriad, -10000..=10000. [Y]
    pub delta_pmy: i32,
    /// Order-book concentration, permyriad 0..=10000 (feed from
    /// `market_purity::NormalizedIpr::pmy` — not recomputed here). [Z]
    pub depth_pmy: u16,
    /// Signed permyriad implied-vol skew / IVR. [theta]
    pub iv_skew_pmy: i32,
    /// Days to expiration, clamped 0..=`DTE_MAX_DAYS`. [W]
    pub dte_days: i32,
}

/// Collapse a market-state point into the `regime_router` query fingerprint.
/// Pure integer, no_std, zero heap: each dimension thermometer-codes into
/// its own band so Hamming distance between two fingerprints tracks the
/// numeric distance between the market states that produced them.
pub fn collapse_market_to_query(p: MarketPoint5D) -> [i8; D_MODEL] {
    let mut q = [-1i8; D_MODEL];

    thermometer_band(p.moneyness_pmy, -10_000, 10_000, &mut q[0 * BAND_SLOTS..1 * BAND_SLOTS]);
    thermometer_band(p.delta_pmy, -10_000, 10_000, &mut q[1 * BAND_SLOTS..2 * BAND_SLOTS]);
    thermometer_band(p.depth_pmy as i32, 0, 10_000, &mut q[2 * BAND_SLOTS..3 * BAND_SLOTS]);
    thermometer_band(p.iv_skew_pmy, -10_000, 10_000, &mut q[3 * BAND_SLOTS..4 * BAND_SLOTS]);
    thermometer_band(p.dte_days, 0, DTE_MAX_DAYS, &mut q[4 * BAND_SLOTS..5 * BAND_SLOTS]);

    q
}

/// Fill `out` with a monotonic thermometer code for `value` clamped to
/// `[lo, hi]`: slot `j` is `+1` once `value` has crossed that slot's
/// evenly-spaced threshold, else `-1`. Integer-only threshold spacing.
fn thermometer_band(value: i32, lo: i32, hi: i32, out: &mut [i8]) {
    let v = value.clamp(lo, hi);
    let span = (hi - lo).max(1);
    let n = out.len().max(1);
    for (j, slot) in out.iter_mut().enumerate() {
        // threshold_j = lo + span * j / (n - 1), n==1 falls through to lo.
        let threshold = if n == 1 { lo } else { lo + (span * j as i32) / (n as i32 - 1) };
        *slot = if v >= threshold { 1 } else { -1 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regime_router::binarize_i8;

    fn base() -> MarketPoint5D {
        MarketPoint5D { moneyness_pmy: 0, delta_pmy: 0, depth_pmy: 0, iv_skew_pmy: 0, dte_days: 0 }
    }

    #[test]
    fn min_value_band_is_mostly_low() {
        let q = collapse_market_to_query(MarketPoint5D { moneyness_pmy: -10_000, ..base() });
        // slot 0's threshold is `lo` itself, so `v >= lo` always passes there.
        assert_eq!(q[0], 1, "the floor still clears its own zeroth threshold");
        assert_eq!(q[BAND_SLOTS - 1], -1, "the floor never clears the ceiling threshold");
    }

    #[test]
    fn max_value_band_is_all_high() {
        let q = collapse_market_to_query(MarketPoint5D { moneyness_pmy: 10_000, ..base() });
        assert!(q[0..BAND_SLOTS].iter().all(|&s| s == 1), "at the ceiling every threshold in the band is crossed");
    }

    #[test]
    fn thermometer_is_monotonic_in_hamming_distance() {
        let low = collapse_market_to_query(MarketPoint5D { moneyness_pmy: -5_000, ..base() });
        let mid = collapse_market_to_query(MarketPoint5D { moneyness_pmy: 0, ..base() });
        let high = collapse_market_to_query(MarketPoint5D { moneyness_pmy: 5_000, ..base() });

        let bits_low = binarize_i8(&low);
        let bits_mid = binarize_i8(&mid);
        let bits_high = binarize_i8(&high);

        let d_low_mid = crate::regime_router::hamming(&bits_low, &bits_mid);
        let d_low_high = crate::regime_router::hamming(&bits_low, &bits_high);
        assert!(d_low_high >= d_low_mid, "farther moneyness must not be Hamming-closer");
    }

    #[test]
    fn dte_band_respects_clamp_ceiling() {
        let over = collapse_market_to_query(MarketPoint5D { dte_days: 999, ..base() });
        let at_max = collapse_market_to_query(MarketPoint5D { dte_days: DTE_MAX_DAYS, ..base() });
        assert_eq!(over[4 * BAND_SLOTS..5 * BAND_SLOTS], at_max[4 * BAND_SLOTS..5 * BAND_SLOTS]);
    }

    #[test]
    fn deterministic() {
        let p = MarketPoint5D { moneyness_pmy: 1234, delta_pmy: -3000, depth_pmy: 8500, iv_skew_pmy: -200, dte_days: 21 };
        assert_eq!(collapse_market_to_query(p), collapse_market_to_query(p));
    }
}
