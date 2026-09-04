# 13forge Final 4-Slide Hackathon Deck

Deck file: `docs/13forge-final-4-slide-hackathon-deck.pptx`

Source deck used read-only: `C:\Users\sehri\Downloads\13forge-judge-ready-hackathon-deck.pptx`

Visual reference used read-only: `C:\Users\sehri\Downloads\13forge Pitch Deck.pdf`

## Slide Plan

| Slide | Judge question | Visible message |
| --- | --- | --- |
| 1 | Why does this matter? | AI trading fails when ideas become orders too fast. |
| 2 | What did we build? | 13forge makes AI ask for permission. |
| 3 | Why is it technically credible? | The order path is a deterministic airlock. |
| 4 | What should I remember? | The demo proves the refusal. |

## Speaker Notes

### Slide 1

Open by naming the risk, not the product. Large language models are good at generating trading ideas, but in options trading a confident mistake can become a real broker payload. The point of 13forge is to stop that handoff from being automatic.

### Slide 2

Now introduce the product experience. 13forge is not another trading chatbot. It uses AI where AI is useful: proposing a thesis. The execution path is separate. Rust owns permission, structure, risk, and the final broker boundary.

### Slide 3

This is the technical proof slide. The important architecture is the separation of authority: suggestion, market data, safety gates, and Alpaca submission are not the same step. The implemented dispatch path includes governor, position-state, oracle verdict, market purity, leg geometry, max-loss, and then broker submission through the Alpaca CLI/API path.

### Slide 4

Close on the exact thing judges can remember. In the public replay, the AI proposes an oversized iron condor. The system computes a maximum loss of two thousand five hundred twenty-five dollars against a two thousand dollar ceiling, refuses the trade, and shows that the broker process was not started. That is why 13forge should stand out: it makes agentic trading useful without giving the model unchecked authority over capital.

## Continuous 3-Minute Script

Large language models are good at generating trading ideas, but in options trading the dangerous moment is when an idea becomes an order too fast. One hallucinated strike, one missed risk check, or one malformed payload can put real capital in front of a broker API.

13forge is built around a simple boundary. AI can propose a bounded thesis, but AI does not get to place the trade. The execution path is separate. Rust rebuilds the order from market data, checks the trade, and produces a receipt for what happened.

That is why the product experience is intentionally narrow. The model suggests. The airlock checks. The system either refuses or allows the order path to continue. The important user experience is not a dashboard full of predictions; it is the moment where the product explains why capital can or cannot move.

Technically, 13forge is a deterministic order airlock. The path checks governor state, position state, oracle permission, market stability, trade geometry, and maximum loss before Alpaca is reached. The strategy builders use ChainQuote data rather than model-invented strikes. The Alpaca mleg payload follows the credit-price convention and is sent through stdin instead of leaking order JSON through command arguments.

The demo proves the core behavior with one memorable case. The AI proposes an oversized iron condor. The system calculates a maximum loss of $2,525 against a $2,000 account risk ceiling. 13forge refuses the order, and the broker process is not started.

That is the reason to remember 13forge: it keeps the creativity of AI upstream, but gives final execution permission to deterministic code. Creative AI, deterministic control.

## Verified Feature Representation

- `LIVE`: deterministic dispatch gate path before broker work.
- `LIVE`: governor, position-state, oracle verdict, market purity, leg geometry, and maximum-loss checks.
- `TESTED`: oversized iron condor refusal where `$2,525` exceeds `$2,000`.
- `VERIFIED`: Alpaca mleg credit price convention serializes credit prices as negative values.
- `VERIFIED BY TEAM RECEIPT`: `159/159 tests: 110 forge-gate + 36 forge-daemon + 13 example tests`.
- `PUBLIC`: live demo portal, demo video, and GitHub repository links are present in README and `demo-portal/proof-data.json`.

## Claim Boundaries

- Do not claim guaranteed safety.
- Do not show win rate, profit factor, or latency as verified unless a fresh receipt is added.
- Do not claim Merkle seal, Fredholm residue, or API pacer as execution gates unless current dispatch evidence proves it.
- Do not present the static portal replay as live account trading.

## Judge Review

`VERDICT`: stronger as a 4-slide pitch than the previous 5-slide version.

`JUDGE SEES`: problem, product, technical boundary, and proof in the correct order.

`STRONG`: one memorable refusal case, real Alpaca boundary, and claim discipline.

`PROBLEM`: the deck is intentionally sparse, so the presenter must use the notes rather than relying on visible paragraphs.

`PRIORITY`: submit this deck only with the public video and portal links beside it.

`FIX`: none required before submission unless a rendered PowerPoint preview shows layout drift on another machine.

`WHY`: it answers the strongest judge question: why should an AI trading agent be trusted near capital?

`NEXT`: open the PPTX once in PowerPoint or Canva, confirm the black-grid slides render correctly, then upload.

## Sources

- Repository README: `README.md`
- Claim ledger: `docs/CLAIM_PROOF_MAP.md`
- Demo proof data: `demo-portal/proof-data.json`
- Demo plan: `docs/VIDEO_DEMO_PLAN.md`
- Official hackathon context checked against Lablab.ai Alpaca AI Trading Agents Hackathon page: `https://lablab.ai/event/alpaca-ai-trading-agents-hackathon`
