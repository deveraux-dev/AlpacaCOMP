from __future__ import annotations

import html
import re
import zipfile
from pathlib import Path

SRC = Path(r"C:\Users\sehri\Downloads\13forge-judge-ready-hackathon-deck.pptx")
REPO = Path(__file__).resolve().parents[1]
OUT = REPO / "docs" / "13forge-final-nontechnical-hackathon-deck.pptx"
ASSET_DIR = REPO / "docs" / "deck-assets"

SLIDE_W = 12192000
SLIDE_H = 6858000
EMU_PER_IN = 914400

INK = "F5F7F5"
MUTED = "9CA3AF"
GREEN = "43B96B"
RED = "FF3D56"
DARK = "030504"
GRID = "161B18"
PANEL = "121217"
PANEL2 = "08120D"
LINE = "303834"


def emu(v: float) -> int:
    return round(v * EMU_PER_IN)


def esc(v: str) -> str:
    return html.escape(v, quote=False)


def text_runs(text: str, size: int, color: str, bold: bool = False, font: str = "Aptos") -> str:
    attrs = f' lang="en-US" sz="{size * 100}"'
    if bold:
        attrs += ' b="1"'
    return (
        f'<a:r><a:rPr{attrs}><a:solidFill><a:srgbClr val="{color}"/>'
        f'</a:solidFill><a:latin typeface="{font}"/></a:rPr><a:t>{esc(text)}</a:t></a:r>'
    )


def shape(
    shape_id: int,
    name: str,
    x: float,
    y: float,
    w: float,
    h: float,
    fill: str | None,
    line: str | None,
    text: str | None = None,
    size: int = 18,
    color: str = INK,
    bold: bool = False,
    font: str = "Aptos",
    prst: str = "rect",
    margin: int = 0,
) -> str:
    fill_xml = '<a:noFill/>' if fill is None else f'<a:solidFill><a:srgbClr val="{fill}"/></a:solidFill>'
    if line is None:
        line_xml = '<a:ln><a:noFill/></a:ln>'
    else:
        line_xml = f'<a:ln w="9525"><a:solidFill><a:srgbClr val="{line}"/></a:solidFill></a:ln>'
    tx = ""
    if text is not None:
        paras = "".join(f"<a:p>{text_runs(part, size, color, bold, font)}</a:p>" for part in text.split("\n"))
        tx = (
            '<p:txBody>'
            f'<a:bodyPr wrap="square" lIns="{margin}" rIns="{margin}" tIns="{margin}" bIns="{margin}"/>'
            f"<a:lstStyle/>{paras}</p:txBody>"
        )
    return f"""
<p:sp>
  <p:nvSpPr><p:cNvPr id="{shape_id}" name="{esc(name)}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
  <p:spPr>
    <a:xfrm><a:off x="{emu(x)}" y="{emu(y)}"/><a:ext cx="{emu(w)}" cy="{emu(h)}"/></a:xfrm>
    <a:prstGeom prst="{prst}"><a:avLst/></a:prstGeom>
    {fill_xml}{line_xml}
  </p:spPr>
  {tx}
</p:sp>"""


def picture(shape_id: int, name: str, rel_id: str, x: float, y: float, w: float, h: float) -> str:
    return f"""
<p:pic>
  <p:nvPicPr>
    <p:cNvPr id="{shape_id}" name="{esc(name)}" descr="{esc(name)}"/>
    <p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr>
    <p:nvPr/>
  </p:nvPicPr>
  <p:blipFill>
    <a:blip r:embed="{rel_id}"/>
    <a:stretch><a:fillRect/></a:stretch>
  </p:blipFill>
  <p:spPr>
    <a:xfrm><a:off x="{emu(x)}" y="{emu(y)}"/><a:ext cx="{emu(w)}" cy="{emu(h)}"/></a:xfrm>
    <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
    <a:ln w="19050"><a:solidFill><a:srgbClr val="{LINE}"/></a:solidFill></a:ln>
  </p:spPr>
</p:pic>"""


def slide_xml(shapes: list[str]) -> str:
    return f'''<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
       xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
      {''.join(shapes)}
    </p:spTree>
  </p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>'''


def notes_xml(notes: str) -> str:
    return f'''<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notes xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
         xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld><p:spTree>
    <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
    <p:grpSpPr><a:xfrm/></p:grpSpPr>
    <p:sp><p:nvSpPr><p:cNvPr id="2" name="Slide Image Placeholder 1"/><p:cNvSpPr/><p:nvPr><p:ph type="sldImg" idx="0"/></p:nvPr></p:nvSpPr><p:spPr/></p:sp>
    <p:sp><p:nvSpPr><p:cNvPr id="3" name="Notes Placeholder 2"/><p:cNvSpPr/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr><p:spPr/>
      <p:txBody><a:bodyPr/><a:lstStyle/><a:p>{text_runs(notes, 14, "000000", False, "Aptos")}</a:p></p:txBody>
    </p:sp>
  </p:spTree></p:cSld>
  <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:notes>'''


