# For Sehrish — Welcome to 13forge

Sean asked me (his Claude session) to write this for you. Use it or don't — it's yours.
If it's easier to read in Urdu, paste it into your own Claude and ask for a translation.

## What this project is, in plain words

An autonomous trading agent for the Alpaca hackathon (deadline: **Thursday, 8:00 PM your time**, 9 AM Sept 4 Mountain). It trades options on a paper (fake-money) account. The heart of it is trust discipline: the AI model is never allowed to place a trade directly. Every order must pass six mechanical safety gates, in a fixed order, and any gate can refuse. The demo's best moment is the machine *refusing* a bad trade with real numbers.

## Your lane

- **Yours**: front end, demo UI, README, docs, video/slide assets. Full ownership.
- **Read-only**: everything under `crates/` (the Rust trading engine). Look all you want, change nothing — that goes for your AI assistant too (tell it so at the start of each session; it will otherwise try to "help").
- The `.claude/workflows/777-cascade-uiux.js` file is a ready-made deep-analysis run for the demo UI. In a Claude Code session in the repo, say: *"run the 777-cascade-uiux workflow"*. It produces a build spec where every screen element is tied to a real data source.

## About last night — please read this one

Your README is in, lightly edited, and it's good. Your idea to cap the butterfly's wings is **in the engine now, credited to your commit**. But your session also pushed a change to the price-sign rule labeled "VERIFIED" that was backwards — Alpaca wants credit orders as *negative* prices, and the change would have flipped a real order from "collect $328" into "pay $328." Nobody's upset; your *instinct* (enforce the sign so nobody can get it wrong) was correct, and it was caught in review. But it's why this project has one law above all others:

**Never let any tool — including AI — label something VERIFIED unless it cites the exact source.** If it can't point to the page or the file line, the label is "unverified guess," and unverified guesses don't get pushed. That one habit is the entire project philosophy, and honestly it's most of what "being technical" is. You already have the instincts; receipts are just how you make instincts portable.

## Working across the distance (built for your grid, not ours)

Power and internet where you are can vanish for hours — so the workflow assumes it:

1. **Pull before you start** (`git pull`), every session, no exceptions — Sean's side pushes engine changes at odd hours.
2. **Commit small and push often.** A push every 20–30 minutes of real work means a load-shedding hit costs you minutes, not a day. An unpushed masterpiece doesn't exist.
3. **Nothing waits on you being online at a set hour.** Coordination is async: the repo, commit messages, and whatever chat you and Sean use. If you're dark for 12 hours, the project keeps moving and nothing you own breaks.
4. Market hours (when trades happen) are **6:30 PM – 1:00 AM your time** — evening work, if you ever want to watch it live. Never required.

## If you're stuck

Ask Sean anything, or open a Claude session in the repo and ask it — the repo's CLAUDE.md gives any session the full picture. There are no dumb questions here; the whole codebase is built on the assumption that everyone, human and AI, verifies instead of pretends to know.

— written 2026-09-02, Sean's session
