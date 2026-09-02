#!/usr/bin/env python3
"""bake_patex_alpaca.py — PaTeX blueprint for the AlpacaCOMP D=T+F+R stack.
Layout/palette ported from Nistam scripts/bake_patex_diagram.py."""

from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

REPO_ROOT = Path(__file__).parent.parent.resolve()
WIDTH, HEIGHT = 1920, 1080

BG_COLOR = (7, 11, 18, 255)
RULE_COLOR = (34, 68, 85, 255)
BOX_CYAN = (56, 189, 248, 255)
BOX_MUTED = (100, 181, 205, 255)
TEXT_WHITE = (240, 246, 252, 255)
GOLD_COLOR = (245, 158, 11, 255)
GREEN_COLOR = (52, 211, 153, 255)
PURPLE_COLOR = (168, 85, 247, 255)
RED_COLOR = (248, 113, 113, 255)


def panel(draw, xy, label=None, border=BOX_MUTED, fill=(12, 20, 32, 230), font=None):
    x0, y0, x1, y1 = xy
    draw.rectangle([x0, y0, x1, y1], fill=fill, outline=border, width=2)
    if label and font:
        draw.text((x0 + 12, y0 - 12), f" {label} ", fill=border, font=font)


def main():
    img = Image.new("RGBA", (WIDTH, HEIGHT), BG_COLOR)
    draw = ImageDraw.Draw(img)

    for x in range(20, WIDTH, 40):
        for y in range(20, HEIGHT, 40):
            draw.point((x, y), fill=(20, 35, 50, 180))

    try:
        font_title = ImageFont.truetype("consolab.ttf", 20)
        font_header = ImageFont.truetype("consolab.ttf", 15)
        font_body = ImageFont.truetype("consola.ttf", 13)
        font_small = ImageFont.truetype("consola.ttf", 11)
        font_mono_bold = ImageFont.truetype("consolab.ttf", 12)
    except Exception:
        font_title = ImageFont.load_default()
        font_header = font_body = font_small = font_mono_bold = font_title

    draw.rectangle([20, 20, WIDTH - 20, HEIGHT - 20], outline=RULE_COLOR, width=2)
    draw.text((35, 30), "1  AUTONOMOUS OPTIONS AGENT (TOP)  1:1 PaTeX Blueprint · D = T + F + R",
              fill=BOX_MUTED, font=font_title)

    # ── Left column: the deterministic no_std core ─────────────────────────
    panel(draw, (40, 70, 1180, 740),
          "FORGE-GATE  #![no_std] · deny(unsafe) · zero heap · lock-free · 90/90 TESTS GREEN",
          border=BOX_CYAN, font=font_header)

    panel(draw, (60, 100, 1160, 230),
          "GATE LATTICE — EVERY ORDER CLEARS ALL FIVE OR DIES (1.5 us)", border=BOX_MUTED, font=font_header)
    layer1 = [
        ("ORDER-STATE DAG", "Illegal transitions clamp\nto ORDER_REJECT", GREEN_COLOR),
        ("RISK ROUTER", "Exposure bound · margin trip\n2%-of-balance max-loss veto", GOLD_COLOR),
        ("ORACLE ARBITER", "Dual S13[i8;13] Bull/Bear\nconsensus or hard veto", BOX_CYAN),
        ("MARKET PURITY", "N x IPR permyriad [0,10000]\ndiffuse book = refuse", PURPLE_COLOR),
        ("MERKLE-MORIN SEAL", "SHA-256 root / 64B leaves\ntamper = RootMismatch", RED_COLOR),
    ]
    bx = 80
    for title, desc, color in layer1:
        draw.rectangle([bx, 130, bx + 195, 215], fill=(15, 26, 42, 255), outline=color, width=2)
        draw.text((bx + 10, 138), title, fill=color, font=font_mono_bold)
        draw.text((bx + 10, 160), desc, fill=TEXT_WHITE, font=font_small)
        draw.line([(bx + 97, 215), (bx + 97, 245)], fill=BOX_CYAN, width=2)
        bx += 213

    draw.rectangle([80, 245, 1140, 275], fill=(20, 40, 65, 255), outline=BOX_CYAN, width=2)
    draw.text((170, 252),
              "════ ZERO GENERATIVE LAW: THE LLM EMITS S13 THESIS TOKENS ONLY — NEVER STRIKES, GREEKS, OR PAYLOADS ════",
              fill=TEXT_WHITE, font=font_mono_bold)

    panel(draw, (60, 290, 1160, 490),
          "STRATEGY ASSEMBLY — LEGS FROM LIVE CHAIN QUOTES ONLY, NEVER MODEL-HALLUCINATED",
          border=BOX_CYAN, font=font_header)
    fleet = [
        ("IRON CONDOR", "16d short / 5d wings · 45 DTE\n50%-credit take-profit\n21-DTE time stop", (80, 320, 400, 410), BOX_CYAN),
        ("IRON BUTTERFLY", "ATM body, landmark-pinned\nbook trigger (N x IPR >= 7500)\nsame exits, tighter tent", (420, 320, 800, 410), GOLD_COLOR),
        ("5D COLLAPSE", "moneyness/delta/depth/skew/DTE\nthermometer -> [i8;512] trits\nHamming regime router", (820, 320, 1140, 410), GREEN_COLOR),
    ]
    for title, desc, coords, color in fleet:
        draw.rectangle(coords, fill=(16, 28, 48, 255), outline=color, width=2)
        draw.text((coords[0] + 14, coords[1] + 10), title, fill=color, font=font_header)
        draw.text((coords[0] + 14, coords[1] + 35), desc, fill=TEXT_WHITE, font=font_body)

    draw.rectangle([80, 425, 1140, 475], fill=(18, 36, 58, 255), outline=BOX_MUTED, width=2)
    draw.text((100, 435), "FREDHOLM RESIDUE ENGINE (D = T + F + R): permyriad fixed-point resolvent, refuses non-convergence",
              fill=BOX_MUTED, font=font_mono_bold)
    draw.text((100, 452), "MATH SUBSTRATE: exact integer permyriad everywhere · Black-Scholes greeks verified vs Hull + put-call parity",
              fill=TEXT_WHITE, font=font_small)

    panel(draw, (60, 510, 410, 720), "ALPACA CLI BRIDGE (STD SEAM)", border=GOLD_COLOR, font=font_header)
    draw.text((75, 540), "* THE one seam: intent -> execution\n* alpaca.exe v0.0.14 SHA256-verified\n* secrets: env-only, zeroize-on-drop,\n  never argv, never a file\n* typed refusals on every path\n* AIMD pacer, 250ms floor",
              fill=TEXT_WHITE, font=font_small)

    panel(draw, (430, 510, 780, 720), "DAEMON GOVERNOR", border=GREEN_COLOR, font=font_header)
    draw.text((445, 540), "* 7-axis StrainScore, 1s tick\n* order-ack deadline watch\n* WS staleness + pacer pressure\n* gate faults are LOUD (Signal Law)\n* subprocess RSS ceiling 512 MB\n* one thread, one loop, no mutex",
              fill=TEXT_WHITE, font=font_small)

    panel(draw, (800, 510, 1160, 720), "PROOF LEDGER (SEALED)", border=RED_COLOR, font=font_header)
    draw.text((815, 540), "* every claim -> oracle -> receipt row\n* ladder: UNPROVEN->PROVEN->VERIFIED\n* Merkle-Morin session roots chain\n  the log: edit one row, break all\n* session root 2026-09-01:\n  1d93d6df3e995c2ddc116d59...\n* paper acct PA3FMNQT9WDW · $100k",
              fill=TEXT_WHITE, font=font_small)

    # ── Right column ───────────────────────────────────────────────────────
    panel(draw, (1210, 70, 1880, 480), "4  AXONOMETRIC  2:1 dimetric (26.67 deg) Gate Lattice Extrusion",
          border=BOX_MUTED, font=font_header)
    cx, cy = 1545, 260
    for i in range(-6, 7):
        for j in range(-3, 4):
            ix = cx + (i - j) * 26
            iy = cy + (i + j) * 13
            h = (abs(i * j) % 5 + 1) * 12
            draw.polygon([(ix, iy - h), (ix + 22, iy - h + 11), (ix, iy - h + 22), (ix - 22, iy - h + 11)],
                         fill=(24, 60, 90, 255), outline=BOX_CYAN)
            draw.polygon([(ix - 22, iy - h + 11), (ix, iy - h + 22), (ix, iy + 22), (ix - 22, iy + 11)],
                         fill=(14, 38, 58, 255), outline=RULE_COLOR)
            draw.polygon([(ix + 22, iy - h + 11), (ix, iy - h + 22), (ix, iy + 22), (ix + 22, iy + 11)],
                         fill=(10, 26, 42, 255), outline=RULE_COLOR)

    panel(draw, (40, 760, 600, 890), "2  FRONT SECTION (strike / credit)  Depth-Shaded Cut", border=BOX_MUTED, font=font_small)
    draw.rectangle([55, 785, 585, 875], fill=(10, 20, 32, 255), outline=RULE_COLOR)
    for bx in range(65, 575, 18):
        h = 10 + (bx * 37 % 65)
        draw.rectangle([bx, 870 - h, bx + 12, 870], fill=BOX_CYAN, outline=BOX_MUTED)

    panel(draw, (620, 760, 1180, 890), "3  SIDE SECTION (tick / purity pmy)  Depth-Shaded Cut", border=BOX_MUTED, font=font_small)
    draw.rectangle([635, 785, 1165, 875], fill=(10, 20, 32, 255), outline=RULE_COLOR)
    for bx in range(645, 1155, 18):
        h = 15 + (bx * 53 % 60)
        draw.rectangle([bx, 870 - h, bx + 12, 870], fill=GOLD_COLOR, outline=BOX_MUTED)

    panel(draw, (1210, 500, 1880, 890), "5  TITLE BLOCK & PROVENANCE LEDGER", border=GOLD_COLOR, font=font_header)
    tb = [
        ("ALPACACOMP — SOVEREIGN OPTIONS AGENT", GOLD_COLOR, font_title),
        ("Deterministic Gate Lattice, Untrusted-LLM Execution Boundary", BOX_CYAN, font_header),
        ("--------------------------------------------------------------------------------", BOX_MUTED, font_small),
        ("* ZERO GENERATIVE LAW: model emits ternary thesis only; one auditable seam", TEXT_WHITE, font_body),
        ("* GATES: order DAG + risk router + dual-oracle S13 arbiter + N x IPR purity", TEXT_WHITE, font_body),
        ("* 1.5 us risk guardrail latency · exact permyriad integer math throughout", TEXT_WHITE, font_body),
        ("* MERKLE-MORIN SEAL: SHA-256 roots chain the decision ledger, tamper-evident", RED_COLOR, font_body),
        ("* EXECUTION: Alpaca official CLI, checksummed, subprocess-supervised", GREEN_COLOR, font_body),
        ("* LIVE 2026-09-01: SPY 762.15 · condor 795C/815C/719P/690P · $375.50 credit", GOLD_COLOR, font_body),
        ("* PAPER ACCT PA3FMNQT9WDW · $100,000 · options level 3 · 105 tests green", TEXT_WHITE, font_body),
        ("--------------------------------------------------------------------------------", BOX_MUTED, font_small),
        ("BAKED BY PATEX GEOMETRIC TYPESETTING ENGINE", GOLD_COLOR, font_header),
        ("Zero Heap Hotpath · Integer-Only Core · 3^5 = 243 Trit States · S13 Ternary Lanes", BOX_MUTED, font_small),
    ]
    ty = 525
    for text, color, fnt in tb:
        draw.text((1225, ty), text, fill=color, font=fnt)
        ty += 24 if fnt in [font_title, font_header] else 19

    draw.text((45, 910), "ALPACA AI TRADING AGENTS HACKATHON @lablabai @AlpacaHQ | SOLO BUILD | REPO: AlpacaCOMP",
              fill=BOX_MUTED, font=font_small)

    for target in [REPO_ROOT / "patex_alpaca.png", REPO_ROOT / "docs" / "patex_alpaca.png"]:
        target.parent.mkdir(parents=True, exist_ok=True)
        img.save(target, format="PNG")
        print(f"[BAKED] {target}")


if __name__ == "__main__":
    main()
