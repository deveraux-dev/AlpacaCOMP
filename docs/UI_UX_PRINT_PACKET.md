# 13forge UI/UX Demo Packet

Print target: one judge-facing proof portal, one README link, one demo spine.

## Selected Option

Use option 2 as the canonical demo:

https://13forge-proof-portal.vercel.app/demo-portal/

Why this option wins for the hackathon:

- It opens directly on the usable proof replay instead of a marketing page.
- It explains the product in one action: an AI-proposed options trade is checked by deterministic Rust gates before Alpaca can receive it.
- It keeps claims tied to receipts and avoids adding unverified live-performance language.
- It is static and demo-safe: no credentials, no live balances, no broker call.

## Screen Flow

1. Intro: "Would this AI trade reach Alpaca?"
2. Story strip: AI suggests, Rust checks, market data structures, Alpaca receives only safe orders.
3. Replay panel: run the safety checks on the oversized iron condor.
4. Verdict: `DispatchRefusal::MaxLossVeto`, broker not reached.
5. Evidence: $2,525 max loss is above the $2,000 hard ceiling.
6. Architecture: strategist, Rust safety gates, Alpaca.
7. Claim board: live, verified, tested, and support claims are separated.

## Print Checklist

- Print the live portal from the browser in light mode.
- Include the README page with the canonical proof portal link.
- Include `docs/CLAIM_PROOF_MAP.md` when judges ask for receipt mapping.
- Do not print or present option 1 or option 3 as separate demos.
- Do not claim win rate, profit factor, or latency as verified unless a fresh receipt is added.

## UX Pattern

The interface uses a proof-first dashboard pattern:

- Primary action is the replay button.
- The refusal result is visible without requiring background-code explanation.
- Claims are grouped by confidence level.
- Risk math is shown as a compact equation.
- Secondary details stay below the replay so the first viewport stays demo-ready.

