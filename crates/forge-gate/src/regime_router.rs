//! Regime router: binary-quantized nearest-historical-regime matcher.
//! Ported from `F:\NewRepo\crates\forge-ml\src\bq_router.rs::BqRouter` —
//! specialists become regimes, XOR+POPCNT Hamming match replaces MoE routing.

/// Bytes per regime centroid. 512-bit market-state fingerprint.
pub const CENTROID_BYTES: usize = 64;

/// Quantized market-state feature dimension (512 bits -> 64 bytes).
pub const D_MODEL: usize = CENTROID_BYTES * 8;

/// Number of tracked historical regime classes.
pub const NUM_REGIMES: usize = 7;

/// Minimum training rows, and minimum positive-outcome rows, before a
/// regime centroid is trusted — a negatives-only centroid is anti-trained.
const MIN_RECORDS: usize = 5;

/// A trained binary centroid for one historical market regime.
#[derive(Clone, Copy)]
pub struct RegimeCentroid {
    pub bits: [u8; CENTROID_BYTES],
    pub record_count: usize,
    pub positive_count: usize,
    pub active: bool,
}

impl Default for RegimeCentroid {
    fn default() -> Self {
        Self { bits: [0u8; CENTROID_BYTES], record_count: 0, positive_count: 0, active: false }
    }
}

/// One labeled training row: a quantized market-state vector, the regime
/// it belongs to, and the outcome score of the strategy that ran in it.
pub struct RegimeTrainingPair {
    pub regime_id: u8,
    pub outcome_score: f32,
    pub query_i8: [i8; D_MODEL],
}

/// CPU-only regime router: matches a live market-state fingerprint to its
/// nearest historical regime analog in O(1) XOR+POPCNT, no tick replay.
#[derive(Clone, Copy)]
pub struct RegimeRouter {
    pub centroids: [RegimeCentroid; NUM_REGIMES],
}

impl RegimeRouter {
    pub fn new() -> Self {
        Self { centroids: [RegimeCentroid::default(); NUM_REGIMES] }
    }

    /// Route a live market-state fingerprint to its nearest active regime.
    /// Returns `(regime_id, margin)`; `None` if no regime is trained yet.
    pub fn route(&self, query_i8: &[i8; D_MODEL]) -> Option<(usize, u32)> {
        let query_bq = binarize_i8(query_i8);
        let mut best_id = 0usize;
        let mut best_dist = u32::MAX;
        let mut second_dist = u32::MAX;

        for (i, c) in self.centroids.iter().enumerate() {
            if !c.active {
                continue;
            }
            let dist = hamming(&query_bq, &c.bits);
            if dist < best_dist {
                second_dist = best_dist;
                best_dist = dist;
                best_id = i;
            } else if dist < second_dist {
                second_dist = dist;
            }
        }

        if best_dist == u32::MAX {
            return None;
        }
        Some((best_id, second_dist.saturating_sub(best_dist)))
    }

    /// Train regime centroids from labeled historical replay rows:
    /// outcome-weighted sign-vote aggregation per regime, then binarize.
    pub fn train_from_pairs(&mut self, pairs: &[RegimeTrainingPair]) {
        let mut votes = [[0.0f32; D_MODEL]; NUM_REGIMES];
        let mut counts = [0usize; NUM_REGIMES];
        let mut pos_counts = [0usize; NUM_REGIMES];

        for p in pairs {
            let rid = p.regime_id as usize;
            if rid >= NUM_REGIMES {
                continue;
            }
            counts[rid] += 1;
            let w = p.outcome_score * 2.0 - 1.0;
            if w > 0.0 {
                pos_counts[rid] += 1;
            }
            for (vote, &q) in votes[rid].iter_mut().zip(p.query_i8.iter()) {
                *vote += w * q as f32;
            }
        }

        for rid in 0..NUM_REGIMES {
            self.centroids[rid].record_count = counts[rid];
            self.centroids[rid].positive_count = pos_counts[rid];
            self.centroids[rid].active = counts[rid] >= MIN_RECORDS && pos_counts[rid] >= MIN_RECORDS;
            if counts[rid] > 0 {
                self.centroids[rid].bits = binarize_f32(&votes[rid]);
            }
        }
    }

