# AlpacaCOMP Technical Review

Date: 2026-09-03

## Verdict

AlpacaCOMP is a strong hackathon prototype solving a real problem: an AI trading model may suggest a trade, but deterministic code retains execution authority. Its best differentiator is the refusal path, not a claim of superior market prediction.

It is not presented as production brokerage software. The demonstrated environment is Alpaca paper trading.

## Current Code-Backed Path

`dispatch_spread` checks these conditions in order before broker submission:

1. legal position-state transition
2. authorized oracle verdict
3. accepted market-stability band
4. finite spread geometry
5. maximum loss within the 2 percent ceiling
6. structured Alpaca multi-leg submission through standard input

The first five items are checks. The sixth is the destination reached only after all checks pass.

## Strongest Receipts

| Behavior | Evidence |
|---|---|
| Illegal position transition refuses first | `LIVE_ORDER_DAG.validate_path` and `open_on_top_of_open_is_refused_before_every_other_gate` |
| Oversized spread is refused | `tonights_simmed_condor_trips_the_veto_on_100k` |
| Credit price uses the Alpaca sign convention | `credit_entry_limit_price_is_negative_per_alpaca_mleg_convention` |
| Broker body is structured multi-leg JSON | `mleg_body` |
| Broker body is sent through standard input | `run_with_stdin(..., "@-")` |
| Strategy uses quoted option strikes and caps wings | `strategy.rs` builder tests |

## Professional Assessment

### Strong

- Generative suggestions are separated from broker authority.
- Refusals are typed and testable.
- The 2 percent loss ceiling is concrete and understandable.
- The strategy refuses missing market quotes instead of inventing legs.
- The gate core is a `no_std`, unsafe-denied Rust crate.

### Still Hackathon-Grade

- The public demo is a static evidence replay, not a live trading dashboard.
- Support modules such as the Merkle seal, Fredholm residue, and API pacer should not be described as live gates without a current path receipt.
- Production readiness would require broader broker-error recovery, operational monitoring, security review, and long-running evidence.
- Performance, P&L, latency, and test-count claims need fresh reproducible receipts before publication.

## Recommendation

Do not add another engine feature before submission from the collaborator lane. Publish the proof portal, record the refusal flow, and keep the README, slides, and video synchronized with `docs/CLAIM_PROOF_MAP.md`.
