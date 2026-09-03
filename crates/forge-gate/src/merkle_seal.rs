//! Merkle-Morin seal: 64-byte verified header + SHA-256 root over 64-byte
//! leaves. Header/container ported from Nistam gemma-s13/s13.rs:642-765;
//! the streaming fold is net-new (drain: leaf builders exist repo-wide, no fold).

use sha2::{Digest, Sha256};

/// Header magic, byte-compatible with the Nistam S13 container format.
pub const SEAL_MAGIC: [u8; 4] = *b"S13M";

/// Fixed leaf size: one 64-byte cache line (320 trits when carrying S13).
pub const LEAF_BYTES: usize = 64;

/// Base-243 sentinel boundary carried in the header, as in the source format.
pub const SENTINEL_BOUNDARY: u8 = 243;

/// Maximum fold depth: 2^40 leaves ≈ 70 TB payload, far past any consumer.
const MAX_DEPTH: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealError {
    TooShort,
    BadMagic,
    BadVersion,
    PayloadTruncated,
    RootMismatch,
}

/// 64-byte aligned verified-container header. Field layout and byte offsets
/// match `MerkleMorinHeader::to_bytes` in the source format exactly.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MerkleMorinHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub flags: u16,
    pub rows: u32,
    pub cols: u32,
    pub merkle_root: [u8; 32],
    pub leaf_size_bytes: u16,
    pub scale_permyriad: i32,
    pub sentinel_boundary: u8,
    pub _reserved: [u8; 11],
}

impl MerkleMorinHeader {
    pub const fn new(rows: u32, cols: u32, merkle_root: [u8; 32], scale_permyriad: i32) -> Self {
        Self {
            magic: SEAL_MAGIC,
            version: 1,
            flags: 0,
            rows,
            cols,
            merkle_root,
            leaf_size_bytes: LEAF_BYTES as u16,
            scale_permyriad,
            sentinel_boundary: SENTINEL_BOUNDARY,
            _reserved: [0u8; 11],
        }
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        out[0..4].copy_from_slice(&self.magic);
        out[4..6].copy_from_slice(&self.version.to_le_bytes());
        out[6..8].copy_from_slice(&self.flags.to_le_bytes());
        out[8..12].copy_from_slice(&self.rows.to_le_bytes());
        out[12..16].copy_from_slice(&self.cols.to_le_bytes());
        out[16..48].copy_from_slice(&self.merkle_root);
        out[48..50].copy_from_slice(&self.leaf_size_bytes.to_le_bytes());
        out[50..54].copy_from_slice(&self.scale_permyriad.to_le_bytes());
        out[54] = self.sentinel_boundary;
        out
    }

    pub fn from_bytes(raw: &[u8]) -> Result<Self, SealError> {
        if raw.len() < 64 {
            return Err(SealError::TooShort);
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&raw[0..4]);
        if magic != SEAL_MAGIC {
            return Err(SealError::BadMagic);
        }
        let version = u16::from_le_bytes([raw[4], raw[5]]);
        if version != 1 {
            return Err(SealError::BadVersion);
        }
        let mut merkle_root = [0u8; 32];
        merkle_root.copy_from_slice(&raw[16..48]);
        Ok(Self {
            magic,
            version,
            flags: u16::from_le_bytes([raw[6], raw[7]]),
            rows: u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]),
            cols: u32::from_le_bytes([raw[12], raw[13], raw[14], raw[15]]),
            merkle_root,
            leaf_size_bytes: u16::from_le_bytes([raw[48], raw[49]]),
            scale_permyriad: i32::from_le_bytes([raw[50], raw[51], raw[52], raw[53]]),
            sentinel_boundary: raw[54],
            _reserved: [0u8; 11],
        })
    }
}

