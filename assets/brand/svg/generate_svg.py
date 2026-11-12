"""Generate the static vector (SVG) versions of the Oxyn mark v3.

Deterministic: no randomness, no timestamps. Re-running produces identical files.

The dot positions, sizes and inks come straight from the v3 renderer
(../render_logo.py, imported, not re-implemented): the canonical pose is
loop frame 0 (p = 0), the rounded-cube state shown at the start of the banner,
with both orbits fully open in the picture plane and the two ember carets at
the top-right and bottom-right of the X.

Usage:  python3 generate_svg.py [path/to/InterDisplay-SemiBold.ttf]
        (or set OXYN_FONT). Requires numpy, pillow, fonttools.
"""
import importlib.util, math, os, sys
sys.dont_write_bytecode = True   # don't leave __pycache__ next to render_logo.py
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("render_logo", os.path.join(HERE, "..", "render_logo.py"))
R = importlib.util.module_from_spec(spec); spec.loader.exec_module(R)

FONT = (sys.argv[1] if len(sys.argv) > 1 else
        os.environ.get("OXYN_FONT", "/workspace/work/concept-video/fonts/InterDisplay-SemiBold.ttf"))

FRAME = 0                      # canonical pose (cube), t = 0.00 s of the 5 s loop
hexc = lambda c: "#%02X%02X%02X" % tuple(int(v) for v in c)
INK, BG, ACC = hexc(R.INK), hexc(R.BG), hexc(R.ACCENT)       # #ECE8E0 #0B0B0C #E27042
RING = hexc(R.RING_FAR + (R.RING_NEAR - R.RING_FAR) * 0.5)   # orbits lie in the picture plane at p=0 -> k=0.5
f2 = lambda v: ("%.2f" % v).rstrip("0").rstrip(".").replace("-0", "-0") if abs(v) >= 0.005 else "0"


def pose(p=FRAME / R.N):
    """Exactly the renderer's maths for the core, rings and carets at loop phase p.
    Returns object-space items: x, y (y up, in units of `scale`), sizes in units of u."""
    w = R.shape_weights(p)
    P = sum(w[k] * R.SHAPES[k] for k in range(3)) * 0.60
    Rm = R.VIEW @ R.rot_y(math.radians(45) + p * 2 * math.pi * 0.25)
    W = P @ Rm.T
    dots = []
    for k in range(R.NDOTS):
        x, y, z = W[k]
        depth = min(1.0, max(0.0, (z / (0.60 * 1.25) + 1) / 2))
        tw = 0.5 + 0.5 * math.sin(2 * math.pi * (2 * p + R.PHASE[k]))
        r = (0.9 + 1.9 * depth) * (0.85 + 0.15 * tw)          # in u
        ink = (0.10 + 0.90 * depth ** 1.4) * (0.85 + 0.15 * tw)
        dots.append((z, x, y, r, ink, depth))
    dots.sort(key=lambda d: d[0])
    rings, carets = [], []
    for sign, direction in ((+1, +1), (-1, -1)):
        a = np.array([sign * math.sqrt(0.5), math.sqrt(0.5), 0.0])
        perp = np.array([-a[1], a[0], 0.0])
        spin = direction * p * 2 * math.pi
        b = math.cos(spin) * perp + math.sin(spin) * np.array([0, 0, 1.0])
        RA, RB = 1.95, 0.78
        rings.append((sign, RA, RB))
        pt = lambda t: a * RA * math.cos(t) + b * RB * math.sin(t)
        head = direction * p * 2 * math.pi * 2 + (0 if sign > 0 else math.pi)
        for j in range(6):
            q = pt(head - direction * j * 0.11)
            kk = (q[2] / RB + 1) / 2
            bright = 0.55 + 0.45 * kk
            if j == 0:
                carets.append(("head", q[0], q[1], 6.5, 17.0, bright))
            else:
                carets.append(("tok", q[0], q[1], 6.0 - 0.7 * j, 6.0 - 0.7 * j, (1 - j / 6) * bright))
    return dots, rings, carets


