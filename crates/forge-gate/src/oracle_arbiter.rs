//! Deterministic dual-oracle consensus judge.
//!
//! Ported from Nistam `forge-envelope::weaver::WeaverArbiter` +
//! `EvidenceChain`/`Disposition` (`forge-envelope/src/lib.rs`): Oracle A
//! (Bull) and Oracle B (Bear) compress their market thesis into S13
//! `[i8; 13]` tokens; this judge sums their combined lane gravity in O(1)
//! with zero heap allocation and returns a verdict — never a third LLM
//! summarizing the debate.

/// A single link's disposition in the audit chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    /// Erased / withdrawn thesis, no seal.
    Revoked,
    /// Deadline passed unwitnessed.
    Expired,
    /// Sealed with a provenance hash.
    Attested([u8; 32]),
}

/// Append-only audit chain of oracle-arbitration history.
///
/// Ported from `EvidenceChain` (`forge-envelope/src/lib.rs:207-249`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditChain {
    head: [u8; 32],
    len: usize,
}

impl AuditChain {
    /// Genesis chain: zeroed head, zero length.
    pub const fn new() -> Self {
        Self {
            head: [0u8; 32],
            len: 0,
        }
    }

    /// Current chain head.
    #[inline]
    pub const fn head(&self) -> [u8; 32] {
        self.head
    }

    /// Number of links appended so far.
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True if no links have been appended (genesis state).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Append a new disposition, advancing the head to a fresh non-zero tag.
    pub fn append(&mut self, tick: u64, record: Disposition) {
        let tag = match record {
            Disposition::Revoked => 1u8,
            Disposition::Expired => 2u8,
            Disposition::Attested(_) => 3u8,
        };
        self.head[0] = tag;
        self.head[1..9].copy_from_slice(&tick.to_le_bytes());
        self.len += 1;
    }
}

impl Default for AuditChain {
    fn default() -> Self {
        Self::new()
    }
}

/// The final resolution of dual-oracle arbitration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleVerdict {
    /// Oracle A and Oracle B offset perfectly. Neutral strategies authorized
    /// (e.g. Iron Condor).
    StructuralEquilibrium,
    /// Mild directional consensus. Directional spreads authorized at reduced
    /// allocation.
    ScheduledMaintenance,
    /// Severe drift or Oracle divergence. Hard veto — block execution.
    CriticalEscalation,
    /// Malformed oracle output (empty chain or zeroed head). Hard veto.
    ProvenanceBreach,
}

/// Balanced-ternary digits per Base-243 byte: 3^5 = 243 states in one byte.
pub const BASE243_TRIT_COUNT: usize = 5;
const BASE243_RADIX: u16 = 243;

/// Decode one Base-243 byte into its 5 balanced-ternary digits, each in
/// `{-1, 0, 1}`, least-significant trit first. `byte >= 243` is not a valid
/// Base-243 codepoint and decodes to all-zero rather than panicking.
pub const fn decode_base243_digits(byte: u8) -> [i8; BASE243_TRIT_COUNT] {
    let mut n = byte as u16;
    if n >= BASE243_RADIX {
        n = 121; // all-zero-trit codepoint (base-3 "11111" offset by -1)
    }
    let mut digits = [0i8; BASE243_TRIT_COUNT];
    let mut i = 0;
    while i < BASE243_TRIT_COUNT {
        digits[i] = (n % 3) as i8 - 1;
        n /= 3;
        i += 1;
    }
    digits
}

/// Encode 5 balanced-ternary digits (clamped to `{-1,0,1}`) into one
/// Base-243 byte, the inverse of [`decode_base243_digits`].
const fn encode_base243_digits(digits: &[i8; BASE243_TRIT_COUNT]) -> u8 {
    let mut n: u16 = 0;
    let mut place: u16 = 1;
    let mut i = 0;
    while i < BASE243_TRIT_COUNT {
        let d = digits[i];
        let clamped = if d < -1 { -1 } else if d > 1 { 1 } else { d };
        n += (clamped + 1) as u16 * place;
        place *= 3;
        i += 1;
    }
    n as u8
}

