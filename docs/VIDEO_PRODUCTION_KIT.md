# 13forge Video Production Kit

This is the fast path for producing the final hackathon demo.

## Exact Workflow

1. Open the recording helper URL.
2. Record a fresh 70-80 second real portal walkthrough first.
3. Import the screen recording into Canva.
4. Import the refined Amelia ElevenLabs voiceover.
5. Align cuts and zooms to the voiceover, not the other way around.
6. Add captions from `docs/video-demo-captions.srt`.
7. Export the final 1080p MP4.
8. Upload the MP4 to the hackathon-required video host.
9. Verify `README.md` and `demo-portal/proof-data.json` both use the final video links: PPT pitch `https://youtu.be/YO1xjnEtoF4` and live tech demo `https://youtu.be/IrLmPIXukyo`.

Do not stretch a short capture to fill the whole narration. The screen recording is the proof; Canva is the polish layer.

## Pitch Deck Visual Reference

Use `C:\Users\sehri\Downloads\13forge Pitch Deck.pdf` as the visual reference for title cards and proof callouts.

- Keep the front and end cards visually related: black grid background, large 13forge wordmark, muted gray support line, and green proof accent.
- Use only one opening brand card. Do not show the logo twice at the start.
- Keep the opening and closing cards short, about 2-3 seconds each, so the demo spends time on the actual portal.
- Preserve the deck's proof language: green means allowed/verified path, red means veto/refusal, gray means system context.
- Favor big single ideas over crowded text. The viewer should understand the proof with sound off.
- Do not copy the deck's source slide into the final video.

## Smooth Recording Pass

Record a new walkthrough at normal speed. Use this timing:

| Time | Action |
| --- | --- |
| 0-6s | Start on the hero. Hold steady on "An AI idea meets a hard safety boundary." |
| 6-16s | Scroll slowly to the proof rail and safety story. Do not rush the scroll. |
| 16-28s | Move to the replay section. Hold on the seven-gate path before clicking. |
| 28-48s | Click **Run safety checks** and let the gates animate naturally. |
| 48-60s | Hold on the refused state. Make `$2,525`, `$2,000`, **REFUSED**, and **Not reached** readable. |
| 60-70s | Scroll to the price-format receipt. Hold briefly on `-3.28`. |
| 70-80s | End on the claim board/system proof section. |

Keep the cursor calm. Move, stop, let the viewer read, then move again.

## Files And Links

- Live portal: `https://13forge-proof-portal.vercel.app/demo-portal/`
- Recording helper: `https://13forge-proof-portal.vercel.app/demo-portal/?recording=1`
- GitHub: `https://github.com/deveraux-dev/AlpacaCOMP`
- Voiceover file: `C:\Users\sehri\Downloads\ElevenLabs_2026-09-03T21_23_40_Amelia - Neutral Narration_pvc_sp100_s42_sb80_v3.mp3`
- Captions: `docs/video-demo-captions.srt`

## Final Script

Use this warmer, more human version for the next voiceover:

Large language models are good at generating trading ideas. The problem is that they can also be confidently wrong.

In options trading, one bad strike or one missed risk calculation can put real capital in danger.

This is 13forge. It lets AI propose an idea, but it does not let AI place the trade.

Every order has to pass through deterministic Rust safety gates before Alpaca is ever contacted.

Here, the AI suggests an oversized iron condor.

The system checks the governor state, position state, model verdict, market stability, trade structure, and maximum loss.

Then it computes the real downside.

A twenty-nine point wing, minus a three dollar and seventy-five cent credit, creates a maximum loss of two thousand five hundred twenty-five dollars.

The account limit is two thousand dollars.

So 13forge refuses the order.

The broker step stays locked. Alpaca is never reached.

That is the boundary: AI can suggest, but code decides whether capital can move.

13forge: creative AI, deterministic control.

## Canva Prompt

