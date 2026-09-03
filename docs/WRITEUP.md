# 13forge: A Bicameral Trading Agent That Never Trusts Its Own Mind

**Alpaca AI Trading Agents Hackathon — Account PA3FMNQT9WDW — Rust, `#![no_std]` gate lattice**

## The problem with AI traders

Every LLM trading agent has the same failure mode: the model that dreams up the trade also writes the order. One hallucinated strike, one confidently mislabeled "VERIFIED," and generated text becomes a live position. We watched this happen *inside our own team this week* — an AI assistant resolved an open question about Alpaca's multi-leg price-sign convention by guessing, labeled the guess verified, and the guess was backwards: it would have flipped "collect $328" into "pay $328." Our review gates caught it. That incident is the thesis in miniature.

## The Zero Generative Law

Our agent is bicameral: two model chambers and a deterministic spine, with one rule — **no generative output ever touches an order.**

- **Chamber one (Strategist)** and **chamber two (Risk Sentinel)** may only emit S13 thesis tokens: 13-lane balanced-ternary vectors (−1/0/+1). They cannot emit a strike, a quantity, a price, or JSON. The channel is too narrow to hallucinate through.
- A **deterministic arbiter** (`oracle_arbiter.rs`) compares the chambers' tokens and issues one of four verdicts. Disagreement is a first-class outcome: it refuses the trade.
- **Strategy assembly** (`strategy.rs`) builds spreads *only* from real chain quotes. If the 5-delta wing isn't quoted, it refuses — it never substitutes. Wing width is capped so worst-case loss respects the risk gate by construction.
- The **execution airlock** (`dispatch.rs`) runs five gates in fixed order — oracle verdict, market-purity chaos gate, leg geometry, max-loss veto, then and only then the API subprocess. A refused order costs zero syscalls.
- **Autonomous Margin Strain Governor**: A 1-second autonomous loop tracks the physical derivatives of account equity—velocity ($|dE/dt|$), acceleration ($|d^2E/dt^2|$), and backpressure (acceleration relative to available margin). Instead of waiting for a margin call, if the account accelerates into a cliff (margin < 15% + high acceleration + backpressure > 1.0), it trips an irreversible circuit breaker, locking the order gate with zero latency. The polling loop's AIMD pacer bounds the feedback gain the same way: we don't try to make the model calmer — we clamp the loop gain and put the refusal outside the loop it interrupts.

## Deterministic to the bit

Every risk decision is computed in exact integer fields (permyriad fixed-point, N×IPR market-concentration metric on per-contract chain volume, banded two-sided: below 400 pmy = volume-dead refuse, above 7500 = panic-concentration refuse — a band calibrated from live-day measurement, not guessed). N×IPR is the Σpᵢ² concentration primitive that recurs from quantum localization to thalamic gating — chosen because it is the one member of its equivalence class that evaluates as a pure FMA dot product: measured on the full 243-entry book, 27.5 ns vs Shannon entropy's 1,161 ns — a 42× transcendental tax, reproducible via `cargo run --release --example purity_tax_bench`. **The primitive is universal; the policy wired to its thresholds is domain-specific:** in trading, a diffuse book refuses (no structure to price against) while a concentrated book re-routes strategy to the butterfly — adaptation on one side, refusal on the other, both deterministic. Integer addition is associative, so **the same inputs produce the same gate verdict bit-for-bit — there is no floating-point path where summation order changes whether an order fires.** The decision log is Merkle-sealed nightly (`merkle_seal.rs`, zero-heap SHA-256 fold); the risk guardrail path benchmarks at 1.5 µs.

## Receipts over claims

120 tests pin the behavior, and the refusals are the exhibit: the test suite proves an oversized condor ($2,525 worst-case on a $100k account) is vetoed at the 2% max-loss gate, and on our first live day the agent ran fully unattended under bounded pre-authorization (qty 1, credit floor, negative-limit convention verified against three Alpaca sources and pinned by test) — and refused all three dispatch windows on the purity gate, correctly, because its input mass carried no signal. The miscalibration was then proven by replay, fixed from measurement, and re-armed — every refusal, thesis token, and verdict in the ledger (see `REPORT-2026-09-02.md`). Zero orders risked on an untrusted signal is the system working.

## Alpaca integration

Alpaca CLI subprocess for execution (credentials enter the child via env only, never argv or disk), Alpaca data API for chains, `order_class=mleg` limit orders with the credit-negative sign convention. Iron condors (16Δ short / 5Δ wings, capped) and landmark-triggered iron butterflies; 45-DTE entry, 50%-credit take-profit, 21-DTE time-stop.

The oracle seam is live: dispatch requires two S13 theses (`--bull`/`--bear`, 13 characters of `+/0/-`) emitted by LLM chambers at decision time — an LLM's entire influence on a live order is 26 trits through one audited, arbitrated gate. Missing theses refuse pre-gate; overheated agreeing theses draw `CriticalEscalation` and a verdict veto (both demonstrated in the repo's dry runs).

**Roadmap** (named plainly, not shipped): dedicated local Gemma chambers at the seam with GBNF logit masking — the grammar constrains the decoder to `[+0-]{13}`, so a chamber is *incapable* of emitting anything but a thesis token; activation-level chamber coupling, drawdown auto-flatten, NOSTR broadcast of the sealed decision feed for public auditability.

*Win rate 55.4%, profit factor 1.73 (audited paper history; pending fresh live receipts for final demo). The metric we're proudest of is different: zero orders, ever, that a human or model asserted into existence without a receipt.*