    pub fn active_count(&self) -> usize {
        self.centroids.iter().filter(|c| c.active).count()
    }

    pub fn record_counts(&self) -> [usize; NUM_REGIMES] {
        let mut out = [0usize; NUM_REGIMES];
        for (slot, c) in out.iter_mut().zip(self.centroids.iter()) {
            *slot = c.record_count;
        }
        out
    }

    /// Serialize to a fixed byte buffer: `[active:u8, count:u32le, bits:64] x 7`.
    /// No file I/O here — the no_std gate never touches a filesystem.
    pub fn pack(&self) -> [u8; NUM_REGIMES * (1 + 4 + CENTROID_BYTES)] {
        const ENTRY: usize = 1 + 4 + CENTROID_BYTES;
        let mut out = [0u8; NUM_REGIMES * ENTRY];
        for i in 0..NUM_REGIMES {
            let off = i * ENTRY;
            out[off] = self.centroids[i].active as u8;
            out[off + 1..off + 5].copy_from_slice(&(self.centroids[i].record_count as u32).to_le_bytes());
            out[off + 5..off + 5 + CENTROID_BYTES].copy_from_slice(&self.centroids[i].bits);
        }
        out
    }

    /// Deserialize from the buffer produced by [`Self::pack`].
    pub fn unpack(data: &[u8; NUM_REGIMES * (1 + 4 + CENTROID_BYTES)]) -> Self {
        const ENTRY: usize = 1 + 4 + CENTROID_BYTES;
        let mut router = Self::new();
        for i in 0..NUM_REGIMES {
            let off = i * ENTRY;
            router.centroids[i].active = data[off] != 0;
            router.centroids[i].record_count =
                u32::from_le_bytes([data[off + 1], data[off + 2], data[off + 3], data[off + 4]]) as usize;
            router.centroids[i].bits.copy_from_slice(&data[off + 5..off + 5 + CENTROID_BYTES]);
        }
        router
    }
}

impl Default for RegimeRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Named archetype for each LUT slot in [`default_lut`]. Hand-specified
/// priors, not fitted from historical replay — no regime-labeled corpus
/// exists in this repo yet. Refine via `train_from_pairs` once one does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RegimeClass {
    BullishContango = 0,
    BearishPanic = 1,
    SidewaysThetaBurn = 2,
    EarningsIvCrush = 3,
    LiquidityVacuum = 4,
    SlowBleedBear = 5,
    MeltUpBull = 6,
}

impl RegimeClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::BullishContango => "bullish_contango",
            Self::BearishPanic => "bearish_panic",
            Self::SidewaysThetaBurn => "sideways_theta_burn",
            Self::EarningsIvCrush => "earnings_iv_crush",
            Self::LiquidityVacuum => "liquidity_vacuum",
            Self::SlowBleedBear => "slow_bleed_bear",
            Self::MeltUpBull => "melt_up_bull",
        }
    }

    /// Advisory routing note. All four tradeable classes now map to a
    /// strategy this crate ships (`strategy.rs`); `LiquidityVacuum`/
    /// `BearishPanic` remain refuse-to-trade advisories, not a strategy pick
    /// — `risk_router`/`order_dag` remain the actual veto.
    pub const fn routing_note(self) -> &'static str {
        match self {
            Self::BullishContango => "bull_put_spread",
            Self::BearishPanic => "no_trade_veto",
            Self::SidewaysThetaBurn => "iron_condor",
            Self::EarningsIvCrush => "iron_butterfly",
            Self::LiquidityVacuum => "critical_escalation_halt",
            Self::SlowBleedBear => "bear_call_spread",
            Self::MeltUpBull => "bull_put_spread",
        }
    }

    /// Maps a [`RegimeRouter::route`] `regime_id` back to its `RegimeClass`.
    /// `None` for any id outside `0..NUM_REGIMES` (defensive — `route`
    /// itself never returns one, since it only iterates `centroids`).
    pub const fn from_index(id: usize) -> Option<Self> {
        match id {
            0 => Some(Self::BullishContango),
            1 => Some(Self::BearishPanic),
            2 => Some(Self::SidewaysThetaBurn),
            3 => Some(Self::EarningsIvCrush),
            4 => Some(Self::LiquidityVacuum),
            5 => Some(Self::SlowBleedBear),
            6 => Some(Self::MeltUpBull),
            _ => None,
        }
    }
}

