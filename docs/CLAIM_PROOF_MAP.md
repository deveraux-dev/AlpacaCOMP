# 13forge Claim Proof Map

Scope: collaborator lane for README, demo portal, video, and presentation wording.
`crates/` remains read-only from Sehrish sessions unless Sean explicitly asks for engine changes.

Doctrine: no claim is shown as verified unless it has a receipt. When the receipt is not present in this repo/session, use "needs receipt" or remove the claim from judge-facing copy.

## Live Execution Claims

| Claim | Status | Proof source | Judge angle | Safe wording |
|---|---:|---|---|---|
| Orders pass through one guarded dispatch door before Alpaca. | LIVE | `crates/forge-daemon/src/dispatch.rs::dispatch_spread` | Brandon, Chiranjeev | Order submission is centralized through one audited dispatch function before the CLI/API call. |
| Oracle verdict can veto an order. | LIVE | `dispatch_spread` returns `DispatchRefusal::VerdictVeto` for non-authorized verdicts | Pawel, Tony | The strategist is not trusted directly; an arbiter verdict can block execution. |
| Market purity/chaos can veto an order. | LIVE | `dispatch_spread` checks `purity.is_chaotic()` and returns `DispatchRefusal::ChaoticBook` | Tony, Pawel | Diffuse or chaotic market structure can stop execution before order submission. |
| Leg geometry is checked before execution. | LIVE | `max_wing_width`, finite wing check, `DispatchRefusal::MalformedLegs` | Tony, Brandon | Malformed spread geometry is refused before any broker call. |
| 2 percent max-loss veto blocks unsafe spreads. | LIVE | `exceeds_max_loss_veto`, test `tonights_simmed_condor_trips_the_veto_on_100k` | Tony, Chiranjeev | A 29-wide condor with $3.75 credit is refused because max loss is $2,525, above the $2,000 limit on a $100k account. |
| Alpaca mleg credit price convention is negative. | VERIFIED BY CODE RECEIPTS | `mleg_body` doc comment, test `credit_entry_limit_price_is_negative_per_alpaca_mleg_convention`, CLAUDE.md ledger notes | Brandon | Credit mleg limit prices serialize as negative values; debit prices serialize as positive values. |
| Order JSON is sent through stdin, not argv. | LIVE | `cli.run_with_stdin(... "--body", "@-")` in `dispatch_spread` | Brandon | The mleg JSON body is piped to the Alpaca CLI through stdin, avoiding payload leakage through argv. |

## Strategy Claims

| Claim | Status | Proof source | Judge angle | Safe wording |
|---|---:|---|---|---|
| Strategy builds Iron Condors from real chain quotes, not model-invented strikes. | TESTED | `build_iron_condor`, tests `builds_iron_condor_from_16_5_delta_wings`, `refuses_condor_on_empty_snapshot_never_guesses` | Pawel, Tony | The model may propose a thesis, but strike selection comes from supplied market-chain quotes only. |
| Strategy builds Iron Butterflies from real chain quotes. | TESTED | `build_iron_butterfly`, tests `builds_iron_butterfly_from_atm_body_and_wings`, `refuses_butterfly_when_wing_delta_missing_from_book` | Pawel, Tony | The butterfly builder refuses missing quote data instead of inventing legs. |
| Wing cap pulls over-wide wings inward to quoted legal strikes when possible. | TESTED | `max_wing_width` parameter, test `wing_cap_pulls_delta_selected_wing_inside_cap` | Tony, business value | Strategy tries to keep trades inside the risk ceiling by selecting the widest quoted wing inside the cap. |
| Wing cap refuses when no quoted legal strike exists. | TESTED | test `wing_cap_refuses_when_no_quoted_strike_fits` | Tony | If the market chain has no safe quoted wing, the strategy refuses instead of inventing one. |
| Butterfly cap idea credited to Sehrish. | CREDITED | `build_iron_butterfly` comment: "Cap-on-butterfly idea: partner's 20a1ea8." | Team contribution | Keep this as contribution evidence, not as a separate performance claim. |

## Support Components

| Claim | Status | Proof source | Judge angle | Safe wording |
|---|---:|---|---|---|
| Position-state DAG | PROVEN SUPPORT | `order_dag.rs` | Tony, Pawel | Built and tested state machine; do not claim it gates live orders unless wired. |
| `forge-gate` is `no_std` and denies unsafe code. | LIVE IN CORE CRATE | `crates/forge-gate/src/lib.rs` has `#![no_std]` and `#![deny(unsafe_code)]` | Brandon | The gate core is a `no_std`, unsafe-denied Rust crate. |
| Margin Strain Governor | LIVE IN BACKGROUND | `StrainScore` detector running in a 1s loop (see Sean's latest report) | Tony | An autonomous circuit breaker monitors the velocity and acceleration of equity decay, locking the order gate if backpressure exceeds safe limits. |
| Merkle seal / evidence chain exists. | PROVEN SUPPORT | `.forge/proof-ledger.tsv`; `crates/forge-gate/src/merkle_seal.rs`; CLAUDE.md says not live dispatch | Chiranjeev | The project includes an append-only proof ledger and seal tooling; do not claim it gates live dispatch unless wired. |
| Fredholm/residue logic exists. | PROVEN SUPPORT | `crates/forge-gate/src/residue.rs`; `.forge/proof-ledger.tsv` | Pawel | Fredholm/residue logic is a tested support/research component; do not claim it blocks live orders unless wired. |
| API pacer exists. | PROVEN SUPPORT | `crates/forge-gate/src/api_pacer.rs`; CLAUDE.md says not live dispatch | Brandon | API pacing is built as support logic; do not claim it controls the live order path unless wired. |

## Metrics Requiring Receipt Before Judge Copy

| Claim | Current status | Needed receipt |
|---|---:|---|
| 55.4 percent win rate | NEEDS RECEIPT | Ledger/backtest/paper-account export showing calculation method and sample period. |
| 1.73 profit factor | NEEDS RECEIPT | Calculation source from closed trades or backtest run. |
| 1.5 microsecond risk guardrail latency | NEEDS RECEIPT | Benchmark output, command used, machine/context, and latest commit. |
| 118 tests green | NEEDS FRESH SESSION RECEIPT | `cargo test -p forge-gate -p forge-daemon` output from latest branch. |

## Panel Mapping

| Judge concern | What to show | Receipt source |
|---|---|---|
| Margin/risk safety | 2 percent max-loss refusal and defined-risk spreads | `risk_router`, `dispatch.rs` refusal test |
| Trading API correctness | mleg JSON, negative credit price, stdin CLI submit | `dispatch.rs` mleg body and sign test |
| Developer workflow | static portal, simple commands, clear proof table | README/demo portal |
| Transparency/story | claim map, proof ledger, refusal receipt | this file, `.forge/proof-ledger.tsv` |
| Agent originality | strategist proposes, sentinel/gates verify | README architecture wording plus dispatch proof |

## Demo Spine

One sentence:

> A model can suggest a trade, but 13forge only submits it after deterministic gates approve the oracle verdict, market purity, leg geometry, and max-loss ceiling; the strongest proof is the unsafe condor that was refused before the Alpaca CLI ever spawned.

