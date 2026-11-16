#!/usr/bin/env python3
"""List warm-hued dark greys (hue 15-65 deg, low saturation, low lightness) in the sources:
components, GPUI palettes, tokens and the whole website shell (site/).

Oxyn's default themes are neutral: dark surfaces are pure greys. Semantic colours (status, risk,
diff, brand) are saturated and are not matched here. Exit code 1 when anything is found.
Usage: python3 scripts/warm_darks.py [--all]   (--all also prints the light warm greys)
"""
import colorsys, os, re, sys

ROOTS = ['crates', 'apps', 'site/src', 'site/public', 'site/astro.config.mjs', 'tokens', 'tools', 'docs/site']
EXT = ('.css', '.rs', '.json', '.ts', '.astro', '.mjs', '.js', '.html', '.svg')
SKIP = ('site/src/generated', 'site/public/wasm', 'node_modules', '/target', 'tokens/presets')
HEX = re.compile(r'#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b')
RGB = re.compile(r'rgba?\(\s*(\d{1,3})\s*[, ]\s*(\d{1,3})\s*[, ]\s*(\d{1,3})')
GPUI = re.compile(r'\b0x([0-9a-fA-F]{6})(?:[0-9a-fA-F]{2})?\b')


def warm(r, g, b, light=False):
    h, l, s = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
    hue = h * 360
    if max(r, g, b) == min(r, g, b):
        return False
    if not (15 <= hue <= 65):
        return False
    if s > 0.35:
        return False
    return light or l < 0.40


def scan(light=False):
    hits = []
    for root in ROOTS:
        walk = [(os.path.dirname(root), [], [os.path.basename(root)])] if os.path.isfile(root) else os.walk(root)
        for dp, _, fs in walk:
            if any(x in dp for x in SKIP):
                continue
            for f in fs:
                if not f.endswith(EXT):
                    continue
                p = os.path.join(dp, f)
                prev = ''
                for i, line in enumerate(open(p, encoding='utf-8', errors='ignore'), 1):
                    # `warm-ok` on the line or the line above: illustration colours, not chrome
                    skip, prev = 'warm-ok' in line or 'warm-ok' in prev, line
                    if skip:
                        continue
                    cols = []
                    for m in HEX.finditer(line):
                        x = m.group(1)
                        if len(x) == 3 and not p.endswith('.css'):
                            continue  # '#884' in Rust/TS text is a ticket number, not a colour
                        if len(x) == 3:
                            x = ''.join(c * 2 for c in x)
                        cols.append((m.group(0), *(int(x[k:k + 2], 16) for k in (0, 2, 4))))
                    for m in GPUI.finditer(line):
                        x = m.group(1)
                        cols.append(('0x' + x, *(int(x[k:k + 2], 16) for k in (0, 2, 4))))
                    for m in RGB.finditer(line):
                        cols.append((m.group(0), *(int(v) for v in m.groups())))
                    for c, r, g, b in cols:
                        if max(r, g, b) <= 255 and warm(r, g, b, light):
                            hits.append((p, i, c))
    return hits


if __name__ == '__main__':
    hits = scan('--all' in sys.argv)
    for p, i, c in hits:
        print(f'{p}:{i}: {c}')
    print(f'{len(hits)} warm dark colour(s)')
    sys.exit(1 if hits else 0)