def svg(size, body, bg=None, radius=0.0):
    out = ['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 %s %s">' % (f2(size), f2(size))]
    if bg:
        out.append('<rect width="%s" height="%s" rx="%s" fill="%s"/>' % (f2(size), f2(size), f2(radius), bg))
    out += body
    out.append("</svg>\n")
    return "\n".join(out)


def mark_body(c, scale, mono=False, color=None, ring_color=None, accent=None):
    """SVG elements for the full-detail mark centred at c (scale = renderer's `scale`)."""
    u = scale / 140.0                    # renderer: scale = 0.2 S, u = S / 700
    dots, rings, carets = pose()
    X = lambda x: c + x * scale
    Y = lambda y: c - y * scale
    ink_col = color or INK
    body = []

    def dot_groups(sel):
        groups = {}
        for z, x, y, r, ink, depth in sel:
            if mono:
                key = "1"
            else:
                key = "%.2f" % round(ink, 2)
            groups.setdefault(key, []).append('<circle cx="%s" cy="%s" r="%s"/>' % (f2(X(x)), f2(Y(y)), f2(r * u)))
        res = []
        for key in sorted(groups, key=float):
            attr = '' if mono else ' fill-opacity="%s"' % key.lstrip("0")
            res.append('<g%s>%s</g>' % (attr, "".join(groups[key])))
        return res

    if mono:
        # single colour: drop the far (back-face) dots that the colour version fades to near-black,
        # keep depth via dot size only.
        dots = [d for d in dots if d[5] >= 0.42]
    back = [d for d in dots if d[0] < 0]
    front = [d for d in dots if d[0] >= 0]
    body.append('<g fill="%s">' % ink_col)
    body += dot_groups(back)
    body.append("</g>")
    rc = ring_color or (color if mono else RING)
    for sign, RA, RB in rings:
        body.append('<ellipse cx="%s" cy="%s" rx="%s" ry="%s" transform="rotate(%d %s %s)" fill="none" stroke="%s" stroke-width="%s"/>'
                    % (f2(c), f2(c), f2(RA * scale), f2(RB * scale), -45 * sign, f2(c), f2(c), rc, f2(1.8 * u)))
    ac = accent or (color if mono else ACC)
    toks, heads = [], []
    for kind, x, y, wdt, hgt, op in carets:
        if mono and kind == "tok" and op < 0.4:
            continue
        rx = wdt / 2 if kind == "head" else wdt * 0.25
        el = '<rect x="%s" y="%s" width="%s" height="%s" rx="%s"%s/>' % (
            f2(X(x) - wdt * u / 2), f2(Y(y) - hgt * u / 2), f2(wdt * u), f2(hgt * u), f2(rx * u),
            "" if mono else ' fill-opacity="%s"' % f2(op).lstrip("0"))
        (heads if kind == "head" else toks).append(el)
    body.append('<g fill="%s">%s</g>' % (ac, "".join(toks + heads)))
    body.append('<g fill="%s">' % ink_col)
    body += dot_groups(front)
    body.append("</g>")
    return body


# ---------- small-size marks (simplified, pixel-snapped) ----------
SMALL_RING = "#5E5851"  # small sizes only: a 1 px orbit needs a touch more contrast than the large hairline


def cube_corners():
    """The 7 visible corners of the rounded cube at the canonical pose, projected with the
    renderer's own shape + view maths. Returns (hexagon silhouette, front corner, the three
    corners joined to the front corner by a cube edge)."""
    C = np.array([[x, y, z] for x in (-1, 1) for y in (-1, 1) for z in (-1, 1)], float)
    W = (R.to_shape(C / math.sqrt(3), 0) * 0.60) @ (R.VIEW @ R.rot_y(math.radians(45))).T
    fi, bi = int(np.argmax(W[:, 2])), int(np.argmin(W[:, 2]))
    hexa = [(W[i][0], W[i][1]) for i in range(8) if i not in (fi, bi)]
    hexa.sort(key=lambda p: math.atan2(p[0], p[1]))
    arms = [(W[i][0], W[i][1]) for i in range(8) if np.sum(C[i] != C[fi]) == 1]
    return hexa, (W[fi][0], W[fi][1]), arms