/// SHA-256 Merkle root over fixed 64-byte leaves, zero heap: binary-counter
/// fold with a fixed `[level; MAX_DEPTH]` stack. Last leaf zero-padded; a
/// lone node at any level is promoted unhashed (Bitcoin-style would double
/// it — promotion is chosen so a single-leaf root equals that leaf's hash).
/// Empty payload folds to the all-zero root, matching `AuditChain` genesis.
pub fn merkle_root(payload: &[u8]) -> [u8; 32] {
    if payload.is_empty() {
        return [0u8; 32];
    }

    let mut stack = [[0u8; 32]; MAX_DEPTH];
    let mut filled = [false; MAX_DEPTH];

    for chunk in payload.chunks(LEAF_BYTES) {
        let mut leaf = [0u8; LEAF_BYTES];
        leaf[..chunk.len()].copy_from_slice(chunk);
        let mut node: [u8; 32] = Sha256::digest(leaf).into();

        let mut level = 0;
        while level < MAX_DEPTH && filled[level] {
            let mut h = Sha256::new();
            h.update(stack[level]);
            h.update(node);
            node = h.finalize().into();
            filled[level] = false;
            level += 1;
        }
        if level < MAX_DEPTH {
            stack[level] = node;
            filled[level] = true;
        }
    }

    let mut acc: Option<[u8; 32]> = None;
    for level in 0..MAX_DEPTH {
        if !filled[level] {
            continue;
        }
        acc = Some(match acc {
            None => stack[level],
            Some(right) => {
                let mut h = Sha256::new();
                h.update(stack[level]);
                h.update(right);
                h.finalize().into()
            }
        });
    }
    acc.unwrap_or([0u8; 32])
}

/// Which side of a combine step the sibling hash sits on, relative to the
/// leaf's running ancestor value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct Sibling {
    pub hash: [u8; 32],
    pub side: Side,
}

/// One leaf's O(log N) ancestor path to the root — 08_THE_MERKLE_MORIN_
/// ARCHITECTURE.md §3.1's leaf-path audit, net-new (source header/fold had
/// no proof half; drain confirmed absent repo- and quarry-wide).
#[derive(Debug, Clone, Copy)]
pub struct LeafProof {
    pub siblings: [Sibling; MAX_DEPTH],
    pub len: usize,
}

/// Build `leaf_index`'s proof against `payload`'s [`merkle_root`]. Replays
/// the identical binary-carry fold, tracking the one leaf's running
/// ancestor value through every combine it participates in, in both the
/// per-leaf carry loop and the final peak-bagging pass. `None` if
/// `leaf_index` is out of range.
pub fn prove_leaf(payload: &[u8], leaf_index: usize) -> Option<LeafProof> {
    let num_leaves = payload.chunks(LEAF_BYTES).count();
    if leaf_index >= num_leaves {
        return None;
    }

    let mut stack = [[0u8; 32]; MAX_DEPTH];
    let mut filled = [false; MAX_DEPTH];
    let mut proof = LeafProof { siblings: [Sibling { hash: [0u8; 32], side: Side::Left }; MAX_DEPTH], len: 0 };
    let mut parked_at: Option<usize> = None;
    let mut active = false;

    for (idx, chunk) in payload.chunks(LEAF_BYTES).enumerate() {
        let mut leaf = [0u8; LEAF_BYTES];
        leaf[..chunk.len()].copy_from_slice(chunk);
        let mut node: [u8; 32] = Sha256::digest(leaf).into();
        if idx == leaf_index {
            active = true;
        }

        let mut level = 0;
        while level < MAX_DEPTH && filled[level] {
            if parked_at == Some(level) {
                proof.siblings[proof.len] = Sibling { hash: node, side: Side::Right };
                proof.len += 1;
                active = true;
                parked_at = None;
            } else if active {
                proof.siblings[proof.len] = Sibling { hash: stack[level], side: Side::Left };
                proof.len += 1;
            }
            let mut h = Sha256::new();
            h.update(stack[level]);
            h.update(node);
            node = h.finalize().into();
            filled[level] = false;
            level += 1;
        }
        if level < MAX_DEPTH {
            stack[level] = node;
            filled[level] = true;
            if active {
                parked_at = Some(level);
                active = false;
            }
        }
    }

    let mut acc: Option<[u8; 32]> = None;
    for level in 0..MAX_DEPTH {
        if !filled[level] {
            continue;
        }
        acc = Some(match acc {
            None => {
                if parked_at == Some(level) {
                    active = true;
                }
                stack[level]
            }
            Some(right) => {
                if parked_at == Some(level) {
                    proof.siblings[proof.len] = Sibling { hash: right, side: Side::Right };
                    proof.len += 1;
                    active = true;
                    parked_at = None;
                } else if active {
                    proof.siblings[proof.len] = Sibling { hash: stack[level], side: Side::Left };
                    proof.len += 1;
                }
                let mut h = Sha256::new();
                h.update(stack[level]);
                h.update(right);
                h.finalize().into()
            }
        });
    }

    Some(proof)
}

