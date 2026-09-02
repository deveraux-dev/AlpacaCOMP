# Video and Demo Plan

Goal: make judges understand the project in under 90 seconds.

## Core Message

AlpacaCOMP is an AI trading agent where the AI is not trusted with the final order. It can produce a market thesis, but deterministic Rust gates decide whether the trade is safe enough to reach Alpaca paper trading.

## Suggested 90-Second Structure

### 0-10 seconds: Hook

"Most AI trading demos focus on what the model wants to buy. AlpacaCOMP focuses on what the model is not allowed to do."

Show: the architecture blueprint or a simple flow:

```text
AI thesis -> Rust gate -> Alpaca paper order/refusal -> receipt
```

### 10-30 seconds: Problem

Explain that AI agents can hallucinate, over-risk, or send invalid orders. In trading, that is not acceptable. The system needs a hard safety boundary between model output and broker execution.

### 30-55 seconds: Solution

Show the gate stack:

```text
Oracle verdict
Market purity
Leg geometry
2% max-loss veto
Alpaca multi-leg order
Receipt
```

Say: "Every order clears these gates in-process before the Alpaca subprocess exists."

### 55-75 seconds: Demo

Use the strongest proof:

```text
Simulated Condor:
29-wide put wing
$3.75 credit
$100,000 paper account
Max loss: $2,525
Allowed max loss: $2,000
Result: refused before dispatch
```

This is better than only showing a successful order because it proves the safety layer has teeth.

### 75-90 seconds: Close

"The agent is autonomous, but not unconstrained. The model can suggest. The deterministic gate decides. Alpaca only sees trades that pass."

End with:

- paper account
- test count
- proof ledger / sealed receipt
- GitHub repo link

## Shots to Capture

1. Repository overview with `README.md`.
2. Architecture blueprint: `docs/patex_alpaca.png`.
3. Test run: `cargo test --workspace`.
4. Dry-run simulation: `cargo run -p forge-daemon --example sim_today`.
5. Proof seal: `cargo run -p forge-daemon --example seal_now`.
6. Optional read-only Alpaca account/clock smoke test if credentials are available.

## What Not To Over-Explain

- Fredholm math derivation
- S13 internals
- Every regime class
- Long benchmark tables

Mention those only as engineering depth. The main story should stay simple: untrusted AI, deterministic gate, safe Alpaca execution, receipts.

## Open Questions For Sean

Ask these before changing trading code:

1. Is `AlpacaCOMP` the final hackathon submission repo, and should this README be the public-facing one?
2. Should we add strategy-side wing-width capping before the first live submit?
3. Has the Alpaca `mleg` credit-spread `limit_price` sign been verified against the docs or a paper order?
4. Should the video lead with a refused trade, a successful paper order, or both?
5. What exact account metrics should be shown in the final submission: balance, P&L, win rate, profit factor, or all of them?
