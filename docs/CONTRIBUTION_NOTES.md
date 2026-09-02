# Contribution Notes

This repo is already technically strong, so the safest first contribution is presentation and verification support.

## Best First Contribution

Polish the judge-facing story:

- Add a clear README.
- Add a 90-second demo/video plan.
- Keep the architecture simple for non-Rust judges.
- Show exactly what is implemented vs what still needs verification.

## Technical Work To Suggest, Not Touch Blindly

Ask Sean before changing these:

- Strategy-side wing-width cap in `strategy.rs`.
- Alpaca multi-leg `limit_price` sign verification.
- Direct wiring of `order_dag.rs` into `dispatch.rs`.
- Live daemon/governor wiring.

## Suggested Message To Sean

```text
I cloned AlpacaCOMP locally and started with the presentation layer because the engine is strong but the repo needs a judge-facing story. I added a README draft and a video/demo plan that separate implemented pieces from unverified/completion items.

Before I touch trading logic, can you confirm two things?
1. Is AlpacaCOMP the final hackathon submission repo?
2. Do you want me to make the next technical PR around strategy-side wing-width capping, or should I stay focused on README/video/demo polish first?
```

## Repo Relationship

The public Nistam repo is useful as background and style reference. It is not the Alpaca hackathon submission.

Use Nistam for:

- understanding Sean's architecture language
- borrowing README/demo structure
- seeing how he separates measured vs stubbed claims

Use AlpacaCOMP for:

- all current hackathon contribution work
- README, video, demo, and final submission assets
- trading-agent verification and small fixes
