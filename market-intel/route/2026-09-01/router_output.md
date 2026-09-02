# Market Pulse — 2026-09-01 (baseline)

## 1. Executive delta

- First run. No prior report exists -- baseline.
- All six pinned domains fetched via WebFetch (markdown-processed, not raw HTML -- see cache note below).
- Biggest finding: every competitor advertises backtesting; AlpacaCOMP's architect_reprime build log lists none. [ASSUMED gap -- not grep-verified against the repo this pass.]
- Second finding: no pinned competitor shows a deterministic, sub-microsecond, autonomous refuse-on-fault risk gate. This is AlpacaCOMP's strongest differentiator candidate.
- Third finding: Composer and QuantConnect's "Mia" let an LLM author strategy logic directly from natural language. AlpacaCOMP's Zero Generative Law (oracle emits S13 thesis tokens only, never strikes/Greeks) is a stricter, inverted posture worth stating explicitly as a trust argument.
- Cache note: WebFetch returns AI-summarized markdown, not raw HTML bytes -- the `cache/raw_html/.../sha256.html` pipeline described in target-policy.md was not executed this pass. Evidence pointers below are marked `unfetched_raw_html_not_cached_via_webfetch` rather than fabricated hashes.

## 2. Running ledger

LAST_REPORT_DATE: none (baseline)
DAYS_TO_DEADLINE: 3 (deadline 2026-09-04T09:00 MDT)
OPEN_HYPOTHESES: backtesting engine is a real gap; risk-gate latency is a real differentiator
VALIDATED_SIGNALS: iron_condor_automation and take_profit_time_stop are table-stakes AlpacaCOMP already meets
WEAKENED_SIGNALS: none yet (baseline)
BUILD_DECISIONS: none made this pass -- see section 5
WRITEUP_DECISIONS: lead with 1.5us risk gate + S13 dual-oracle contrast vs Mia/Composer AI-authorship
NEXT_REPORT_FOCUS: confirm backtesting-gap claim via repo grep; check Alpaca CLI/paper-account blocker status

## 3. Claim ledger highlights

| Domain | Claim type | Primitive | Confidence | Why it matters |
|---|---|---|---|---|
| alpaca.markets | options_workflow | iron_condor_automation | high | Own ecosystem supports multi-leg condors natively -- confirms CLI/API path is viable |
| alpaca.markets | latency | low_latency_telemetry | high | 1.5ms OMS order processing -- different layer than our 1.5us gate, do not conflate |
| composer.trade | strategy | volatility_triggered_strategy | high | LLM authors strategy logic directly -- opposite of our Zero Generative Law |
| composer.trade | automation | iron_condor_automation | high | Fully automated end-to-end brokerage -- direct autonomy comparable |
| optionalpha.com | autonomy | iron_condor_automation | high | Unattended bot execution -- closest comparable to our daemon-loop goal |
| optionalpha.com | risk_management | take_profit_time_stop | high | Same exit primitive we already ship -- parity confirmed |
| quantconnect.com | backtesting | backtesting_engine | high | 15,000+ backtests/day -- sharpest capability gap vs AlpacaCOMP |
| quantconnect.com | autonomy | dual_oracle_consensus | high | "Mia" agentic AI designs+live-trades -- comparable AI-agent-trades-capital framing |
| tastytrade.com | risk_management | deterministic_risk_gate | high | Visual, human-in-the-loop stress test -- not an autonomous gate |
| numer.ai | strategy | dual_oracle_consensus | high | Weak comparable -- prediction tournament, not live options execution |

## 4. Capability primitive map

| Primitive | Competitors claiming it | AlpacaCOMP mapping | Router decision |
|---|---|---|---|
| iron_condor_automation | alpaca, composer, optionalpha, quantconnect, tastytrade | strategy.rs (built) | copy -- already have it |
| deterministic_risk_gate | alpaca (partial), tastytrade (partial) | risk_router.rs + market_purity.rs, 1.5us | weaponize |
| dual_oracle_consensus | composer, quantconnect, numerai | oracle_arbiter.rs | invert |
| take_profit_time_stop | optionalpha | strategy.rs (built) | copy -- already have it |
| backtesting_engine | alpaca, composer, optionalpha, quantconnect, tastytrade (all 5) | none found [ASSUMED gap] | watch |
| broker_api_cli_mcp | all 6 | Alpaca CLI decision made, deferred on account/keys | watch -- blocked, not a build-time question |

