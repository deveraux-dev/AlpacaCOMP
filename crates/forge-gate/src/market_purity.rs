//! Market purity metric: normalized order-book concentration (N x IPR).
//!
//! Ported near-verbatim from `F:\v3\crates\forge-hal-clockspine\src\nipr.rs`
//! (`NormalizedIpr`/`NiprPackedWord`) — the metric is already domain-neutral,
//! so it is fed L2 order-book depth-at-price sizes here instead of neural
//! activations. Deterministic, zero-transcendental, exact fixed-point in
//! Permyriad units (1 pmy = 0.01% = 10^-4).

use core::sync::atomic::{AtomicU64, Ordering};

/// Permyriad threshold at or above which the book is considered localized
/// (a sharp concentration at one price level).
pub const LANDMARK_PMY: u16 = 7500;

/// Permyriad threshold below which the book is considered diffuse
/// (delocalized / high entropy).
pub const DIFFUSE_PMY: u16 = 2500;

/// Canonical Permyriad scale (1.0 = 10,000 pmy).
pub const PERMYRIAD_SCALE: u128 = 10_000;

/// Normalized order-book concentration result over a discrete price basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizedIpr {
    /// Concentration in permyriad: 0 (uniform book) to 10000 (single-level spike).
    pub pmy: u16,
    /// Basis dimension N (number of price levels observed).
    pub dimension: u32,
    /// Total depth mass S1 = sum(v_i). 0 signifies an empty book.
    pub total_mass: u64,
    /// Second power sum S2 = sum(v_i^2), carried so channels can be joined
    /// without re-reading their depth levels — see `join`.
    pub second_moment: u64,
}

impl NormalizedIpr {
    /// Compute the normalized concentration metric over a non-negative
    /// discrete slice of `u16` depth-at-price sizes.
    pub fn compute_u16(slice: &[u16]) -> Self {
        let n = slice.len() as u32;
        if n == 0 {
            return Self::from_power_sums(0, 0, 0);
        }

        let mut s1: u64 = 0;
        let mut s2: u64 = 0;

        for &v in slice {
            let val = v as u64;
            s1 += val;
            s2 += val * val;
        }

        Self::from_power_sums(n, s1, s2)
    }

    fn from_power_sums(n: u32, s1: u64, s2: u64) -> Self {
        if n == 0 || s1 == 0 {
            return Self { pmy: 0, dimension: n, total_mass: 0, second_moment: 0 };
        }
        if n == 1 {
            return Self { pmy: 10_000, dimension: 1, total_mass: s1, second_moment: s2 };
        }

        let n_128 = n as u128;
        let s1_128 = s1 as u128;
        let s2_128 = s2 as u128;
        let s1_sq = s1_128 * s1_128;

        let n_s2 = n_128 * s2_128;
        let numerator = (n_s2 - s1_sq) * PERMYRIAD_SCALE;
        let denominator = (n_128 - 1) * s1_sq;

        let pmy = ((numerator / denominator).min(10_000)) as u16;

        Self {
            pmy,
            dimension: n,
            total_mass: s1,
            second_moment: s2,
        }
    }

    /// Join independent price-level channels (e.g. multiple venues/legs)
    /// into one concentration gauge over their concatenated basis, without
    /// re-reading a single depth level.
    pub fn join(parts: &[Self]) -> Self {
        let mut n: u32 = 0;
        let mut s1: u64 = 0;
        let mut s2: u64 = 0;
        for p in parts {
            n = n.saturating_add(p.dimension);
            s1 = s1.saturating_add(p.total_mass);
            s2 = s2.saturating_add(p.second_moment);
        }
        Self::from_power_sums(n, s1, s2)
    }

    /// O(1) silence sentinel: true iff the book is provably empty.
    #[inline]
    pub fn is_silent(&self) -> bool {
        self.second_moment == 0
    }

    /// True if the book is localized (>= 7500 pmy) and carries non-zero depth.
    #[inline]
    pub fn is_landmark(&self) -> bool {
        self.pmy >= LANDMARK_PMY && self.total_mass > 0
    }

    /// True if the book is diffuse (< 2500 pmy) or carries zero depth.
    #[inline]
    pub fn is_diffuse(&self) -> bool {
        self.pmy < DIFFUSE_PMY || self.total_mass == 0
    }

    /// RAMUSPRIME "chaos" trigger: the corrected mapping onto this real
    /// permyriad scale. A diffuse/low-concentration book is read as chaotic
    /// (no reliable price consensus), authorizing Tikhonov-style position
    /// damping. Superseeds the original spec's raw `< 1.0` threshold, which
    /// does not exist as code anywhere in this metric's real implementation.
    #[inline]
    pub fn is_chaotic(&self) -> bool {
        self.is_diffuse()
    }
}

/// Gate status enumeration for telemetry packed words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum NiprGateStatus {
    /// Initializing state.
    Init = 0,
    /// Active state with valid concentration metric.
    Active = 1,
    /// Fallback state (heuristic or constrained path active).
    Fallback = 2,
    /// Fault state (anomalous condition / violation detected).
    Fault = 3,
}

impl NiprGateStatus {
    /// Decode a 2-bit or 16-bit integer into a `NiprGateStatus`.
    #[inline]
    pub const fn from_raw(val: u16) -> Self {
        match val & 0x3 {
            0 => Self::Init,
            1 => Self::Active,
            2 => Self::Fallback,
            _ => Self::Fault,
        }
    }
}

