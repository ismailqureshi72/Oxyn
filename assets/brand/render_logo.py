"""Oxyn mark v3: particle core (thinking-orb style dot field) morphing
rounded cube -> tube -> sphere -> cube, two thin muted orbits locked in an X
(each spins on its long axis), and a token-stream caret travelling each orbit.
Dot language inspired by thinking-orbs (Jakub Antalik, MIT): depth carried by
dot size + ink weight, z-sorted, matte, no glow.
"""
import math, os, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont

BG = np.array([11, 11, 12], float)
INK = np.array([236, 232, 224], float)
RING_FAR = np.array([38, 36, 34], float)
RING_NEAR = np.array([84, 79, 72], float)
ACCENT = np.array([226, 112, 66], float)     # muted ember, used only for the carets
FONT = "/tmp/inter/extras/ttf/InterDisplay-SemiBold.ttf"
FONT_R = "/tmp/inter/extras/ttf/Inter-Regular.ttf"
N = 150   # 5 s loop @ 30 fps
NDOTS = 900

def ease(t):
    t = max(0.0, min(1.0, t)); return 0.5 - 0.5 * math.cos(math.pi * t)

# morph weights over the loop: cube -> tube -> sphere -> cube
KEYS = [(0.00, 0), (0.16, 0), (0.33, 1), (0.50, 1), (0.67, 2), (0.83, 2), (1.00, 0)]
def shape_weights(p):
    for (t0, a), (t1, b) in zip(KEYS, KEYS[1:]):
        if t0 <= p <= t1:
            k = ease((p - t0) / (t1 - t0)) if t1 > t0 else 0
            w = np.zeros(3); w[a] += 1 - k; w[b] += k
            return w
    return np.array([1.0, 0, 0])

# Regular grid on each cube face (reads as a cube in dots), then every
# point keeps its direction so it can glide onto the tube and the sphere.
G = 11
g = (np.arange(G) + 0.5) / G * 2 - 1
pts = []
for axis in range(3):
    for sgn in (-1, 1):
        for a_ in g:
            for b_ in g:
                v = [0.0, 0.0, 0.0]
                v[axis] = sgn; v[(axis + 1) % 3] = a_; v[(axis + 2) % 3] = b_
                pts.append(v)
DIRS = np.array(pts); DIRS /= np.linalg.norm(DIRS, axis=1)[:, None]
NDOTS = len(DIRS)
i = np.arange(NDOTS)
PHASE = (np.sin(i * 12.9898) * 43758.5453) % 1.0

def to_shape(d, kind):
    x, y, z = np.abs(d[:, 0]), np.abs(d[:, 1]), np.abs(d[:, 2])
    if kind == 0:   # rounded cube (superellipsoid p=7)
        p = 9.0; n = (x**p + y**p + z**p) ** (1 / p); s = 0.92
    elif kind == 1:  # tube: round in xz, flat caps (soft)
        q = 7.0; r = np.sqrt(x * x + z * z); n = (r**q + (y / 1.15)**q) ** (1 / q); s = 0.86
    else:            # sphere
        n = np.sqrt(x * x + y * y + z * z); s = 1.0
    return d / n[:, None] * s

SHAPES = [to_shape(DIRS, k) for k in range(3)]

def rot_y(a):
    c, s = math.cos(a), math.sin(a); return np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])
def rot_x(a):
    c, s = math.cos(a), math.sin(a); return np.array([[1, 0, 0], [0, c, -s], [0, s, c]])

VIEW = rot_x(math.radians(-26))