/// Recompute `leaf`'s ancestor path through `proof` and compare to `root`
/// — O(log N) sequential SHA-256 hashes, no payload replay.
pub fn verify_leaf(leaf: &[u8], proof: &LeafProof, root: [u8; 32]) -> bool {
    let mut padded = [0u8; LEAF_BYTES];
    let n = leaf.len().min(LEAF_BYTES);
    padded[..n].copy_from_slice(&leaf[..n]);
    let mut node: [u8; 32] = Sha256::digest(padded).into();

    for sib in &proof.siblings[..proof.len] {
        let mut h = Sha256::new();
        match sib.side {
            Side::Left => {
                h.update(sib.hash);
                h.update(node);
            }
            Side::Right => {
                h.update(node);
                h.update(sib.hash);
            }
        }
        node = h.finalize().into();
    }
    node == root
}

/// Zero-copy verified container: header parse, bounds check, then a full
/// root recomputation over the payload — refuse on any mismatch.
#[derive(Debug, Clone)]
pub struct SealedPayload<'a> {
    pub header: MerkleMorinHeader,
    pub payload: &'a [u8],
}

impl<'a> SealedPayload<'a> {
    /// Seal `payload`: header carrying its root, `rows`=payload length,
    /// `cols`=leaf count, both informational for byte payloads.
    pub fn header_for(payload: &[u8], scale_permyriad: i32) -> MerkleMorinHeader {
        let leaves = payload.len().div_ceil(LEAF_BYTES) as u32;
        MerkleMorinHeader::new(payload.len() as u32, leaves, merkle_root(payload), scale_permyriad)
    }