## 5. Build-router decisions

### Copy
Iron condor/butterfly automation and take-profit/time-stop exits are table-stakes AlpacaCOMP already ships. No further build time needed here.

### Invert
Dual-oracle S13 thesis-token-only arbitration inverts the "AI writes the strategy" pattern seen at Composer and QuantConnect (Mia). Frame as a compliance/trust differentiator, not just an architecture choice.

### Ignore
Numerai's staking/tournament mechanics -- different market structure (long-horizon equity prediction vs live options income), not a judged wedge.

### Watch
Backtesting engine [ASSUMED gap, unverified this pass] and the Alpaca CLI/paper-account blocker. Both need a same-day check before any build-time is spent or skipped.

### Weaponize
1.5us deterministic risk gate with named watchdog states (Divergence / Normal / Convergence Spike) is the standout number nothing in the pinned set matches. This goes on the writeup and demo, contrasted directly against Alpaca's own 1.5ms OMS figure and tastytrade's human-in-the-loop stress test tool.

## 6. Differentiate / table-stakes / cut checkpoint

WRITEUP_WEDGE: 1.5us risk gate + watchdog state machine; S13 dual-oracle thesis-token-only constraint vs competitor LLM-authored strategies
BUILD_WEDGE: [ASSUMED, pending grep-confirmation] backtesting harness, only if remaining hours allow after the CLI/account blocker clears
DO_NOT_OVER_EXPLAIN: Mersenne31 Fredholm resolvent internals -- state the guarantee (R=0 target, exact integer fields), not the derivation
TRUST_ARTIFACT: audited 55.4% win rate / 1.73 profit factor, shown against the watchdog state machine live in the paper account
FIRST_DEMO_MOMENT: a Convergence Spike (>7500 permyriad landmark) triggering a live Gate Fault refusal on camera
FOLLOWUP_PATH: public writeup section explicitly naming alpaca.markets, composer.trade, optionalpha.com, quantconnect.com, tastytrade.com as the compared set

## 7. Friction and failure modes

| Friction | Why it matters | Resolution rule | Kill criterion |
|---|---|---|---|
| Deadline proximity vs audience vanity | 3 days to deadline (2026-09-04 09:00 MDT); no time for a backtesting-engine rebuild | Confirm the gap is real before spending an hour on it -- don't build against an [ASSUMED] claim | If repo grep shows a backtesting primitive already exists, this item is dead on arrival |
| P&L score vs eligibility-gate checkbox | CLAUDE.md ranks P&L as the scored objective; CLI/MCP/options are gates only | Any remaining build hours go to P&L-moving work first, gate-satisfying work second | If the CLI/paper-account blocker isn't cleared, no P&L work is possible at all -- this is the real critical path |
| Evidence quality vs overclaiming | WebFetch summaries are AI-paraphrased, not raw-HTML-hashed per target-policy.md | Treat all quotes above as medium-confidence unless independently re-checked before publishing in the writeup | Any quote used publicly should be re-verified against the live page, not just this pass's summary |

## 8. Next 48-hour actions

1. Grep the AlpacaCOMP repo for any existing backtesting primitive to resolve the [ASSUMED] gap claim before deciding to build or explicitly disclaim it.
2. Escalate the fresh-paper-account + API-key blocker (submission_checklist critical item) -- it blocks both the CLI wiring and any further live P&L evidence.
3. Draft the writeup's competitive-contrast paragraph using the 1.5us gate + dual-oracle framing from this report while it's fresh.

## 9. Evidence appendix

All citations are the six pinned homepages fetched 2026-09-01 via WebFetch (markdown-processed summaries, not raw HTML). See `extract/claims/2026-09-01/claims.jsonl` for the full claim ledger and `extract/primitives/2026-09-01/primitives.json` for the primitive map. No raw HTML/PDF cache was written this pass -- see cache note in section 1.
