# Why Alpaca Is Running This Hackathon — deduced 2026-09-01

Label: informed inference from the published rules + product surface, not insider fact.

## The tell is in the eligibility rules, not the prize

- Prize pool is $6,000 — trivial against what they get back.
- Hard requirement: Trading API **plus either their MCP server or their CLI**.
  Both products shipped recently, both are agent-facing surfaces. The rule
  forces every team to exercise the exact interfaces Alpaca needs battle-tested.
- Paper-only, fresh dedicated account per team: a clean, instrumented sandbox
  per participant. They can watch every order an AI agent produces.

## What they're actually buying

1. **Free QA at scale on agent tooling.** Hundreds of teams hammering the MCP
   server and CLI for a week finds integration bugs, auth friction, and rate
   -limit edges no internal test team reproduces.
2. **A corpus of failure modes at the trust boundary.** The unsolved problem in
   AI trading is the seam where model output becomes an order. Most teams let
   the LLM write the payload; the ways that breaks — hallucinated strikes,
   runaway loops, rate-limit storms — land in Alpaca's logs, tagged by account.
3. **Market positioning.** "The brokerage AI agents trade on" is the category
   they want to own. A public hackathon plants the flag cheaply and generates
   build-in-public content tagging them (rule: posts tag @AlpacaHQ).
4. **A hiring/partner funnel.** Judges review every finalist. The submissions
   double as portfolios; the best boundary-handling architectures identify the
   people who can build the agent-execution layer they will need in-house.

## Consequence for this submission

P&L is the scored number, but the audience behind the score is an engineering
org studying how builders keep an untrusted model from touching execution.
A submission that makes its trust boundary legible — one named module where
intent becomes an order, everything else deterministic and refusing by default
— speaks directly to the reason the competition exists.
