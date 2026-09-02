<div align="center">

  <h1>13forge</h1>
  <p><i>AI proposes. Deterministic gates decide.</i></p>

  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
  [![Rust](https://img.shields.io/badge/Rust-no__std-orange.svg)]()
</div>

--- 

## Overview

13forge is an autonomous options-trading agent built in Rust. Its critical `forge-gate` crate is `no_std` and denies unsafe code, while the order path applies deterministic checks before broker submission.

Unlike conventional AI bots that rely on unpredictable LLM outputs, 13forge enforces a rigorous architectural philosophy to guarantee execution safety and bounded risk: the model is never trusted with an order.

## Guided Proof Portal

The judge-facing portal turns the core safety story into one short interaction: replay an oversized iron-condor proposal, watch each deterministic check run, and see the order refused because `$2,525` maximum loss exceeds the `$2,000` account limit.

- [Open the proof portal](./demo-portal/index.html)
- [Read the complete claim-to-proof map](./docs/CLAIM_PROOF_MAP.md)
- [Follow the 90-second demo plan](./docs/VIDEO_DEMO_PLAN.md)

The portal is intentionally static. It contains no Alpaca credentials, invented balances, or unverified performance metrics, and the replay never submits an order.

## 📊 The Claim-Proof Ledger

Our core thesis for this hackathon: *no claim is shown as verified unless it has a receipt.* Below is the definitive mapping of how our "Zero-Claims Doctrine" fulfills the judging criteria.

### Live Execution Claims
| Claim | Status | Proof Source | Judge Angle |
|-------|--------|--------------|-------------|
| **Oracle verdict can veto an order.** | **LIVE** | `DispatchRefusal::VerdictVeto` in `dispatch_spread` | **Pawel, Tony** (Arbiter verdict blocks execution) |
| **Market purity/chaos can veto an order.** | **LIVE** | `purity.is_chaotic()` -> `DispatchRefusal::ChaoticBook` | **Tony, Pawel** (Chaotic market structure stops execution) |
| **2% max-loss veto blocks unsafe spreads.** | **LIVE** | `exceeds_max_loss_veto` test | **Tony, Chiranjeev** (Margin safety, 0 syscalls) |
| **Alpaca mleg credit price is negative.** | **VERIFIED** | `mleg_body` test, negative limit price pinning | **Brandon** (API correctness) |
| **Order JSON is sent through stdin.** | **LIVE** | `cli.run_with_stdin` in `dispatch_spread` | **Brandon** (Payload leakage prevention) |

### Strategy Claims
| Claim | Status | Proof Source | Judge Angle |
|-------|--------|--------------|-------------|
| **Strategy builds from real chain quotes.** | **TESTED** | `build_iron_condor` / `build_iron_butterfly` | **Pawel, Tony** (No model-invented strikes) |
| **Wing cap pulls over-wide wings inward.** | **TESTED** | `max_wing_width` parameter | **Tony** (Keeps trades inside risk ceiling) |

### Support Components
| Claim | Status | Proof Source | Judge Angle |
|-------|--------|--------------|-------------|
| **Order-state DAG.** | **TESTED** | `order_dag.rs` | **Tony, Pawel** (Built/tested state machine; not live yet) |
| **`forge-gate` is `no_std`.** | **LIVE** | `#![no_std]` in `crates/forge-gate/src/lib.rs` | **Brandon** (Bare-metal constraints) |
| **Merkle seal / evidence chain.** | **PROVEN** | `.forge/proof-ledger.tsv` | **Chiranjeev** (Unassailable receipts) |

> **Demo Spine:** A model can suggest a trade, but 13forge only submits it after deterministic gates approve the oracle verdict, market purity, leg geometry, and max-loss ceiling; the strongest proof is the unsafe condor that was refused before the Alpaca CLI ever spawned.

## 🧠 The Zero Generative Law

We do not allow predictive models to hallucinate strikes, Greeks, or JSON payloads directly. Our architecture enforces a strict separation of concerns:

1. **Emission:** The dual-oracle (Bull/Bear) emits constrained `S13` thesis tokens.
2. **Gating:** Tokens hit a deterministic, refuse-by-default gate lattice.
3. **Assembly:** The strategy layer builds the trade exclusively from real `ChainQuote` market data.
4. **Execution:** Only mathematically verified, strictly bounded trades reach the Alpaca V2 API.

```mermaid
graph TD
    A[Dual-Oracle <br> Bull/Bear] -->|Emits| B(S13 Thesis Tokens)
    B --> C{Deterministic <br> Gate Lattice}
    C -->|Refuse| D[Execution Aborted <br> 0 Syscalls]
    C -->|Pass| E[Strategy Assembly]
    E -->|Reads| F[(ChainQuote <br> Market Data)]
    E --> G[Alpaca API]

    style A fill:#111,stroke:#333,stroke-width:2px,color:#fff
    style B fill:#222,stroke:#444,color:#fff
    style C fill:#d63031,stroke:#ff7675,stroke-width:2px,color:#fff
    style D fill:#636e72,color:#fff
    style E fill:#00b894,stroke:#55efc4,stroke-width:2px,color:#fff
    style F fill:#e17055,color:#fff
    style G fill:#fbc531,color:#111,stroke-width:2px
```

## ⚙️ Setup & Verification Guide

Ensure you have Rust installed and your Alpaca paper credentials set in your environment. **We strictly avoid writing secrets to disk.**

```bash
# 1. Set your Alpaca V2 paper credentials (session-only)
export APCA_API_KEY_ID="your_key_id"
export APCA_API_SECRET_KEY="your_secret_key"

# 2. Run the current gate-lattice test suites
cargo test -p forge-gate -p forge-daemon

# 3. Dry-run today's strategy selection against the live SPY chain (no orders placed)
cargo run --example sim_today -p forge-daemon

# 4. Live account smoke test (GET /v2/account only)
cargo run --example live_smoke -p forge-daemon
```

## ✅ Hackathon Submission Checklist

- [x] **Account Verification**: Confirm fresh Alpaca paper account balance is exactly `$100,000`. (Status: `ACTIVE`, Buying Power: `$400,000`)
- [x] **Judging ID**: Retrieve new Alpaca Account ID for official P&L judging. (Account ID: `PA3FMNQT9WDW`)
- [ ] **Technical Repository**: Publish public GitHub repository and demo URL.
- [ ] **Write-Up**: Finalize the 1-page write-up around the deterministic gate path and Alpaca CLI infrastructure.
- [ ] **Presentation Assets**: Compile video presentation, slide deck, and cover image.
- [ ] **Build-in-Public**: Publish Build-in-Public posts on X/LinkedIn tagging `@lablabai` and `@AlpacaHQ`.

---
<div align="center">
  <b>13forge</b>
</div>
