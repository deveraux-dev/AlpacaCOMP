//! Seal the proof ledger + latest chain snapshot: print each Merkle-Morin
//! root and the combined session root. Read-only; the loop logs the output.

use forge_gate::merkle_seal::{merkle_root, SealedPayload};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() {
    let ledger = std::fs::read(r".forge\proof-ledger.tsv").expect("ledger readable");
    let chain = std::fs::read(r".forge\sim\spy_chain_20261016.json").expect("chain readable");

    let ledger_header = SealedPayload::header_for(&ledger, 10_000);
    let chain_header = SealedPayload::header_for(&chain, 10_000);

    let mut combined = Vec::with_capacity(64);
    combined.extend_from_slice(&ledger_header.merkle_root);
    combined.extend_from_slice(&chain_header.merkle_root);
    let session_root = merkle_root(&combined);

    println!("ledger  root: {} ({} bytes, {} leaves)", hex(&ledger_header.merkle_root), ledger_header.rows, ledger_header.cols);
    println!("chain   root: {} ({} bytes, {} leaves)", hex(&chain_header.merkle_root), chain_header.rows, chain_header.cols);
    println!("session root: {}", hex(&session_root));
}
