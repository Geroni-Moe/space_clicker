"""Build assets/fonts/PixelifySans.ttf from the upstream font in art/fonts/.

Pixelify Sans draws its 5 as a mirrored 2 with rounded corners, so at UI sizes
"1.25K" reads as "1.2SK". This pins the variable font to its Regular instance
(the only weight the game uses) and redraws the 5 with a flat top and a square
left shoulder, built on the same pixel grid and bowl as the font's own 6.

    python3 -m venv .venv && .venv/bin/pip install fonttools
    .venv/bin/python scripts/patch_font.py
"""

from pathlib import Path

from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "art/fonts/PixelifySans.ttf"
OUT = ROOT / "assets/fonts/PixelifySans.ttf"

# one clockwise contour; the lower half is the 6's bowl, the hook its lower-left corner
FIVE = [
    (151, -12), (151, 78), (60, 78), (60, 179), (161, 179), (161, 88), (423, 88),
    (423, 260), (60, 260), (60, 631), (525, 631), (525, 530), (161, 530), (161, 359),
    (433, 359), (433, 270), (525, 270), (525, 78), (433, 78), (433, -12),
]


def main() -> None:
    font = instantiateVariableFont(TTFont(SRC), {"wght": 400})
    glyphs = font.getGlyphSet()
    pen = TTGlyphPen(glyphs)
    pen.moveTo(FIVE[0])
    for point in FIVE[1:]:
        pen.lineTo(point)
    pen.closePath()
    glyph = pen.glyph()
    glyph.recalcBounds(font["glyf"])
    font["glyf"]["five"] = glyph
    width, _ = font["hmtx"]["five"]
    font["hmtx"]["five"] = (width, glyph.xMin)
    font.save(OUT)
    print(f"wrote {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
