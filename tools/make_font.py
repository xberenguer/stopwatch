#!/usr/bin/env python3
"""Generate compact grayscale glyph masks. Pillow is needed only to regenerate."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import argparse
import struct

p = argparse.ArgumentParser()
p.add_argument('--font', default='/System/Library/Fonts/HelveticaNeue.ttc')
p.add_argument('--index', type=int, default=7)  # Helvetica Neue Light
a = p.parse_args()
root = Path(__file__).resolve().parents[1]
chars = '0123456789:. LAPRESETUM'
for name, size in [('LARGE', 80), ('COMPACT', 57), ('SMALL', 23), ('LABEL', 16)]:
    scale = 3
    font = ImageFont.truetype(a.font, size * scale, index=a.index)
    data = bytearray()
    glyphs = []
    digit_width = round(max(font.getlength(str(i)) for i in range(10)) / scale) + 1
    for c in dict.fromkeys(chars + ('HODF' if name == 'LABEL' else '')):
        l, t, r, b = font.getbbox(c, anchor='ls')
        w, h = max(1, (r - l + scale - 1) // scale), max(1, (b - t + scale - 1) // scale)
        im = Image.new('L', (w * scale, h * scale))
        ImageDraw.Draw(im).text((-l, -t), c, fill=255, font=font, anchor='ls')
        im = im.resize((w, h), Image.Resampling.LANCZOS)
        pixels = [round(v / 17) for v in im.tobytes()]
        offset = len(data)
        for j in range(0, len(pixels), 2):
            data.append((pixels[j] << 4) | (pixels[j+1] if j+1 < len(pixels) else 0))
        advance = digit_width if c.isdigit() else max(w, round(font.getlength(c) / scale)) + 1
        glyphs.append(struct.pack('<BBBBbH', ord(c), w, h, advance, round(t / scale), offset))
    packed = bytes([len(glyphs)]) + b''.join(glyphs) + data
    (root / 'assets' / f'{name.lower()}.bin').write_bytes(packed)
