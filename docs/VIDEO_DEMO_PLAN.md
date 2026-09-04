# 13forge Demo Video Plan

Goal: create a polished 70-85 second hackathon demo that feels smooth, human, and cinematic while proving the real product works.

## Core Message

AI may propose the trade. Deterministic Rust gates decide whether capital can move. Alpaca receives an order only after every implemented safety check passes.

The real proof is the portal replay: an AI-proposed oversized iron condor is refused because `$2,525` maximum loss exceeds the `$2,000` risk ceiling, and the broker step is not reached.

## Refined Voiceover

Use this warmer, more human script for the final ElevenLabs or Canva voiceover:

> Large language models are good at generating trading ideas. The problem is that they can also be confidently wrong.
>
> In options trading, one bad strike or one missed risk calculation can put real capital in danger.
>
> This is 13forge. It lets AI propose an idea, but it does not let AI place the trade.
>
> Every order has to pass through deterministic Rust safety gates before Alpaca is ever contacted.
>
> Here, the AI suggests an oversized iron condor.
>
> The system checks the governor state, position state, model verdict, market stability, trade structure, and maximum loss.
>
> Then it computes the real downside.
>
> A twenty-nine point wing, minus a three dollar and seventy-five cent credit, creates a maximum loss of two thousand five hundred twenty-five dollars.
>
> The account limit is two thousand dollars.
>
> So 13forge refuses the order.
>
> The broker step stays locked. Alpaca is never reached.
>
> That is the boundary: AI can suggest, but code decides whether capital can move.
>
> 13forge: creative AI, deterministic control.

## 90-Second Storyboard

| Time | Visual | Narration focus |
| --- | --- | --- |
| 0-5s | Canva intro card with 13forge logo/name | Creative AI, deterministic control |
| 5-16s | Portal headline and proof rail | AI can be useful and confidently wrong |
| 16-28s | Seven-gate safe order path | Alpaca is behind deterministic checks |
| 28-50s | Click **Run safety checks** | The unsafe condor is dissected by the gates |
| 50-62s | Hold on `$2,525`, `$2,000`, **REFUSED**, **Not reached** | Maximum-loss veto prevents broker contact |
| 62-72s | Price-format receipt | API correctness and receipt discipline |
| 72-82s | Claim board / system proof | Code decides whether capital can move |

## Recording Link

Use the normal judge link for submission:

```text
https://13forge-proof-portal.vercel.app/demo-portal/
```

Use this recording helper link only while capturing footage:

```text
https://13forge-proof-portal.vercel.app/demo-portal/?recording=1
```

The helper mode adds a small recording slate and supports these keys:

- `1`: jump to intro
- `2`: jump to proof replay
- `3`: run safety checks
- `4`: jump to evidence
- `5`: jump to architecture
- `Esc`: hide the slate

## Recording Checklist

- Record at 1920x1080 with browser zoom at 100 percent.
- Hide bookmarks, notifications, account identifiers, credentials, and unrelated tabs.
- Use one continuous 70-80 second portal walkthrough; avoid terminal switching unless Sean supplies a clean fresh receipt.
- Do not stretch a 30-second recording to match the full voiceover.
- Pause visibly on **$2,525**, **$2,000**, **REFUSED**, **Order refused before Alpaca**, and **Not reached**.
- Keep unreceipted win-rate, profit-factor, and latency figures off screen.
- Export one clean 1080p MP4 and verify audio before submission.

## Canva/Edit Instructions

- Import the screen recording and ElevenLabs MP3.
- Use Canva for the intro, outro, captions, subtle zooms, and final export.
- Keep all AI-generated visuals around the edges only; the middle must be real portal footage.
- Add callouts only to receipt-backed facts: `$2,525`, `$2,000`, `REFUSED`, `NOT REACHED`, and `Broker process not started`.
- Avoid generic trading charts, Wall Street footage, robot visuals, fake dashboards, and any claim that says or implies "absolute safety."

## Submission Links

- Live portal: `https://13forge-proof-portal.vercel.app/demo-portal/`
- Repository: `https://github.com/deveraux-dev/AlpacaCOMP`
- PPT pitch video: `https://youtu.be/YO1xjnEtoF4`
- Live tech demo video: `https://youtu.be/IrLmPIXukyo`

Upload the final video early and verify every link in an incognito/private browser.

## Acceptance Checks

- Without sound, a judge can still understand the refusal.
- On mobile, `$2,525`, `$2,000`, **REFUSED**, and **NOT REACHED** remain readable.
- The video shows the real 13forge UI, not only slides.
- No unsupported claims appear, especially win rate, profit factor, latency, or "absolute safety."
- The final frame includes the live portal URL and GitHub repository.