def small_body(px):
    """Simplified mark for 16/24 px. Same pose and projection; the cube becomes a few large
    pixel-aligned dots (7 corners at 16 px; corners + the front corner's three edge midpoints at 24 px) on a
    faint hexagon of the core colour, the orbits are 1 px strokes kept clear of the core, and the
    two ember caret heads sit at the orbit tips as in the full mark."""
    hexa, front, arms = cube_corners()
    pts = hexa + [front]
    if px > 16:   # 24 px: add the midpoints of the three edges meeting at the front corner (the cube's "Y")
        pts += [((front[0] + p[0]) / 2, (front[1] + p[1]) / 2) for p in arms]
    c = px / 2
    scale = 4.6 * px / 16 if px <= 16 else 7.0
    core = 1.3
    r = 0.5 if px <= 16 else 0.6      # corner radius of the 2x2 px dot
    tint = 0.14 if px <= 16 else 0.12
    snap = lambda v: float(round(v))                           # dot centres on pixel corners -> crisp 2x2 px dots, symmetric about the centre line
    poly = lambda k: " ".join("%s,%s" % (f2(c + x * scale * core * k), f2(c - y * scale * core * k)) for x, y in hexa)
    mid = "oxyn%dm" % px
    body = ['<mask id="%s"><rect width="%d" height="%d" fill="#fff"/><polygon points="%s"/></mask>' % (mid, px, px, poly(1.18)),
            '<g fill="none" stroke="%s" stroke-width="1" mask="url(#%s)">' % (SMALL_RING, mid)]
    for sign in (+1, -1):
        body.append('<ellipse cx="%s" cy="%s" rx="%s" ry="%s" transform="rotate(%d %s %s)"/>'
                    % (f2(c), f2(c), f2(1.95 * scale), f2(0.78 * scale), -45 * sign, f2(c), f2(c)))
    body.append("</g>")
    body.append('<polygon points="%s" fill="%s" fill-opacity="%s"/>' % (poly(1.05), INK, f2(tint).lstrip("0")))
    # each dot is a 2x2 px rounded square: fully covers its pixels at 1x, still reads as a dot when zoomed
    body.append('<g fill="%s">%s</g>' % (INK, "".join('<rect x="%s" y="%s" width="2" height="2" rx="%s"/>' % (
        f2(snap(c + x * scale * core) - 1), f2(snap(c - y * scale * core) - 1), f2(r)) for x, y in pts)))
    t = 1.95 * math.sqrt(0.5) * scale
    cw, ch = (2, 3) if px <= 16 else (2, 4)
    for sy in (-1, +1):
        x = min(px - cw, round(c + t - cw / 2)); y = max(0, min(px - ch, round(c + sy * t - ch / 2)))
        body.append('<rect x="%d" y="%d" width="%d" height="%d" rx="1" fill="%s"/>' % (x, y, cw, ch, ACC))
    return body


# ---------- wordmark ----------
def glyph_paths(text, size, x0, baseline):
    from fontTools.ttLib import TTFont
    from fontTools.pens.svgPathPen import SVGPathPen
    from fontTools.pens.transformPen import TransformPen
    f = TTFont(FONT)
    gs = f.getGlyphSet(); cmap = f.getBestCmap(); upm = f["head"].unitsPerEm
    k = size / upm
    pen = SVGPathPen(gs, ntos=lambda v: f2(v))
    x = 0.0
    for ch in text:
        gname = cmap[ord(ch)]
        tp = TransformPen(pen, (k, 0, 0, -k, x0 + x * k, baseline))
        gs[gname].draw(tp)
        x += f["hmtx"][gname][0]
    return pen.getCommands(), x * k


