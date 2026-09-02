# AlpacaCOMP Technical Review

Date: 2026-09-02

## Executive Judgment

AlpacaCOMP is solving a real problem: how to put a deterministic safety boundary between an AI trading agent and broker execution.

The strong part is not that the system "uses AI to trade." Many projects can do that. The stronger professional angle is that the AI is constrained. It can contribute a bounded market thesis, but deterministic Rust gates own the final decision before an Alpaca order is built or submitted.

Current maturity: strong hackathon-grade core and proof artifacts, not yet a complete production trading system.

## Real Problem Being Solved

AI trading agents have three serious risks:

- hallucinated market data or invented strikes
- over-sized positions that look attractive but violate risk limits
- invalid or unsafe broker orders sent before checks happen

AlpacaCOMP addresses these with a refuse-before-dispatch architecture:

```text
AI/oracle thesis -> deterministic gate stack -> strategy legs -> Alpaca CLI -> receipt
```

This is a meaningful architecture. It is closer to a safety-critical control boundary than a generic trading bot.

## Verified / High-Confidence Claims

| Claim | Status | Evidence |
|---|---|---|
| Deterministic Rust gate core | Verified by source | `crates/forge-gate/src/lib.rs` uses `#![no_std]` and `#![deny(unsafe_code)]`. |
| Zero heap / no locks in gate core | High confidence from source scan | `forge-gate/src` did not show `Vec`, `String`, `Box`, `Arc`, `Mutex`, `std::`, or `alloc::` patterns. |
| Strategy layer exists | Verified by source | `strategy.rs` builds Iron Condors and Iron Butterflies from `ChainQuote`. |
| AI does not invent strikes/Greeks inside strategy | Verified by design | Strategy functions require caller-provided quote snapshots and return `None` if required legs are missing. |
| 2% max-loss veto exists | Verified by source | `risk_router.rs` implements `exceeds_max_loss_veto`. |
| Dispatch refuses before Alpaca subprocess spawn | Verified by tests/design | `dispatch.rs` tests use a nonexistent Alpaca executable to prove gate refusals happen before CLI execution. |
| 29-wide / $3.75 / $100k refusal case | Verified by test | `dispatch.rs` pins the exact max-loss case: `(29 - 3.75) * 100 = $2,525`, above $2,000. |
| Alpaca multi-leg JSON generation | Verified by source | `mleg_body` builds `order_class: "mleg"` with four legs and position intents. |
| Secrets not passed through argv | Verified by source | `alpaca_cli.rs` passes credentials through environment variables and order body through stdin. |
| Receipts/proof ledger exists | Verified by source | `.forge/proof-ledger.tsv`, `merkle_seal.rs`, and `seal_now.rs`. |
| Test coverage exists | Verified by scan | 112 Rust `#[test]` markers found in `crates/`. |

## Unverified / Incomplete Claims

| Claim | Status | Why It Matters |
|---|---|---|
| Alpaca `mleg limit_price` sign for credit spreads | Unverified | The proof ledger explicitly marks this as unverified. A wrong sign can turn a safe-looking order into bad execution behavior. |
| Strategy-side wing cap | Needed | Strategy can currently construct a spread that dispatch later rejects. Better architecture is to avoid producing invalid candidates earlier. |
| Order state DAG wired into dispatch | Incomplete | `order_dag.rs` exists and is tested, but `dispatch_spread` does not directly consume it. |
| Self-healing governor | Scaffolded, not complete | `governor.rs` exists, but comments say health atomics are not populated yet. |
| Fully autonomous live loop | Not proven by repo | There are dry-run and smoke-test examples, but not a fully wired unattended live trading loop in the inspected source. |
| Production trading readiness | Not claimed | This is hackathon/paper-trading software. It lacks the broader controls expected for production trading. |

## Professional Assessment

What is genuinely strong:

- The architecture separates model suggestion from execution authority.
- The gate core is small, deterministic, and test-heavy.
- Refusals are typed, not silent.
- The max-loss veto is concrete and easy to explain.
- The proof ledger culture is useful for judges because it gives receipts, not just claims.

What is still prototype-level:

- The live loop is not fully wired.
- Some claims live in comments/proof ledger rather than executable integration.
- The README/demo layer was missing before this contribution.
- The execution path needs final broker-specific verification around multi-leg pricing.
- The strategy should reduce invalid candidate generation instead of relying on dispatch as the first place that catches wing width.

## Best Next Technical Actions

### 1. Verify Alpaca `mleg limit_price` sign

This is the highest-risk blocker before a real paper submit.

Acceptance criteria:

- confirm the expected sign for multi-leg credit spreads using Alpaca documentation or a tiny paper-order experiment
- add a test or note that locks the convention
- update proof ledger/README from `UNVERIFIED` to verified only after evidence exists

### 2. Add strategy-side wing cap

The strategy should know the maximum allowed width:

```text
max_loss <= 2% of balance
(wing_width - credit) * 100 <= 0.02 * balance
wing_width <= (0.02 * balance / 100) + credit
```

For a $100,000 paper account and $3.75 credit:

```text
wing_width <= 20 + 3.75 = 23.75
```

So the 29-wide candidate should never become the preferred candidate.

### 3. Wire state DAG into dispatch or clearly scope it

Either:

- wire `OrderStateDag` into `dispatch_spread`, or
- document that `order_dag.rs` is currently a core module not yet connected to the daemon dispatch path

Avoid overclaiming it as active execution protection until it is in the path.

### 4. Keep governor wording honest

Use:

```text
governor scaffold / supervision layer
```

Avoid:

```text
fully self-healing live trading loop
```

unless the health telemetry is populated and demonstrated.

## Recommended Contribution Path

Immediate:

- push README/demo/technical review docs
- ask Sean to confirm final repo and preferred next code change

Next code PR:

- strategy-side wing-width cap, with tests

Do not do yet:

- broad refactor
- live order submission
- governor rewrite
- changes to Alpaca credentials or execution behavior without confirmation

## Bottom Line

This is not just a basic prototype. The deterministic gate architecture is a real differentiator for an AI trading agent.

But it is also not a finished production trading agent. It is a strong hackathon core with a few critical integration gaps. The best professional move is to present it honestly:

```text
Implemented: deterministic gate, strategy assembly, dispatch refusal, receipts.
Unverified: Alpaca mleg credit price sign.
Next: strategy wing cap, DAG wiring, live-loop hardening.
```
