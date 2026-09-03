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
2. Check every visible claim against `docs/CLAIM_PROOF_MAP.md`.
3. Publish a static preview without keys or live broker calls.
4. Record the refusal replay and add the final URL to the README.
5. Finish the slide PDF and presentation video.

## Review Message

```text
I synced the proof portal with the latest live order path, including the position-state check. I also removed stale and unreceipted claims from the public docs. Please review the portal flow and claim wording; the remaining blocker is publishing the static preview URL, then I can record the demo and finish the slides.
```
