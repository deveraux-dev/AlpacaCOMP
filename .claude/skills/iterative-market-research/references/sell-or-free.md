# Differentiate vs Table-Stakes Decision Guide (retargeted from sell-vs-free)

## Default rule

Put trust in the writeup/demo. Spend build time on the real gap.

## Put in writeup/demo when

- It builds judge credibility faster than it costs build hours.
- It shows a number a competitor platform can't match (1.5µs risk gate, audited win rate, profit factor).
- It creates a shareable proof artifact (video, chart, log excerpt).
- It shows the shape of D=T+F+R without over-explaining the engine internals.
- It helps a judge quickly place AlpacaCOMP against `alpaca.markets`, `composer.trade`, `optionalpha.com`, `quantconnect.com`, `tastytrade.com`, `numer.ai`.

Good writeup/demo assets:

- audited win-rate / profit-factor summary
- risk-gate latency benchmark
- architecture diagram
- short demo video of a live paper trade under the watchdog state machine
- comparison table vs. pinned competitors

## Spend build time when

- The gap is demonstrable and directly maps to a judging criterion (P&L, autonomy, options usage, Alpaca CLI/MCP).
- A competitor ships something AlpacaCOMP claims but can't yet show live.
- The workflow (order-state DAG, risk router, oracle arbiter) needs hardening to survive a live demo.
- The output is something a judge will directly click through or watch run.

Good build-time wedges:

- closing a concrete capability gap vs. a pinned competitor
- Alpaca CLI daemon loop / reconcile-only desync handling
- anything that raises audited P&L before deadline
- anything that makes the autonomy/MCP-CLI/options eligibility gates unambiguous

## Do not over-invest in

- rebuilding a competitor's UI
- reusable automation that isn't visible in the demo or writeup
- private workflow details that don't move a judging criterion
- proprietary scoring logic explained at a level that invites nitpicking instead of trust
- polish with no time-to-deadline payoff

## Effort posture

Start with the bounded, demoable increment, not a platform rebuild.

Possible time bands (hours remaining before 2026-09-04 09:00 MDT):

- trivial: writeup copy edit, chart, README polish
- small: single capability demo hardening
- medium: one new strategy leg / one new risk check, tested
- large: daemon loop / live paper-trading integration — only if blocking deferred item is cleared

Use real judging-criterion weight (P&L is scored; autonomy/MCP-CLI/options are eligibility gates) to set final priority.