def render_mark(f, size, ss=3, bg=False):
    S = size * ss
    p = f / N
    img = Image.new("RGBA", (S, S), tuple(int(c) for c in BG) + (255 if bg else 0,))
    d = ImageDraw.Draw(img)
    cx = cy = S / 2
    scale = S * 0.20
    obj = 0.60
    u = S / 700.0  # pixel unit relative to a 700px mark
    items = []

    # --- particle core ---
    w = shape_weights(p)
    P = sum(w[k] * SHAPES[k] for k in range(3)) * obj
    R = VIEW @ rot_y(math.radians(45) + p * 2 * math.pi * 0.25)
    W = P @ R.T
    for k in range(NDOTS):
        x, yv, z = W[k]
        depth = min(1.0, max(0.0, (z / (obj * 1.25) + 1) / 2))  # 0 far .. 1 near
        tw = 0.5 + 0.5 * math.sin(2 * math.pi * (2 * p + PHASE[k]))  # gentle twinkle
        r = (0.9 + 1.9 * depth) * u * (0.85 + 0.15 * tw)
        ink = 0.10 + 0.90 * depth ** 1.4
        col = BG + (INK - BG) * ink * (0.85 + 0.15 * tw)
        items.append((z, 0, (cx + x * scale, cy - yv * scale, r, col)))

    # --- orbits: X locked, spinning on long axis ---
    def ring(sign, direction):
        a = np.array([sign * math.sqrt(0.5), math.sqrt(0.5), 0.0])
        perp = np.array([-a[1], a[0], 0.0])
        spin = direction * p * 2 * math.pi
        b = math.cos(spin) * perp + math.sin(spin) * np.array([0, 0, 1.0])
        RA, RB = 1.95, 0.78
        pt = lambda t: a * RA * math.cos(t) + b * RB * math.sin(t)
        segs = 240
        lw = 1.8 * u
        for s_ in range(segs):
            t0 = 2 * math.pi * s_ / segs; t1 = 2 * math.pi * (s_ + 1) / segs
            p0, p1 = pt(t0), pt(t1)
            z = (p0[2] + p1[2]) / 2
            k = (z / RB + 1) / 2 if RB else 0.5
            col = RING_FAR + (RING_NEAR - RING_FAR) * max(0, min(1, k))
            items.append((z, 1, ((cx + p0[0] * scale, cy - p0[1] * scale),
                                 (cx + p1[0] * scale, cy - p1[1] * scale), lw, col)))
        # token-stream caret: a rounded bar + trailing token squares
        head = direction * p * 2 * math.pi * 2 + (0 if sign > 0 else math.pi)
        for j in range(6):
            t = head - direction * j * 0.11
            q = pt(t)
            z = q[2]
            k = (z / RB + 1) / 2
            bright = (0.55 + 0.45 * k)
            if j == 0:
                items.append((z + 0.01, 2, (cx + q[0] * scale, cy - q[1] * scale,
                                            6.5 * u, 17 * u, ACCENT * bright + BG * (1 - bright))))
            else:
                fade = (1 - j / 6) * bright
                sz = (6.0 - 0.7 * j) * u
                col = BG + (ACCENT - BG) * fade
                items.append((z, 3, (cx + q[0] * scale, cy - q[1] * scale, sz, col)))

    ring(+1, +1)
    ring(-1, -1)

    items.sort(key=lambda t: t[0])
    for _, kind, data in items:
        if kind == 0:
            x, yv, r, col = data
            d.ellipse([x - r, yv - r, x + r, yv + r], fill=tuple(int(c) for c in col) + (255,))
        elif kind == 1:
            a0, a1, lw, col = data
            c = tuple(int(v) for v in col) + (255,)
            d.line([a0, a1], fill=c, width=max(1, int(lw)))
            d.ellipse([a0[0] - lw / 2, a0[1] - lw / 2, a0[0] + lw / 2, a0[1] + lw / 2], fill=c)
        elif kind == 2:
            x, yv, wdt, hgt, col = data
            d.rounded_rectangle([x - wdt / 2, yv - hgt / 2, x + wdt / 2, yv + hgt / 2],
                                radius=wdt / 2, fill=tuple(int(c) for c in col) + (255,))
        else:
            x, yv, sz, col = data
            d.rounded_rectangle([x - sz / 2, yv - sz / 2, x + sz / 2, yv + sz / 2],
                                radius=sz * 0.25, fill=tuple(int(c) for c in col) + (255,))
    return img.resize((size, size), Image.LANCZOS)

def banner(f, W=1600, H=800):
    img = Image.new("RGB", (W, H), tuple(int(c) for c in BG))
    mark = render_mark(f, 700)
    img.paste(mark, (90, 50), mark)
    d = ImageDraw.Draw(img)
    d.text((800, 270), "oxyn", font=ImageFont.truetype(FONT, 190), fill=tuple(int(c) for c in INK))
    d.text((810, 510), "Agent UI components for Rust", font=ImageFont.truetype(FONT_R, 42), fill=(140, 136, 130))
    return img

if __name__ == "__main__":
    which = sys.argv[1] if len(sys.argv) > 1 else "all"
    jobs = {"banner": banner, "mark": lambda f: render_mark(f, 720, bg=True).convert("RGB")}
    for name, fn in jobs.items():
        if which not in ("all", name):
            continue
        os.makedirs(f"frames_{name}", exist_ok=True)
        for f in range(N):
            fn(f).save(f"frames_{name}/{f:03d}.png")