/// Hand-authored 5D market-state archetype for one LUT regime. Run through
/// `market_collapse::collapse_market_to_query` + `binarize_i8`, same as any
/// live fingerprint, so the LUT and live traffic share one encoding path.
const fn archetype(
    moneyness_pmy: i32,
    delta_pmy: i32,
    depth_pmy: u16,
    iv_skew_pmy: i32,
    dte_days: i32,
) -> crate::market_collapse::MarketPoint5D {
    crate::market_collapse::MarketPoint5D { moneyness_pmy, delta_pmy, depth_pmy, iv_skew_pmy, dte_days }
}

/// `iv_skew_pmy` below is `(iv_rank - 50) * 100`, iv_rank the standard
/// trailing-252-session 0..100 rank (>50 "elevated", <30 "low" —
/// Barchart/moomoo convention), so it is capped at +-5000 by construction.
/// The Panic/SidewaysThetaBurn/LiquidityVacuum/SlowBleedBear ranks below are
/// each a real trailing-252-session rank computed from the free CBOE VIX
/// daily-close history (`VIX_History.csv`, 1990-present) at a named date —
/// PROVEN this session, not estimated. EarningsIvCrush is single-name
/// idiosyncratic vol (no free historical single-name IV-rank series exists
/// to check against) and MeltUpBull (elevated IV during a steady uptrend is
/// atypical) are still hand-estimated, marked `[ASSUMED]` below.
/// The 7 hand-specified regime archetypes, indexed by [`RegimeClass`] as u8.
const ARCHETYPES: [fn() -> crate::market_collapse::MarketPoint5D; NUM_REGIMES] = [
    || archetype(3_000, 7_000, 9_000, -5_000, 30), // BullishContango: iv_rank=0 (2017-11-03 close 9.14, all-time-low calm), strong positive delta, deep book
    || archetype(-6_000, -9_000, 1_500, 5_000, 10), // BearishPanic: iv_rank~100 (2008-11-20 close 80.86 / 2020-03-16 close 82.69, both trailing-rank 99.6), extreme negative delta, thin book
    || archetype(0, 0, 7_000, -2_300, 45), // SidewaysThetaBurn: iv_rank=26.6 (2021-11-08 close 17.22, calm grind), flat delta, balanced book
    || archetype(500, 200, 6_000, 4_000, 5), // EarningsIvCrush: [ASSUMED] iv_rank~90 pre-print, single-name idiosyncratic, not checked against a free series
    || archetype(-8_000, -5_000, 0, 5_000, 2), // LiquidityVacuum: iv_rank~98-99.6 (2010-05-06/07 Flash Crash, close 32.80->40.95), zero depth (spread/depth evaporation, ScienceDirect/FasterCapital)
    || archetype(-3_000, -4_000, 5_000, 2_400, 35), // SlowBleedBear: iv_rank=74.2 (2019-06-03 close 18.86, trade-war selloff), steady negative drift
    || archetype(6_000, 6_000, 5_000, 2_000, 35), // MeltUpBull: [ASSUMED] elevated IV during a steady uptrend is atypical, no verified real-date analog found
];

