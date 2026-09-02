# Sehrish — Operational Brief

Working agreements for this repo, both directions. (Urdu version: ask your Claude.)

## Lanes

- **Sehrish**: front end, demo UI, README, docs/, presentation assets. Full ownership, no review gate from our side.
- **Sean's sessions**: `crates/` (trading engine), live orders, ledger.
- `crates/` is single-writer to avoid another collision like last night — engine change requests go through Sean's session, credited like the butterfly cap (your 20a1ea8, now in `strategy.rs`).
- `.claude/workflows/777-cascade-uiux.js` is staged for the demo UI: "run the 777-cascade-uiux workflow" in a repo session outputs a build spec with every screen element tied to its data source.

## Last night, for the record

Three commits landed under your account at 10:32 (your README plus two engine changes — likely your assistant acting on CLAUDE.md's pending-work list). The sign-convention change asserted "VERIFIED" without a source and had it backwards (Alpaca mleg: credit = negative `limit_price`; three cited sources in `dispatch.rs` and the CLAUDE.md ledger). Reversed in `7776b8c`. README adopted, butterfly cap ported and credited. The repo CLAUDE.md now blocks assistant sessions from resolving pending-list items — that hole was ours, not yours.

## Repo law (applies to every session, Sean's included)

- "VERIFIED" only ever appears next to a cited receipt (URL quoted, or file:line read that session). Otherwise write "unverified."
- Pull before working; push small and often — the schedule assumes either side can drop offline for hours without costing the other anything.
- Nothing on the critical path waits on anyone's clock. Async by default.

## Clocks

- Deadline: **Thu 8:00 PM PKT** (Sep 4, 9:00 AM MDT).
- Market hours: 6:30 PM – 1:00 AM PKT. Live trading runs from Sean's side; watching is optional.
