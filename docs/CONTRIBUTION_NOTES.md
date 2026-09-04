# Frontend and Submission Contribution Notes

## Current Responsibility

- guided proof portal
- README and claim-safe public documentation
- deployed static demo URL
- 90-second demo recording
- five-slide presentation and three-minute pitch

`crates/` is read-only from collaborator sessions.

## Completed on the Engine Side

The current code contains receipts for these previously open items:

- Alpaca multi-leg credit prices serialize as negative values.
- Condor and butterfly builders apply a wing-width cap.
- The position-state DAG is checked first in `dispatch_spread`.

These are no longer collaborator implementation tasks.

## Remaining Collaborator Critical Path

1. Keep the deployed root pointed at `demo-portal/`.
2. Keep every visible claim aligned with `docs/CLAIM_PROOF_MAP.md`.
3. Preserve the portal as a static, receipt-backed replay with no keys, live balances, or broker calls.
4. Keep the README links current for the pitch video, technical demo, proof portal, and repository.
5. Treat new frontend/backend wiring as out of scope unless the engine owner provides a deliberately safe integration path.

## Review Message

```text
I synced the proof portal, README, docs, cover image, slide assets, and final video links around the same claim-safe story: AI can propose an options trade, but deterministic Rust gates decide whether it can reach Alpaca. The portal remains a static proof replay, not a runtime trading dashboard, which keeps the public demo reliable and credential-free.
```