def base(shapes: list[str]) -> list[str]:
    out = [shape(2, "Background", 0, 0, 13.333, 7.5, DARK, DARK)]
    sid = 3
    for x in [i / 4 for i in range(0, 54)]:
        out.append(shape(sid, "Grid vertical", x, 0, 0.006, 7.5, GRID, None)); sid += 1
    for y in [i / 4 for i in range(0, 31)]:
        out.append(shape(sid, "Grid horizontal", 0, y, 13.333, 0.006, GRID, None)); sid += 1
    out += shapes
    return out


def deck_slides() -> list[tuple[str, str]]:
    slides = []
    sid = 100

    def add(items: list[str], *args, **kwargs) -> None:
        nonlocal sid
        items.append(shape(sid, *args, **kwargs))
        sid += 1

    s = [
    ]
    add(s, "Logo mark", 0.67, 0.36, 0.34, 0.26, None, None, ">", 22, GREEN, True, "Consolas")
    add(s, "Logo wordmark", 1.06, 0.34, 1.55, 0.34, None, None, "13forge", 24, INK, True, "Aptos Display")
    add(s, "Title", 0.67, 1.35, 10.65, 1.2, None, None, "AI can suggest a trade. 13forge checks it before money moves.", 38, INK, True, "Aptos Display")
    add(s, "Subtitle", 0.69, 2.86, 8.7, 0.42, None, None, "A bad idea should stop at the safety check, not reach the broker.", 20, MUTED)
    add(s, "Flow panel", 0.8, 4.62, 11.75, 1.2, PANEL, LINE, prst="roundRect")
    add(s, "Flow 1", 1.28, 5.05, 2.1, 0.32, None, None, "AI idea", 18, MUTED, True)
    add(s, "Arrow 1", 3.85, 4.97, 0.35, 0.35, None, None, ">", 24, GREEN, True, "Consolas")
    add(s, "Flow 2", 4.65, 5.05, 2.6, 0.32, None, None, "Safety check", 18, GREEN, True)
    add(s, "Stop", 7.45, 4.97, 0.35, 0.35, None, None, "x", 24, RED, True, "Consolas")
    add(s, "Flow 3", 8.28, 5.05, 2.9, 0.32, None, None, "Broker blocked", 18, RED, True)
    add(s, "Footer", 0.67, 6.72, 2.0, 0.22, None, None, "PROBLEM", 12, GREEN, True, "Aptos")
    slides.append((slide_xml(base(s)), "Start in plain language. The problem is not whether AI can come up with a trade idea. The problem is what happens next. In 13forge, the idea has to pass a safety check before money can move toward Alpaca."))

    s = [
    ]
    add(s, "Logo", 0.67, 0.36, 2.2, 0.42, None, None, "13forge", 26, INK, True, "Aptos Display")
    add(s, "Title", 0.67, 1.05, 6.2, 1.18, None, None, "The product makes the decision visible.", 32, INK, True, "Aptos Display")
    add(s, "Subtitle", 0.69, 2.38, 5.85, 0.62, None, None, "A judge can see the idea, the checks, and the result in one screen.", 16, MUTED)
    s.append(picture(sid, "Proof Portal replay screenshot", "rIdProductProof", 6.65, 1.05, 5.75, 4.3)); sid += 1
    for i, (num, head, sub) in enumerate([("1", "Idea", "AI suggests"), ("2", "Check", "Rules test it"), ("3", "Result", "Explain why")]):
        x = 0.82 + i * 1.9
        add(s, f"Step {num}", x, 3.45, 1.6, 1.1, PANEL, LINE, prst="roundRect")
        add(s, f"Step number {num}", x + 0.16, 3.68, 0.32, 0.28, None, None, num, 18, GREEN, True, "Consolas")
        add(s, f"Step title {num}", x + 0.52, 3.7, 0.94, 0.28, None, None, head, 14, INK, True)
        add(s, f"Step body {num}", x + 0.52, 4.12, 0.94, 0.22, None, None, sub, 11, MUTED, False)
    add(s, "Close", 0.82, 5.95, 5.8, 0.46, None, None, "A real product, not a slide-only concept.", 22, GREEN, True, "Aptos Display")
    slides.append((slide_xml(base(s)), "This slide is where the judge sees the product. Keep the explanation simple: 13forge shows the AI idea, runs visible safety checks, and returns a clear result. The screenshot matters because it proves this is not only a pitch concept."))

    s = [
    ]
    add(s, "Logo", 0.67, 0.36, 2.2, 0.42, None, None, "13forge", 26, INK, True, "Aptos Display")
    add(s, "Title", 0.67, 1.18, 10.6, 0.72, None, None, "Every trade has to pass the same checks.", 36, INK, True, "Aptos Display")
    add(s, "Subtitle", 0.69, 2.05, 7.2, 0.32, None, None, "If one check fails, Alpaca never receives the order.", 18, MUTED)
    gates = ["Account", "Position", "AI approval", "Market", "Trade shape", "Loss limit", "Alpaca"]
    for i, label in enumerate(gates):
        x = 0.55 + i * 1.75
        stroke = GREEN if label == "Alpaca" else LINE
        txt = GREEN if label == "Alpaca" else INK
        add(s, f"Gate {i+1}", x, 3.26, 1.45, 1.0, PANEL, stroke, prst="roundRect")
        add(s, f"Gate number {i+1}", x + 0.16, 3.46, 0.48, 0.22, None, None, f"0{i+1}", 14, GREEN, True, "Consolas")
        add(s, f"Gate label {i+1}", x + 0.16, 3.82, 1.1, 0.22, None, None, label, 13, txt, True)
        if i < len(gates) - 1:
            add(s, f"Gate arrow {i+1}", x + 1.52, 3.6, 0.25, 0.25, None, None, ">", 18, MUTED, True, "Consolas")
    for i, label in enumerate(["Uses real market data", "Builds broker order only after pass", "Refuses unsafe trades by default"]):
        x = 1.15 + i * 3.75
        add(s, f"Evidence {i+1}", x, 5.45, 3.0, 0.62, PANEL2 if i == 0 else PANEL, GREEN if i == 0 else LINE, label, 14, GREEN if i == 0 else INK, True, prst="roundRect", margin=91440)
    slides.append((slide_xml(base(s)), "This is the engineering, translated for a non-technical judge. Every proposed trade goes through the same checkpoints: account condition, current position, AI approval, market condition, trade shape, and loss limit. Alpaca is last, and it is only reached when the checks pass."))

    s = [
    ]
    add(s, "Logo", 0.67, 0.36, 2.2, 0.42, None, None, "13forge", 26, INK, True, "Aptos Display")
    add(s, "Title", 0.67, 1.04, 5.9, 0.58, None, None, "Unsafe trade stopped.", 36, INK, True, "Aptos Display")
    add(s, "Subtitle", 0.69, 1.88, 5.8, 0.52, None, None, "The loss is too high, so the broker is not reached.", 17, MUTED)
    s.append(picture(sid, "Proof Portal refusal screenshot", "rIdRefusalProof", 6.95, 1.08, 5.45, 3.08)); sid += 1
    cards = [("$2,525", "possible loss", RED), ("$2,000", "allowed limit", INK), ("REFUSED", "No Alpaca", RED)]
    for i, (big, small, c) in enumerate(cards):
        x = 0.85 + i * 1.98
        add(s, f"Metric card {i+1}", x, 3.28, 1.7, 1.05, PANEL, RED if big == "REFUSED" else LINE, prst="roundRect")
        add(s, f"Metric {i+1}", x + 0.18, 3.52, 1.35, 0.36, None, None, big, 23 if big != "REFUSED" else 20, c, True, "Aptos Display")
        add(s, f"Metric label {i+1}", x + 0.18, 4.0, 1.35, 0.2, None, None, small, 11, MUTED, True)
    s.append(picture(sid, "Proof video QR code", "rIdProofQr", 10.92, 4.48, 1.18, 1.18)); sid += 1
    add(s, "Proof CTA", 0.85, 5.25, 2.7, 0.5, PANEL2, GREEN, "WATCH THE PROOF", 15, GREEN, True, prst="roundRect", margin=91440)
    add(s, "Proof URL", 3.72, 5.37, 2.6, 0.24, None, None, "youtu.be/IrLmPIXukyo", 13, MUTED, True)
    add(s, "Close", 0.85, 6.35, 6.0, 0.32, None, None, "AI can suggest. 13forge decides.", 22, INK, True, "Aptos Display")
    slides.append((slide_xml(base(s)), "Close with the proof in human terms. The proposed trade could lose two thousand five hundred twenty-five dollars, but the account limit is two thousand dollars. 13forge refuses it, and Alpaca is not reached. That is the whole product promise in one moment."))
    return slides