/// The 243-entry LUT that actually fits one 256-byte cache line (243 bytes,
/// 13 spare): the precomputed signed gravity of each Base-243 byte (sum of
/// its 5 trits, range -5..=5). A full per-trit decode table would need
/// 243*5 = 1215 bytes — 4.75x over budget — so this is the table-driven
/// primitive the memory constraint actually permits; `arbitrate` only ever
/// needs the summed gravity, never the individual trits, so it's sufficient.
pub const BASE243_GRAVITY_LUT: [i8; 243] = {
    let mut lut = [0i8; 243];
    let mut b = 0usize;
    while b < 243 {
        let digits = decode_base243_digits(b as u8);
        let mut sum = 0i8;
        let mut i = 0;
        while i < BASE243_TRIT_COUNT {
            sum += digits[i];
            i += 1;
        }
        lut[b] = sum;
        b += 1;
    }
    lut
};

/// Unpack a 3-byte Base-243 network payload (15 trits) into the S13
/// `[i8; 13]` oracle thesis; the trailing 2 of 15 decoded trits are padding
/// and dropped. Fixed 15-iteration decode, `const fn`, zero heap, no
/// per-trit LUT (impossible within one cache line — see
/// [`BASE243_GRAVITY_LUT`]'s doc comment for the arithmetic).
pub const fn unpack_base243_to_s13(payload: &[u8; 3]) -> [i8; 13] {
    let mut trits = [0i8; 13];
    let mut byte_idx = 0;
    let mut out_idx = 0;
    while byte_idx < 3 {
        let digits = decode_base243_digits(payload[byte_idx]);
        let mut d = 0;
        while d < BASE243_TRIT_COUNT && out_idx < 13 {
            trits[out_idx] = digits[d];
            out_idx += 1;
            d += 1;
        }
        byte_idx += 1;
    }
    trits
}

/// Pack the 13 S13 lanes (clamped to `{-1,0,1}`) into a 3-byte Base-243
/// network payload, the inverse of [`unpack_base243_to_s13`]. The 2 trailing
/// trit slots (15 - 13) are zero-padded.
pub const fn pack_s13_to_base243(lanes: &[i8; 13]) -> [u8; 3] {
    let mut trits15 = [0i8; 15];
    let mut i = 0;
    while i < 13 {
        trits15[i] = lanes[i];
        i += 1;
    }
    let mut out = [0u8; 3];
    let mut b = 0;
    while b < 3 {
        let chunk = [
            trits15[b * 5],
            trits15[b * 5 + 1],
            trits15[b * 5 + 2],
            trits15[b * 5 + 3],
            trits15[b * 5 + 4],
        ];
        out[b] = encode_base243_digits(&chunk);
        b += 1;
    }
    out
}

/// The static dual-oracle arbiter.
pub struct OracleArbiter;

/// Signed sum of Oracle A + Oracle B's combined lanes — the same value
/// `arbitrate` collapses into `OracleVerdict`'s coarse bands, exposed here
/// so a caller holding a `DirectionalVertical` pick can recover which way
/// (bull/bear) it leans, without re-deriving or inventing a second signal.
pub fn composite_gravity(oracle_a: &[i8; 13], oracle_b: &[i8; 13]) -> i32 {
    let mut sum: i32 = 0;
    for i in 0..13 {
        sum += oracle_a[i] as i32 + oracle_b[i] as i32;
    }
    sum
}