def text_bbox(text, size, x0, baseline):
    from fontTools.ttLib import TTFont
    from fontTools.pens.boundsPen import BoundsPen
    from fontTools.pens.transformPen import TransformPen
    f = TTFont(FONT); gs = f.getGlyphSet(); cmap = f.getBestCmap(); k = size / f["head"].unitsPerEm
    bp = BoundsPen(gs); x = 0.0
    for ch in text:
        g = cmap[ord(ch)]
        gs[g].draw(TransformPen(bp, (k, 0, 0, -k, x0 + x * k, baseline))); x += f["hmtx"][g][0]
    return bp.bounds  # xmin, ymin, xmax, ymax (svg coords)


def banner_baseline():
    # banner draws "oxyn" with PIL at (800, 270), anchor 'la' (top = ascender), 190 px
    from PIL import ImageFont
    return 270 + ImageFont.truetype(FONT, 190).getmetrics()[0]


def main():
    out = HERE
    files = {}
    S = 512
    sc = 148.0                     # ring extent ~±1.49 scale + caret -> ~±226 px, ~30 px clear margin
    files["oxyn-mark.svg"] = svg(S, mark_body(S / 2, sc))
    files["oxyn-mark-dark-bg.svg"] = svg(S, mark_body(S / 2, 134.0), bg=BG, radius=112)
    files["oxyn-mark-mono.svg"] = svg(S, mark_body(S / 2, sc, mono=True, color="currentColor"))
    files["oxyn-mark-16.svg"] = svg(16, small_body(16))
    files["oxyn-mark-24.svg"] = svg(24, small_body(24))

    # wordmark: lowercase "oxyn", Inter Display SemiBold, as in the banner
    size = 190.0
    bl = banner_baseline()
    xmin, ymin, xmax, ymax = text_bbox("oxyn", size, 0, 0)
    d, _ = glyph_paths("oxyn", size, -xmin, -ymin)
    files["oxyn-wordmark.svg"] = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 %s %s">\n<path fill="%s" d="%s"/>\n</svg>\n'
                                  % (f2(xmax - xmin), f2(ymax - ymin), INK, d))

    # lockup: banner geometry (mark 700 px at (90,50) -> centre (440,400), scale 140; text at x 800)
    mcx, mcy, msc = 440.0, 400.0, 140.0
    ext = 1.95 * math.sqrt(0.5) * math.hypot(1, 0.4) * msc    # ring half-extent
    ext = max(ext, 1.95 * math.sqrt(0.5) * msc + 8.5 * msc / 140)
    tx0, ty0, tx1, ty1 = text_bbox("oxyn", size, 800, bl)
    pad = 40.0
    vx0, vx1 = mcx - ext - pad, tx1 + pad
    vy0, vy1 = mcy - ext - pad, mcy + ext + pad
    # vertically centre the wordmark's x-height on the mark (banner places it slightly high; keep banner spacing)
    d, _ = glyph_paths("oxyn", size, 800, bl)
    body = ['<g transform="translate(%s %s)">' % (f2(-vx0), f2(-vy0))]
    mb = mark_body(0, msc)
    body.append('<g transform="translate(%s %s)">' % (f2(mcx), f2(mcy)))
    body += mb
    body.append("</g>")
    body.append('<path fill="%s" d="%s"/>' % (INK, d))
    body.append("</g>")
    files["oxyn-lockup.svg"] = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 %s %s">\n' % (f2(vx1 - vx0), f2(vy1 - vy0))
                                + "\n".join(body) + "\n</svg>\n")

    for name, txt in files.items():
        with open(os.path.join(out, name), "w") as fh:
            fh.write(txt)
        print("%-24s %7d bytes" % (name, len(txt.encode())))


if __name__ == "__main__":
    main()