/// Build the static regime LUT: each archetype collapsed through the same
/// `market_collapse` path live fingerprints use, then binarized. O(1) fixed
/// cost at construction (7 collapses), not on the per-tick `route` path.
pub fn default_lut() -> RegimeRouter {
    let mut router = RegimeRouter::new();
    for (i, make) in ARCHETYPES.iter().enumerate() {
        let point = make();
        let query = crate::market_collapse::collapse_market_to_query(point);
        router.centroids[i] = RegimeCentroid {
            bits: binarize_i8(&query),
            record_count: 0,
            positive_count: 0,
            active: true,
        };
    }
    router
}

/// Binarize an i8 market-state vector by sign bit, 8 values per byte.
#[inline]
pub fn binarize_i8(x: &[i8; D_MODEL]) -> [u8; CENTROID_BYTES] {
    let mut bits = [0u8; CENTROID_BYTES];
    for (chunk_idx, chunk) in x.chunks(8).enumerate().take(CENTROID_BYTES) {
        let mut byte = 0u8;
        for (bit_pos, &val) in chunk.iter().enumerate() {
            if val >= 0 {
                byte |= 1 << bit_pos;
            }
        }
        bits[chunk_idx] = byte;
    }
    bits
}

/// Binarize an f32 vote-vector by sign: positive -> 1, else -> 0.
#[inline]
fn binarize_f32(x: &[f32; D_MODEL]) -> [u8; CENTROID_BYTES] {
    let mut bits = [0u8; CENTROID_BYTES];
    for (chunk_idx, chunk) in x.chunks(8).enumerate().take(CENTROID_BYTES) {
        let mut byte = 0u8;
        for (bit_pos, &val) in chunk.iter().enumerate() {
            if val > 0.0 {
                byte |= 1 << bit_pos;
            }
        }
        bits[chunk_idx] = byte;
    }
    bits
}

