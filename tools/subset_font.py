#!/usr/bin/env python3
"""Keep the shipped CJK font small while covering the actual UI strings."""
from pathlib import Path
import argparse
from fontTools import subset

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("font", type=Path, help="NotoSansCJK-Regular.ttc")
args = parser.parse_args()
characters = "".join(path.read_text() for path in (root / "src").rglob("*.rs"))
options = subset.Options()
options.font_number = 2  # Simplified Chinese in the Noto CJK collection.
font = subset.load_font(str(args.font), options)
subsetter = subset.Subsetter(options=options)
subsetter.populate(text=characters + "".join(chr(i) for i in range(32, 127)))
subsetter.subset(font)
subset.save_font(font, str(root / "assets/fonts/AyakageUI.otf"), options)
