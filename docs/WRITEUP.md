# 13forge: AI Proposes, Deterministic Gates Decide

**Alpaca AI Trading Agents Hackathon | Rust gate core | Alpaca paper trading**

## The Problem

An AI model can produce a useful market thesis while still proposing an invalid or over-risked order. In most agent demos, the same generative path that forms the idea also controls the broker request. In finance, that is too much authority for probabilistic output.

13forge separates those responsibilities. The model may suggest a trade, but fixed Rust code decides whether the order is allowed to reach Alpaca.

## The Governed Order Path

Every spread submission enters one dispatch function and follows the same order:

1. The position-state DAG refuses illegal transitions, including opening a spread on top of an open spread.
2. The oracle verdict must authorize execution.
3. The market-stability band must accept the current option-chain structure.
4. The option legs must form valid, finite spread geometry.
5. Maximum potential loss must remain within 2 percent of account balance.
6. Only then is the structured multi-leg request sent to the Alpaca CLI through standard input.

The strategy layer builds iron condors and iron butterflies from supplied option-chain quotes. It refuses missing legs rather than inventing strikes, and it pulls over-wide wings inward only when a quoted strike fits the configured cap.

## The Proof

The clearest test case is an iron condor with a 29-point put wing and a $3.75 credit on a $100,000 paper account:

```text
Maximum loss = (29 - 3.75) x 100 = $2,525
Allowed loss = 2% x $100,000 = $2,000
Result       = REFUSED BEFORE ALPACA
```

The dispatch test pins this result as `DispatchRefusal::MaxLossVeto`. A separate order-body test pins Alpaca's multi-leg pricing convention: a $3.28 net credit serializes as `-3.28`, while debit prices are positive.

## Technical Difference

The critical gate logic lives in a `no_std`, unsafe-denied Rust crate. The model does not write option symbols or broker JSON directly. Market data supplies the available contracts, deterministic code assembles and checks the spread, and one guarded dispatch path controls broker submission.

The repository also contains proof-ledger, Merkle-seal, Fredholm-residue, and API-pacing support components. These are presented as supporting engineering unless a current receipt shows that they control the demonstrated live path.

## Why It Matters

13forge is not claiming that AI can predict markets perfectly. It demonstrates a more defensible capability: an autonomous agent can be useful without being trusted blindly. The creative system proposes; the deterministic system controls whether capital may move.

Every judge-facing claim should map to code, a test, an Alpaca response, or a recorded ledger receipt. Performance and latency figures are included only when a fresh, reproducible receipt accompanies them.