Create a polished hackathon demo video for a product named 13forge. Style: modern technical, proof-first, restrained, high contrast, no generic trading footage, no fake dashboards. Use the uploaded screen recording as the main footage. Use the uploaded Amelia voiceover as the master audio. Follow the attached pitch deck's black grid, white headline, muted gray text, green proof accent, and red refusal accent style for title cards and callouts. Create one 2-3 second intro card with the 13forge logo and the subtitle "Creative AI. Deterministic control." Then show the real portal recording as the main body. Add clean captions from the supplied transcript. Add subtle zooms and callouts only on these proof moments: "$2,525", "$2,000", "REFUSED", "NOT REACHED", and "Broker process not started." End with a matching 2-3 second outro showing the live portal URL and GitHub URL. Do not use stock Wall Street footage, robots, generic charts, duplicate opening logo screens, or claims of absolute safety.

## Editor Prompt After Recording

Use this prompt after the real screen recording exists:

```text
Act as an elite hackathon demo video editor and product storyteller.

Create a smooth 70-85 second demo video for 13forge using my uploaded screen recording as the main proof footage and my uploaded Amelia ElevenLabs voiceover as the master audio.

Context:
- 13forge is a proof-first autonomous options execution engine.
- The core story is: AI may propose a trade, but deterministic Rust code decides whether capital can move.
- The demo must prove one concrete thing: an oversized iron condor is refused because $2,525 maximum loss exceeds the $2,000 risk ceiling.
- Alpaca must be shown as "not reached" after the refusal.
- The project is for hackathon judges, so the video must show the real portal UI rather than stock footage or fake generated dashboards.

Editing instructions:
- Start with one 2-3 second intro card: "13forge" and "Creative AI. Deterministic control."
- Match the intro and outro to the pitch deck visual system: black grid, large white type, muted gray support copy, green proof accent, and red only for refusal/veto moments.
- Use the real portal screen recording for the main body.
- Make the movement feel natural: normal speed scrolling, short holds, and smooth zooms.
- Do not stretch the whole recording unnaturally. If extra time is needed, use still holds on receipt-backed proof moments.
- Avoid long holds on the first page, last page, or single-number proof cards. Add short intermediate portal screens instead of freezing one frame for too long.
- Sync the proof moments to the voiceover.
- Add subtle zooms/callouts only on: "$2,525", "$2,000", "REFUSED", "NOT REACHED", and "Broker process not started."
- Use gentle push-in zooms of 105-115%, not aggressive camera moves.
- Add readable captions from the supplied transcript.
- End with a matching 2-3 second outro showing:
  - Live portal: https://13forge-proof-portal.vercel.app/demo-portal/
  - GitHub: https://github.com/deveraux-dev/AlpacaCOMP

Constraints:
- Do not add generic trading footage, Wall Street clips, robot visuals, fake dashboards, or unsupported performance claims.
- Do not say or imply "absolute safety."
- Keep the final export clean, technical, premium, human, and understandable with the sound off.
- Export as 1920x1080 MP4.
```

## README Update After Upload

The final uploaded demo is:

```md
**PPT pitch video:** [Watch the 13forge presentation](https://youtu.be/YO1xjnEtoF4)

**Live tech demo video:** [Watch the 13forge proof replay](https://youtu.be/IrLmPIXukyo)
```

`README.md` and `demo-portal/proof-data.json` should both point to `https://youtu.be/YO1xjnEtoF4` for the PPT pitch and `https://youtu.be/IrLmPIXukyo` for the live tech demo.

## Edit Checklist

- Keep the finished video between 80 and 90 seconds.
- Put all AI-generated polish around the real screen recording.
- Make the refusal readable without audio.
- Keep captions large enough for mobile.
- Verify the final upload in a private browser before submitting.

## Export Settings

- Format: MP4
- Resolution: 1920x1080
- Frame rate: 30 fps
- Visibility: public or unlisted, depending on the hackathon platform rules
- Final filename: `13forge-demo-video.mp4`
