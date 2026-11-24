"""Pack assets/icon/icon_*.png (drawn by art/scripts/icon.lua) into
assets/icon/VoidArmada.ico for the Windows executable. Each size keeps its
hand-drawn pixels: the PNGs go into the .ico as they are (Vista+ format).

    python3 scripts/make_ico.py
"""

import struct
from pathlib import Path

ICON = Path(__file__).resolve().parent.parent / "assets/icon"
SIZES = [16, 32, 64, 128, 256]


def main() -> None:
    images = [(ICON / f"icon_{s}.png").read_bytes() for s in SIZES]
    out = struct.pack("<HHH", 0, 1, len(images))
    offset = 6 + 16 * len(images)
    for size, data in zip(SIZES, images):
        side = size % 256  # 0 means 256
        out += struct.pack("<BBBBHHII", side, side, 0, 0, 1, 32, len(data), offset)
        offset += len(data)
    out += b"".join(images)
    (ICON / "VoidArmada.ico").write_bytes(out)
    print("wrote assets/icon/VoidArmada.ico")


if __name__ == "__main__":
    main()