/// 64-bit atomic packed telemetry word for zero-copy lock-free streaming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NiprPackedWord {
    /// Raw packed 64-bit word.
    pub raw: u64,
}

impl NiprPackedWord {
    /// Sentinel dimension value when true dimension N >= 65535.
    pub const DIMENSION_OVERFLOW_SENTINEL: u16 = 0xFFFF;

    /// Pack telemetry metrics into a 64-bit scalar word.
    #[inline]
    pub fn pack(pmy_level: u16, dimension: u32, gate_status: NiprGateStatus, sequence_tick: u16) -> Self {
        let dim_sat = if dimension >= 0xFFFF {
            Self::DIMENSION_OVERFLOW_SENTINEL
        } else {
            dimension as u16
        };

        let raw = (pmy_level as u64)
            | ((dim_sat as u64) << 16)
            | (((gate_status as u16) as u64) << 32)
            | ((sequence_tick as u64) << 48);

        Self { raw }
    }

    /// Construct a packed word directly from a `NormalizedIpr` evaluation.
    #[inline]
    pub fn from_ipr(ipr: &NormalizedIpr, gate_status: NiprGateStatus, sequence_tick: u16) -> Self {
        Self::pack(ipr.pmy, ipr.dimension, gate_status, sequence_tick)
    }

    /// Extract the Permyriad concentration level (0..=10000).
    #[inline]
    pub const fn pmy_level(&self) -> u16 {
        (self.raw & 0xFFFF) as u16
    }

    /// Extract the saturating basis dimension N (0..=65535).
    #[inline]
    pub const fn dimension_n(&self) -> u16 {
        ((self.raw >> 16) & 0xFFFF) as u16
    }

    /// True if the dimension saturated the 16-bit packed word boundary.
    #[inline]
    pub const fn is_dimension_overflow(&self) -> bool {
        self.dimension_n() == Self::DIMENSION_OVERFLOW_SENTINEL
    }

    /// Extract the gate status.
    #[inline]
    pub const fn gate_status(&self) -> NiprGateStatus {
        NiprGateStatus::from_raw(((self.raw >> 32) & 0xFFFF) as u16)
    }

    /// Extract the metronome sequence tick.
    #[inline]
    pub const fn sequence_tick(&self) -> u16 {
        ((self.raw >> 48) & 0xFFFF) as u16
    }

    /// Store the packed word into an `AtomicU64` with the specified memory ordering.
    #[inline]
    pub fn store_atomic(&self, dst: &AtomicU64, order: Ordering) {
        dst.store(self.raw, order);
    }

    /// Load a packed word from an `AtomicU64` with the specified memory ordering.
    #[inline]
    pub fn load_atomic(src: &AtomicU64, order: Ordering) -> Self {
        Self {
            raw: src.load(order),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_is_basis_independent() {
        // Fixed no_std-friendly bases in place of the original's runtime Vec.
        assert_eq!(NormalizedIpr::compute_u16(&[7u16; 2]).pmy, 0);
        assert_eq!(NormalizedIpr::compute_u16(&[7u16; 8]).pmy, 0);
        assert_eq!(NormalizedIpr::compute_u16(&[7u16; 64]).pmy, 0);

        let mut spike2 = [0u16; 2];
        spike2[0] = 9_999;
        assert_eq!(NormalizedIpr::compute_u16(&spike2).pmy, 10_000);

        let mut spike8 = [0u16; 8];
        spike8[0] = 9_999;
        assert_eq!(NormalizedIpr::compute_u16(&spike8).pmy, 10_000);
    }

    #[test]
    fn chaotic_book_matches_diffuse() {
        // A perfectly uniform book (no concentration) reads as chaotic.
        let uniform = NormalizedIpr::compute_u16(&[10, 10, 10, 10]);
        assert!(uniform.is_chaotic());
        assert_eq!(uniform.is_chaotic(), uniform.is_diffuse());

        // A single-level spike (strong consensus) is not chaotic.
        let spike = NormalizedIpr::compute_u16(&[100, 0, 0, 0]);
        assert!(!spike.is_chaotic());
    }

    #[test]
    fn empty_book_is_chaotic_and_silent() {
        let empty = NormalizedIpr::compute_u16(&[]);
        assert!(empty.is_chaotic());
        assert!(empty.is_silent());
    }

    #[test]
    fn packed_word_roundtrip() {
        let ipr = NormalizedIpr {
            pmy: 8500,
            dimension: 1024,
            total_mass: 50000,
            second_moment: 0,
        };

        let packed = NiprPackedWord::from_ipr(&ipr, NiprGateStatus::Active, 42);
        assert_eq!(packed.pmy_level(), 8500);
        assert_eq!(packed.dimension_n(), 1024);
        assert!(!packed.is_dimension_overflow());
        assert_eq!(packed.gate_status(), NiprGateStatus::Active);
        assert_eq!(packed.sequence_tick(), 42);

        let atomic = AtomicU64::new(0);
        packed.store_atomic(&atomic, Ordering::SeqCst);

        let loaded = NiprPackedWord::load_atomic(&atomic, Ordering::SeqCst);
        assert_eq!(loaded, packed);
    }
}
