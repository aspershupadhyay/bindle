# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
"""Builds the app icons from the 24px logo sprite.

Matches the Brand sheet: the logo on a clay-wash rounded square, filling three
quarters of it, scaled by a whole number with no smoothing.

The Tauri icon tool smooths small sizes, which blurs pixel art. So run it, then
run this script again to redraw the small PNGs crisply:

    python design/tools/app_icon.py app/src/art/bindle-24.png app/src-tauri/icons
    (cd app && npx tauri icon src-tauri/icons/source.png)
    python design/tools/app_icon.py app/src/art/bindle-24.png app/src-tauri/icons
"""
import os
import sys

from PIL import Image, ImageDraw

CLAY_WASH = (0xF6, 0xE6, 0xDC, 255)
SAND = (0xE8, 0xE1, 0xD6, 255)

# Every size keeps the 24px sprite at a whole-number scale: size * 3/4 / 24.
SIZES = {
    "source.png": 1024,
    "32x32.png": 32,
    "64x64.png": 64,
    "128x128.png": 128,
    "128x128@2x.png": 256,
}


def draw(sprite, size):
    """Draws one icon of `size` pixels square."""
    scale = size // 32
    icon = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    ImageDraw.Draw(icon).rounded_rectangle(
        (0, 0, size - 1, size - 1),
        radius=round(size * 240 / 1024),
        fill=CLAY_WASH,
        outline=SAND,
        width=max(1, size // 128),
    )
    big = sprite.resize((24 * scale, 24 * scale), Image.NEAREST)
    icon.alpha_composite(big, (4 * scale, 4 * scale))
    return icon


def main(sprite_path, out_dir):
    sprite = Image.open(sprite_path).convert("RGBA")
    for name, size in SIZES.items():
        draw(sprite, size).save(os.path.join(out_dir, name))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
