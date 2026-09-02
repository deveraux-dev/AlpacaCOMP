# 13forge: A Bicameral Trading Agent That Never Trusts Its Own Mind

---

## Slide 1: The Problem
**LLMs Hallucinate Financial Risk**
- **The Core Issue:** Every LLM trading agent has the same failure mode: the model that dreams up the trade also writes the order.
- **The Danger:** One hallucinated strike, one confidently mislabeled assumption (like flipping credit vs. debit signs), and generated text becomes a live, disastrous position.
- **The Reality:** We caught our own AI making a sign error that would have flipped a $328 credit into a $328 debit. 
- **The Question:** How do we let AI discover opportunities without trusting it to write the order?

---

## Slide 2: The Solution
**The Zero-Generative Law & Bicameral Architecture**
- **Zero-Generative Law:** The LLM is banned from writing orders. It can only emit a 13-lane balanced-ternary vector (−1/0/+1) representing a market thesis.
- **Bicameral System:** Two independent models (Strategist and Risk Sentinel) generate competing theses.
- **Deterministic Arbiter:** A hard-coded Rust oracle compares the vectors. Disagreement refuses the trade instantly.

---

## Slide 3: Architecture
**The 5-Step Execution Airlock**
Orders pass through 5 deterministic, math-only gates before the Alpaca CLI is ever spawned. A refused order costs zero syscalls.
1. **Oracle Verdict Veto:** Do the two models agree?
2. **Purity / Chaos Gate:** Is the market structure safe (bid/ask spread valid)?
3. **Leg Geometry:** Are the spread wings valid and quoted?
4. **2% Max-Loss Veto:** Is the maximum potential loss mathematically under 2% of equity?
5. **CLI Subprocess:** Only then is the order dispatched to Alpaca.

---

## Slide 4: Proof & Receipts
**The Engine Working In Reality**
- **Live Proof:** We tested the system by feeding it an oversized, unsafe Iron Condor (max loss $2,525 on a $100k account).
- **Result:** The 2% Max-Loss Veto killed the trade *before* it reached Alpaca.
- **Sign Safety:** The engine forces correct limit price signs (negative for credit spreads), catching the very hallucination that sparked this project.
- *Everything is logged in an immutable Merkle proof ledger.*

---

## Slide 5: The Value
**Safer Autonomous Options Trading**
- **Business Value:** Brokerages and retail traders can finally trust AI agents with live capital, because the risk layer is mathematically guaranteed.
- **Originality:** We didn't build an LLM wrapper. We built a `no_std` Rust gate lattice that treats the LLM as an untrusted signal generator.
- **Next Steps:** Open-source the gate lattice, wire in the position-state DAG, and deploy on bare metal.
- **Receipts:** Zero orders, ever, asserted into existence without mathematical proof.