def update_package() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    slides = deck_slides()
    product_img = ASSET_DIR / "slide2-product-proof-crop.png"
    refusal_img = ASSET_DIR / "slide4-refusal-proof-crop.png"
    qr_img = ASSET_DIR / "proof-video-qr.png"
    logo_svg = REPO / "demo-portal" / "assets" / "brand" / "13forge-logo-dark.svg"
    with zipfile.ZipFile(SRC, "r") as zin, zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as zout:
        for item in zin.infolist():
            name = item.filename
            if re.match(r"ppt/slides/slide[1-5]\.xml$", name):
                idx = int(re.search(r"slide(\d+)", name).group(1))
                if idx <= 4:
                    zout.writestr(item, slides[idx - 1][0])
                continue
            if re.match(r"ppt/notesSlides/notesSlide[1-5]\.xml$", name):
                idx = int(re.search(r"notesSlide(\d+)", name).group(1))
                if idx <= 4:
                    zout.writestr(item, notes_xml(slides[idx - 1][1]))
                continue
            if name == "ppt/presentation.xml":
                xml = zin.read(name).decode("utf-8-sig")
                xml = re.sub(r'<p:sldId id="260" r:id="Rd964569c05dc4675" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" />', "", xml)
                zout.writestr(item, xml)
                continue
            if name == "ppt/_rels/presentation.xml.rels":
                xml = zin.read(name).decode("utf-8-sig")
                xml = re.sub(r'<Relationship Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="/ppt/slides/slide5.xml" Id="Rd964569c05dc4675" />', "", xml)
                zout.writestr(item, xml)
                continue
            if name == "[Content_Types].xml":
                xml = zin.read(name).decode("utf-8-sig")
                if '<Default Extension="png"' not in xml:
                    xml = xml.replace(
                        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml" />',
                        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml" /><Default Extension="png" ContentType="image/png" />'
                    )
                if '<Default Extension="svg"' not in xml:
                    xml = xml.replace(
                        '<Default Extension="png" ContentType="image/png" />',
                        '<Default Extension="png" ContentType="image/png" /><Default Extension="svg" ContentType="image/svg+xml" />'
                    )
                xml = re.sub(r'<Override PartName="/ppt/slides/slide5.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide\+xml" />', "", xml)
                xml = re.sub(r'<Override PartName="/ppt/notesSlides/notesSlide5.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesSlide\+xml" />', "", xml)
                zout.writestr(item, xml)
                continue
            if name == "ppt/slides/_rels/slide2.xml.rels":
                xml = zin.read(name).decode("utf-8-sig")
                if "rIdProductProof" not in xml:
                    xml = xml.replace(
                        "</Relationships>",
                        '<Relationship Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/slide2-product-proof-crop.png" Id="rIdProductProof" /></Relationships>'
                    )
                zout.writestr(item, xml)
                continue
            if name == "ppt/slides/_rels/slide1.xml.rels":
                xml = zin.read(name).decode("utf-8-sig")
                if "rIdLogoDark" not in xml:
                    xml = xml.replace(
                        "</Relationships>",
                        '<Relationship Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/13forge-logo-dark.svg" Id="rIdLogoDark" /></Relationships>'
                    )
                zout.writestr(item, xml)
                continue
            if name == "ppt/slides/_rels/slide4.xml.rels":
                xml = zin.read(name).decode("utf-8-sig")
                if "rIdRefusalProof" not in xml:
                    xml = xml.replace(
                        "</Relationships>",
                        '<Relationship Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/slide4-refusal-proof-crop.png" Id="rIdRefusalProof" /></Relationships>'
                    )
                if "rIdProofQr" not in xml:
                    xml = xml.replace(
                        "</Relationships>",
                        '<Relationship Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/proof-video-qr.png" Id="rIdProofQr" /></Relationships>'
                    )
                zout.writestr(item, xml)
                continue
            if re.match(r"ppt/slides/_rels/slide5\.xml\.rels$", name):
                continue
            if re.match(r"ppt/notesSlides/_rels/notesSlide5\.xml\.rels$", name):
                continue
            zout.writestr(item, zin.read(name))
        zout.writestr("ppt/media/slide2-product-proof-crop.png", product_img.read_bytes())
        zout.writestr("ppt/media/slide4-refusal-proof-crop.png", refusal_img.read_bytes())
        zout.writestr("ppt/media/proof-video-qr.png", qr_img.read_bytes())
        zout.writestr("ppt/media/13forge-logo-dark.svg", logo_svg.read_bytes())
    print(OUT)


if __name__ == "__main__":
    update_package()