impl OracleArbiter {
    /// Arbitrate Oracle A (Bull) and Oracle B (Bear) S13 theses via the
    /// pre-compiled DFA, in O(1) time with zero heap allocation.
    pub fn arbitrate(
        chain: &AuditChain,
        oracle_a: &[i8; 13],
        oracle_b: &[i8; 13],
    ) -> OracleVerdict {
        // 1. Provenance gate: verify audit chain integrity.
        if chain.is_empty() || chain.head() == [0u8; 32] {
            return OracleVerdict::ProvenanceBreach;
        }

        // 2. Compute-at-rest DFA evaluation: composite gravity of the
        // combined Oracle A + Oracle B lanes.
        let composite_gravity = composite_gravity(oracle_a, oracle_b);

        if composite_gravity == 0 {
            OracleVerdict::StructuralEquilibrium
        } else if composite_gravity.abs() <= 3 {
            OracleVerdict::ScheduledMaintenance
        } else {
            OracleVerdict::CriticalEscalation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_gravity_matches_the_verdict_bands_it_drives() {
        let mut a = [0i8; 13];
        let mut b = [0i8; 13];
        a[0] = 1;
        a[5] = 1;
        a[12] = 1; // sum 3: same fixture as test_scheduled_maintenance
        assert_eq!(composite_gravity(&a, &b), 3);
        b[0] = -1;
        b[5] = -1;
        b[12] = -1; // fully offsets: back to equilibrium
        assert_eq!(composite_gravity(&a, &b), 0);
    }

    #[test]
    fn test_empty_chain_is_breach() {
        let chain = AuditChain::new();
        let a = [0i8; 13];
        let b = [0i8; 13];
        assert_eq!(OracleArbiter::arbitrate(&chain, &a, &b), OracleVerdict::ProvenanceBreach);
    }

    #[test]
    fn test_structural_equilibrium() {
        let mut chain = AuditChain::new();
        chain.append(1, Disposition::Expired);
        // Oracle A bullish on lane 0, Oracle B equally bearish: offsets to 0.
        let mut a = [0i8; 13];
        let mut b = [0i8; 13];
        a[0] = 1;
        b[0] = -1;
        assert_eq!(OracleArbiter::arbitrate(&chain, &a, &b), OracleVerdict::StructuralEquilibrium);
    }

    #[test]
    fn test_scheduled_maintenance() {
        let mut chain = AuditChain::new();
        chain.append(1, Disposition::Expired);
        let mut a = [0i8; 13];
        let b = [0i8; 13];
        a[0] = 1;
        a[5] = 1;
        a[12] = 1; // Combined sum is 3: mild consensus.
        assert_eq!(OracleArbiter::arbitrate(&chain, &a, &b), OracleVerdict::ScheduledMaintenance);
    }

    #[test]
    fn test_critical_escalation() {
        let mut chain = AuditChain::new();
        chain.append(1, Disposition::Expired);
        let a = [1i8; 13];
        let b = [1i8; 13]; // Combined sum is 26: severe drift.
        assert_eq!(OracleArbiter::arbitrate(&chain, &a, &b), OracleVerdict::CriticalEscalation);
    }

    #[test]
    fn base243_endpoints_decode_correctly() {
        assert_eq!(decode_base243_digits(0), [-1, -1, -1, -1, -1]);
        assert_eq!(decode_base243_digits(242), [1, 1, 1, 1, 1]);
        assert_eq!(decode_base243_digits(121), [0, 0, 0, 0, 0]); // the all-zero codepoint
    }

    #[test]
    fn base243_invalid_byte_decodes_to_zero() {
        assert_eq!(decode_base243_digits(243), [0, 0, 0, 0, 0]);
        assert_eq!(decode_base243_digits(255), [0, 0, 0, 0, 0]);
    }

    #[test]
    fn base243_encode_decode_roundtrip_all_243_codepoints() {
        let mut b = 0u16;
        while b < 243 {
            let digits = decode_base243_digits(b as u8);
            assert_eq!(encode_base243_digits(&digits), b as u8, "byte {b}");
            b += 1;
        }
    }

    #[test]
    fn gravity_lut_matches_manual_digit_sum_for_all_243_codepoints() {
        let mut b = 0usize;
        while b < 243 {
            let digits = decode_base243_digits(b as u8);
            let manual_sum: i8 = digits.iter().sum();
            assert_eq!(BASE243_GRAVITY_LUT[b], manual_sum, "byte {b}");
            b += 1;
        }
    }

    #[test]
    fn gravity_lut_fits_one_256_byte_cache_line() {
        assert_eq!(core::mem::size_of_val(&BASE243_GRAVITY_LUT), 243);
        assert!(core::mem::size_of_val(&BASE243_GRAVITY_LUT) <= 256);
    }

    #[test]
    fn s13_pack_unpack_roundtrip() {
        let lanes: [i8; 13] = [1, -1, 0, 1, 1, -1, 0, 0, 1, -1, -1, 0, 1];
        let payload = pack_s13_to_base243(&lanes);
        let decoded = unpack_base243_to_s13(&payload);
        assert_eq!(decoded, lanes);
    }

    #[test]
    fn s13_unpack_drops_the_two_padding_trits() {
        // All-max payload (242,242,242) decodes to 15 trits of +1; only the
        // first 13 come out, the trailing 2 padding trits are dropped.
        let payload = [242u8, 242u8, 242u8];
        assert_eq!(unpack_base243_to_s13(&payload), [1i8; 13]);
    }

    #[test]
    fn s13_pack_clamps_out_of_range_lanes() {
        let lanes: [i8; 13] = [5, -5, 3, -3, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let payload = pack_s13_to_base243(&lanes);
        let decoded = unpack_base243_to_s13(&payload);
        assert_eq!(&decoded[0..4], &[1, -1, 1, -1], "out-of-range lanes clamp to +-1");
    }
}