/// Hamming distance over 64 bytes via safe 8x u64 XOR+POPCNT.
/// `deny(unsafe_code)` forbids the original's raw pointer reads;
/// `from_ne_bytes` gets the same POPCNT throughput without `unsafe`.
#[inline]
pub fn hamming(a: &[u8; CENTROID_BYTES], b: &[u8; CENTROID_BYTES]) -> u32 {
    let mut dist = 0u32;
    for i in 0..(CENTROID_BYTES / 8) {
        let mut xa = [0u8; 8];
        let mut xb = [0u8; 8];
        xa.copy_from_slice(&a[i * 8..i * 8 + 8]);
        xb.copy_from_slice(&b[i * 8..i * 8 + 8]);
        let wa = u64::from_ne_bytes(xa);
        let wb = u64::from_ne_bytes(xb);
        dist += (wa ^ wb).count_ones();
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binarize_chunk_packing() {
        let mut x = [0i8; D_MODEL];
        for (i, slot) in x.iter_mut().enumerate() {
            *slot = if i % 2 == 0 { 1 } else { -1 };
        }
        let bits = binarize_i8(&x);
        for &b in &bits {
            assert_eq!(b, 0x55);
        }
    }

    #[test]
    fn binarize_all_positive() {
        let bits = binarize_i8(&[1i8; D_MODEL]);
        assert!(bits.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn hamming_identical() {
        let a = [0xFFu8; CENTROID_BYTES];
        assert_eq!(hamming(&a, &a), 0);
    }

    #[test]
    fn hamming_opposite() {
        assert_eq!(hamming(&[0x00; CENTROID_BYTES], &[0xFF; CENTROID_BYTES]), 512);
    }

    #[test]
    fn hamming_single_bit() {
        let a = [0x00u8; CENTROID_BYTES];
        let mut b = [0x00u8; CENTROID_BYTES];
        b[0] = 0x01;
        assert_eq!(hamming(&a, &b), 1);
    }

    #[test]
    fn route_no_active_returns_none() {
        assert!(RegimeRouter::new().route(&[1i8; D_MODEL]).is_none());
    }

    #[test]
    fn route_picks_nearest() {
        let mut r = RegimeRouter::new();
        r.centroids[0] = RegimeCentroid { bits: [0xFF; CENTROID_BYTES], record_count: 10, positive_count: 10, active: true };
        r.centroids[1] = RegimeCentroid { bits: [0x00; CENTROID_BYTES], record_count: 10, positive_count: 10, active: true };

        let (id, margin) = r.route(&[1i8; D_MODEL]).unwrap();
        assert_eq!(id, 0);
        assert_eq!(margin, 512);

        let (id, margin) = r.route(&[-1i8; D_MODEL]).unwrap();
        assert_eq!(id, 1);
        assert_eq!(margin, 512);
    }

    #[test]
    fn threshold_gate_n5() {
        let mut r = RegimeRouter::new();
        let pairs: [RegimeTrainingPair; 9] = core::array::from_fn(|i| RegimeTrainingPair {
            regime_id: if i < 4 { 2 } else { 3 },
            outcome_score: if i < 4 { 0.9 } else { 0.8 },
            query_i8: [1i8; D_MODEL],
        });
        r.train_from_pairs(&pairs);

        assert!(!r.centroids[2].active); // 4 < 5
        assert_eq!(r.centroids[2].record_count, 4);
        assert!(r.centroids[3].active); // 5 >= 5
        assert_eq!(r.centroids[3].record_count, 5);
    }

    #[test]
    fn negative_only_centroid_never_reports_active() {
        let mut r = RegimeRouter::new();
        let pairs: [RegimeTrainingPair; 83] = core::array::from_fn(|_| RegimeTrainingPair {
            regime_id: 6,
            outcome_score: 0.0,
            query_i8: [1i8; D_MODEL],
        });
        r.train_from_pairs(&pairs);

        assert_eq!(r.centroids[6].record_count, 83);
        assert_eq!(r.centroids[6].positive_count, 0);
        assert!(!r.centroids[6].active, "a negatives-only centroid is anti-trained, not trained");
        assert_eq!(r.active_count(), 0);
    }

    #[test]
    fn pack_unpack_roundtrip() {
        let mut r = RegimeRouter::new();
        r.centroids[2] = RegimeCentroid { bits: [0xAB; CENTROID_BYTES], record_count: 42, positive_count: 42, active: true };
        r.centroids[5] = RegimeCentroid { bits: [0x13; CENTROID_BYTES], record_count: 7, positive_count: 7, active: true };

        let packed = r.pack();
        let loaded = RegimeRouter::unpack(&packed);

        assert!(loaded.centroids[2].active);
        assert_eq!(loaded.centroids[2].record_count, 42);
        assert_eq!(loaded.centroids[2].bits, [0xAB; CENTROID_BYTES]);
        assert!(loaded.centroids[5].active);
        assert!(!loaded.centroids[0].active);
    }

    #[test]
    fn default_lut_has_all_seven_active() {
        let r = default_lut();
        assert_eq!(r.active_count(), NUM_REGIMES);
    }

    #[test]
    fn default_lut_routes_exact_archetype_to_its_own_slot() {
        let r = default_lut();
        for (i, make) in ARCHETYPES.iter().enumerate() {
            let point = make();
            let query = crate::market_collapse::collapse_market_to_query(point);
            let (id, _margin) = r.route(&query).unwrap();
            assert_eq!(id, i, "archetype {i} ({}) must route to itself", RegimeClass::label(RegimeClass::from_index(i).unwrap()));
        }
    }

    #[test]
    fn from_index_round_trips_every_regime_class_and_refuses_out_of_range() {
        for i in 0..NUM_REGIMES {
            assert_eq!(RegimeClass::from_index(i).unwrap() as u8, i as u8);
        }
        assert!(RegimeClass::from_index(NUM_REGIMES).is_none());
    }
}