    /// Open a `[64-byte header][payload]` buffer, verifying magic, bounds,
    /// and the recomputed root before any byte is trusted.
    pub fn open(raw: &'a [u8]) -> Result<Self, SealError> {
        let header = MerkleMorinHeader::from_bytes(raw)?;
        let payload = &raw[64..];
        if payload.len() != header.rows as usize {
            return Err(SealError::PayloadTruncated);
        }
        if merkle_root(payload) != header.merkle_root {
            return Err(SealError::RootMismatch);
        }
        Ok(Self { header, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_TEST_LEAVES: usize = 17;

    fn test_payload() -> [u8; MAX_TEST_LEAVES * LEAF_BYTES] {
        let mut buf = [0u8; MAX_TEST_LEAVES * LEAF_BYTES];
        for (i, b) in buf.iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }
        buf
    }

    #[test]
    fn every_leaf_proves_against_the_independently_computed_root() {
        // Dual-oracle: merkle_root() is the first oracle (root), verify_leaf
        // via prove_leaf is the second, independent path. Sweeps leaf counts
        // across several binary-carry boundaries (1, 2, 3, 4, 5, 7, 8, 9, 16,
        // 17 leaves) so every carry/park/final-bagging code path fires.
        let buf = test_payload();
        for num_leaves in [1usize, 2, 3, 4, 5, 7, 8, 9, 16, 17] {
            let payload = &buf[..num_leaves * LEAF_BYTES];
            let root = merkle_root(payload);
            for leaf_index in 0..num_leaves {
                let proof = prove_leaf(payload, leaf_index).expect("leaf in range must prove");
                let leaf_bytes = &payload[leaf_index * LEAF_BYTES..(leaf_index + 1) * LEAF_BYTES];
                assert!(
                    verify_leaf(leaf_bytes, &proof, root),
                    "leaf {leaf_index} of {num_leaves} must verify against the independently computed root"
                );
            }
        }
    }

    #[test]
    fn a_tampered_leaf_fails_verification() {
        let buf = test_payload();
        let payload = &buf[..9 * LEAF_BYTES];
        let root = merkle_root(payload);
        let proof = prove_leaf(payload, 4).unwrap();
        let mut tampered = [0u8; LEAF_BYTES];
        tampered.copy_from_slice(&payload[4 * LEAF_BYTES..5 * LEAF_BYTES]);
        tampered[0] ^= 0x01;
        assert!(!verify_leaf(&tampered, &proof, root));
    }

    #[test]
    fn a_proof_from_the_wrong_leaf_index_fails_verification() {
        let buf = test_payload();
        let payload = &buf[..9 * LEAF_BYTES];
        let root = merkle_root(payload);
        let proof_for_4 = prove_leaf(payload, 4).unwrap();
        let leaf_5 = &payload[5 * LEAF_BYTES..6 * LEAF_BYTES];
        assert!(!verify_leaf(leaf_5, &proof_for_4, root));
    }

    #[test]
    fn out_of_range_leaf_index_refuses_to_prove() {
        let payload = [7u8; 3 * LEAF_BYTES];
        assert!(prove_leaf(&payload, 3).is_none());
        assert!(prove_leaf(&payload, 999).is_none());
    }

    #[test]
    fn single_leaf_proof_is_empty_and_the_leaf_hash_is_the_root() {
        let payload = [9u8; LEAF_BYTES];
        let root = merkle_root(&payload);
        let proof = prove_leaf(&payload, 0).unwrap();
        assert_eq!(proof.len, 0);
        assert!(verify_leaf(&payload, &proof, root));
    }

    #[test]
    fn header_round_trips_through_bytes() {
        let h = MerkleMorinHeader::new(320, 5, [0xAB; 32], 10_000);
        let back = MerkleMorinHeader::from_bytes(&h.to_bytes()).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn bad_magic_is_refused() {
        let mut raw = MerkleMorinHeader::new(1, 1, [0; 32], 0).to_bytes();
        raw[0] = b'X';
        assert_eq!(MerkleMorinHeader::from_bytes(&raw), Err(SealError::BadMagic));
    }

    #[test]
    fn short_buffer_is_refused() {
        assert_eq!(MerkleMorinHeader::from_bytes(&[0u8; 63]), Err(SealError::TooShort));
    }

    #[test]
    fn single_leaf_root_is_the_padded_leaf_hash() {
        let payload = b"one leaf";
        let mut leaf = [0u8; LEAF_BYTES];
        leaf[..payload.len()].copy_from_slice(payload);
        let want: [u8; 32] = Sha256::digest(leaf).into();
        assert_eq!(merkle_root(payload), want);
    }

    #[test]
    fn root_is_deterministic_and_content_sensitive() {
        let a = merkle_root(b"the proof ledger, day 20698");
        let b = merkle_root(b"the proof ledger, day 20698");
        let c = merkle_root(b"the proof ledger, day 20699");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn a_multi_leaf_root_differs_from_any_single_leaf_hash() {
        let payload = [7u8; LEAF_BYTES * 3 + 5];
        let root = merkle_root(&payload);
        for chunk in payload.chunks(LEAF_BYTES) {
            let mut leaf = [0u8; LEAF_BYTES];
            leaf[..chunk.len()].copy_from_slice(chunk);
            let h: [u8; 32] = Sha256::digest(leaf).into();
            assert_ne!(root, h);
        }
    }

    #[test]
    fn empty_payload_is_the_genesis_root() {
        assert_eq!(merkle_root(b""), [0u8; 32]);
    }

    #[test]
    fn sealed_payload_opens_clean_and_refuses_tamper() {
        let payload = b"tick 42: purity=46 condor=375.50";
        let header = SealedPayload::header_for(payload, 10_000);

        let mut buf = [0u8; 64 + 32];
        buf[..64].copy_from_slice(&header.to_bytes());
        buf[64..].copy_from_slice(payload);

        let opened = SealedPayload::open(&buf).expect("clean seal opens");
        assert_eq!(opened.payload, payload);

        // Flip one payload byte: the root recomputation must refuse it.
        buf[64] ^= 0x01;
        assert_eq!(SealedPayload::open(&buf).unwrap_err(), SealError::RootMismatch);
    }

    #[test]
    fn truncated_payload_is_refused_before_root_check() {
        let payload = b"short";
        let header = SealedPayload::header_for(payload, 0);
        let mut buf = [0u8; 64 + 3];
        buf[..64].copy_from_slice(&header.to_bytes());
        buf[64..].copy_from_slice(&payload[..3]);
        assert_eq!(SealedPayload::open(&buf).unwrap_err(), SealError::PayloadTruncated);
    }
}
