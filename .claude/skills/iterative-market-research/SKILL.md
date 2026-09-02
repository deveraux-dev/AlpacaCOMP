---
name: iterative-market-research
description: local-first iterative market research and launch-prep reporting for the Alpaca AI Trading Agents Hackathon — fixed competitive targets (Alpaca ecosystem, automated options/algo-trading platforms), constrained evidence ledgers, strategy-parity validation, differentiation-vs-table-stakes decisions, and every-other-day pulse reports through the 2026-09-04 deadline. use when the user asks to run or prepare recurring market intelligence, competitor feature extraction, opportunity cards, submission positioning, or writeup/demo differentiation decisions for AlpacaCOMP.
---

# Iterative Market Research — AlpacaCOMP

## Operating doctrine

Run a bounded market-intelligence loop that combines:

- local-first competitive extraction
- strict cost and browsing constraints
- claim/evidence ledgering
- capability primitive normalization
- crucible/friction validation
- build-router outputs
- differentiate-vs-table-stakes submission decision support

Do not produce generic market hype. Produce decision-ready reports that help the user decide what to build, demo, highlight in the writeup, or cut before the 2026-09-04 09:00 MDT deadline.

## Non-negotiable constraints

- Do not use paid APIs, SERP APIs, proxy services, cloud vector databases, or external LLM APIs.
- Do not recommend deep-research modes that trigger credit usage.
- Do not perform broad web crawling.
- Do not scrape behind login, auth, checkout, or account walls.
- If content cannot be fetched within constraints, mark it `unfetched` and continue.
- Use short verbatim quotes only, never more than 25 words per source.
- Treat evidence as claim support, not as proof of market size.
- Fail closed when source quality, access, or confidence is weak.
- GIT=0 applies: this skill never runs git against AlpacaCOMP. Read/Write only.

## Default cadence

Every-other-day report cadence through the deadline unless the user gives a different rhythm.

Each report should answer:

1. What changed since the last run (competitor side, and our build side)?
2. What claims/features did competitor platforms surface?
3. What capability primitives are implied, and do we already have them (see `F:\AlpacaCOMP\CLAUDE.md` architect_reprime)?
4. What does this imply for the remaining build roadmap or the writeup?
5. What should be foregrounded in the demo/writeup, deprioritized, or cut?
6. What is the next bounded test?

If the user wants reminders or automatic recurring reports, ask for the preferred delivery time before scheduling anything.

## Target policy

Use `references/target-policy.md` for pinned targets, allowlisted paths, limits, and output folders.

Default pinned domains:

- `alpaca.markets`
- `composer.trade`
- `optionalpha.com`
- `quantconnect.com`
- `tastytrade.com`
- `numer.ai`

Do not expand the target list unless the user explicitly adds targets.

## Report modes

### Market pulse mode

Use for every-other-day launch reports.

Output:

1. Executive delta
2. Claim ledger highlights
3. Capability primitive map
4. Competitor movement
5. Build-router decisions
6. Differentiate/table-stakes/cut checkpoint
7. Risks and friction
8. Next 48-hour actions
9. Evidence appendix

### Crucible mode

Use when the user asks whether to build, feature, cut, or highlight a capability for judging.

Run:

1. Framing: one sentence, no adjectives.
2. Buyer: who benefits — here, the judge scoring the submission and any paper-trading user.
3. Pain: what costly workflow or risk is removed (e.g. manual iron condor leg selection, unbounded tail risk).
4. Falsification: what would make the claim weak.
5. Prior art: who already ships this (Composer, Option Alpha, tastytrade automation, QuantConnect algos).
6. Service wedge: what's demonstrable in the paper account today.
7. Free wedge: what belongs in the public writeup/repo to build judge trust.
8. Moat: what D=T+F+R / no_std / lock-free / 1.5µs risk gate does faster/safer than competitor stacks.
9. Kill criteria: what stops the feature from making the cut before deadline.

### Build-router mode

Use when evidence needs to become implementation work.

Classify each primitive as:

- `copy`: table-stakes, competitors all have it, build a minimal version.
- `invert`: competitor pattern exists but AlpacaCOMP's stack reverses the cost/latency/safety tradeoff.
- `ignore`: not relevant to the judged wedge (P&L + autonomy + options + Alpaca CLI/MCP).
- `watch`: not enough evidence yet.
- `weaponize`: turn evidence into a writeup talking point, demo moment, or slide.

## Opportunity card format

Use this exact shape when producing opportunities:

