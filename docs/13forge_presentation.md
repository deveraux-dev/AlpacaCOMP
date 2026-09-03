# 13forge Presentation Outline

Five slides for a clear three-minute pitch.

## Slide 1: The Problem

### AI trading needs an execution boundary

- A model can produce a useful trade thesis and still propose invalid legs or excessive risk.
- Sending model output directly to a broker turns a reasoning mistake into a financial action.
- The question: how can an agent remain autonomous without giving the model unchecked authority?

Visual: one line from AI proposal to a blocked broker order.

## Slide 2: The Solution

### AI proposes. Deterministic Rust gates decide.

- The strategist emits a constrained thesis.
- Market quotes, not model text, supply the option strikes.
- A fixed Rust path can refuse the order before Alpaca is contacted.

Visual: Strategist -> Rust safety gates -> Alpaca paper trading.

## Slide 3: The Live Order Path

### Five checks before one broker submission

1. Position-state check
2. Model-verdict check
3. Market-stability check
4. Trade-structure check
5. Maximum-loss check

Only a passing order reaches the Alpaca multi-leg submission step.

Visual: use the portal's six-row safe order path.

## Slide 4: The Proof

### The strongest result is a refusal

- Proposed trade: 29-point iron-condor wing with a $3.75 credit.
- Calculated maximum loss: $2,525.
- Allowed ceiling: 2 percent of a $100,000 paper account, or $2,000.
- Result: refused before Alpaca submission.
- API receipt: a $3.28 credit serializes as `-3.28`.

Visual: `$2,525 > $2,000`, followed by **REFUSED** and **ALPACA NOT REACHED**.

## Slide 5: Why It Matters

### Autonomous, not unconstrained

- **Application of technology:** AI generates the thesis; deterministic code controls execution.
- **Business value:** a reusable safety boundary for autonomous brokerage workflows.
- **Originality:** a `no_std`, unsafe-denied Rust gate core instead of a chatbot directly writing orders.
- **Transparency:** public claims map to code, tests, or receipts.

Close:

> The model can suggest. The gate decides. Alpaca only sees orders that pass.

Show the repository and deployed proof-portal URL.

## Presenter Notes

- Say **paper trading**, not live customer capital.
- Call Alpaca submission the destination, not a sixth safety gate.
- Do not show performance, latency, or test-count figures without a fresh receipt.
- Keep Fredholm, Merkle, and API pacing as optional engineering depth, not the main demo claim.
