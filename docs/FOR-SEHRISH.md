# Collaborator Working Agreement

This file defines ownership and verification rules for collaborator sessions.

## Ownership

- Sehrish owns the frontend, demo UI, README, public docs, and video/slide assets.
- Sean owns `crates/`, live orders, credentials, and engine receipts.
- Collaborator and AI sessions may inspect `crates/` for evidence but must not edit it.
- Engine suggestions go to Sean for review and implementation.

## Evidence Rule

- Use **verified** only when the same session can cite the exact URL, code location, test, API response, or ledger receipt.
- Without a receipt, write **unverified**, **needs receipt**, or remove the claim from judge-facing material.
- Keep live account identifiers, credentials, and private operational details out of frontend assets.

## Workflow

1. Pull the latest changes before starting.
2. Keep changes inside the collaborator-owned files.
3. Commit and push small, reviewable updates.
4. Open a pull request and wait for Sean's approval before merging.
5. Prefer a reliable, understandable demo over new last-minute features.

## Current Demo Focus

The strongest proof is the recorded oversized-condor refusal:

```text
$2,525 maximum loss > $2,000 account ceiling
Result: refused before Alpaca submission
```

Final submission focus: keep the README, proof portal, pitch video, technical demo, and claim map aligned around this single refusal proof.
