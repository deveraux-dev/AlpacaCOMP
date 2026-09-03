# 13forge Demo Video Plan

Goal: make the problem, working behavior, and technical difference clear in 90 seconds.

## Core Message

AI proposes. Deterministic Rust gates decide. Alpaca receives an order only after every implemented safety check passes.

The video uses the static proof portal as its main visual. It does not need live credentials or a live order.

## 90-Second Storyboard

### 0-12 seconds: The Problem

Narration:

> An AI trading model can produce a confident idea and still create an invalid or over-risked order. 13forge separates creative trading ideas from execution authority.

Show the portal headline and the three-step summary: AI suggests, Rust verifies, Alpaca receives passed orders only.

### 12-30 seconds: The Governed Path

Narration:

> Before Alpaca is contacted, the order must pass position-state, model-verdict, market-stability, trade-structure, and maximum-loss checks.

Show the six-row safe order path. Make clear that Alpaca submission is the destination, not another safety gate.

### 30-65 seconds: The Proof

Click **Run safety checks**.

Narration:

> This recorded test case proposes a 29-point iron-condor wing for a 3 dollar and 75 cent credit. Its maximum loss is 2,525 dollars. The account ceiling is 2,000 dollars, so the final risk check refuses it. Alpaca is never reached.

Pause briefly on **REFUSED** and **NOT REACHED**.

### 65-78 seconds: API Correctness

Scroll to the price-format receipt.

Narration:

> The order builder also pins Alpaca's multi-leg convention: credits serialize as negative prices and debits as positive prices.

### 78-90 seconds: Close

Scroll to the system map and claim board.

Narration:

> This is autonomous trading with a hard boundary: the model can suggest, but code decides whether capital may move. Every public claim maps to code, a test, or a receipt.

End on the project name, public repository, and deployed portal URL.

## Recording Checklist

- Record at 1920x1080 with browser zoom at 100 percent.
- Hide bookmarks, notifications, account identifiers, credentials, and unrelated tabs.
- Use one continuous portal walkthrough; avoid terminal switching unless Sean supplies a clean fresh receipt.
- Confirm the replay ends with **Order refused before Alpaca** and **Not reached**.
- Keep unreceipted win-rate, profit-factor, latency, and test-count figures off screen.
- Export one clean 1080p MP4 and verify audio before submission.

## Optional Evidence Inserts

Use only if Sean provides a fresh capture from the current commit:

- full Rust test result
- read-only Alpaca paper-account status
- paper-order receipt
- benchmark output with command and machine context

The portal replay remains the reliable fallback if any live evidence is unavailable.