```markdown
## OPPORTUNITY: short.machine.readable.name

BUYER:
PAIN:
JUDGING_CRITERION_LEVER:
STACK_LEVER:
DEMO_WEDGE:
WRITEUP_WEDGE:
COMPILER_PRODUCT:
SIGNALS:
PRIOR_ART:
WHY_NOW:
WHY_USER:
FAST_TEST:
TIME_TO_DEADLINE_COST:
DISTRIBUTION_PATH:
FAILURE_MODE:
FRICTION:
CONFIDENCE:
NEXT_ACTION:
CITATIONS:
```

Use tags where helpful:

`[TAG:JUDGING] [TAG:STACK] [TAG:COMPILER] [TAG:DEMO] [TAG:WRITEUP] [TAG:PNL] [TAG:AUTONOMY] [TAG:FRICTION] [TAG:QUALITY]`

## Differentiate vs table-stakes decision rule

Use `references/sell-or-free.md` (retargeted: "sell" = differentiator worth build time, "free" = cheap trust-building writeup/demo content) when the user is unsure whether a capability is worth building before the deadline.

Default posture:

- Put trust-building artifacts (writeup, evidence of audited win rate, risk-gate latency numbers) in the public-facing writeup/demo.
- Spend remaining build time on bounded work that closes a real gap vs. `alpaca.markets`, `composer.trade`, `optionalpha.com`, `quantconnect.com`, `tastytrade.com`, `numer.ai`.
- Keep the D=T+F+R engine and S13 dual-oracle arbiter as leverage, not the first thing over-explained in the pitch.

Good free (writeup/demo) assets:

- audited win-rate and profit-factor numbers
- risk-gate latency benchmark (1.5µs)
- architecture diagram of D=T+F+R
- short demo video of a live paper trade with the watchdog state machine visible
- sample claim ledger of competitor feature parity

Good build-time wedges (worth spending remaining hours on):

- closing a demonstrable capability gap vs. a pinned competitor
- hardening the Alpaca CLI daemon loop / reconcile-only desync handling
- anything that raises audited P&L before the deadline
- anything that makes autonomy/MCP-CLI/options eligibility gates unambiguous to a judge

## Claim extraction

Use the schema in `references/schemas.md`.

Every claim needs:

- source domain
- URL
- retrieval date/time if available
- claim type
- primitive
- short snippet
- cautious paraphrase
- confidence
- evidence pointer or citation
- caveat

Allowed claim types:

`strategy`, `risk_management`, `latency`, `execution`, `automation`, `backtesting`, `capital_allocation`, `options_workflow`, `api_access`, `pricing`, `compliance`, `autonomy`.

## Capability primitives

Normalize competitor language into primitives:

- iron condor / iron butterfly automation
- risk-bounded position sizing
- real-time L2 order book microstructure
- lock-free / low-latency telemetry
- backtesting engine
- paper-trading account infrastructure
- broker API / CLI / MCP integration
- volatility-triggered strategy switching
- dual-oracle / consensus decisioning
- take-profit / time-stop automation
- margin/exposure guardrails
- deterministic risk gate

## Quality gates

An opportunity passes only if:

- a reachable judge/buyer path is named
- a painful workflow or risk gap is named
- evidence supports the gap vs. a pinned competitor
- AlpacaCOMP's stack creates a concrete leverage path (no_std, lock-free, Mersenne31 exact math, 1.5µs gate, etc.)
- the capability is demonstrable in the paper account before deadline
- a writeup/demo wedge exists that builds judge trust without overclaiming
- there is a bounded 48-hour (or less, given the 2026-09-04 deadline) test
- kill criteria are stated before build time is spent

## Output tone

Use direct, grounded language. No hype. No TAM theater. No "billion dollar market" language.

Prefer:

```text
Composer ships iron condor automation with cloud latency; our risk gate authorizes/refuses in 1.5µs locally — that's the differentiator to put on slide 2, not a rebuild of their UI.
```

Avoid:

```text
This disrupts a massive market.
```

## Cadence memory

Each new report should include a compact running ledger:

```markdown
## Running ledger

LAST_REPORT_DATE:
DAYS_TO_DEADLINE:
OPEN_HYPOTHESES:
VALIDATED_SIGNALS:
WEAKENED_SIGNALS:
BUILD_DECISIONS:
WRITEUP_DECISIONS:
NEXT_REPORT_FOCUS:
```

If no prior report is available, start the ledger fresh and label it `baseline`.
