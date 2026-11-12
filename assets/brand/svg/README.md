# Oxyn logo v3 — vector files

Static vector versions of the finalized v3 mark (the animated originals are `../oxyn-mark.mp4` and `../oxyn-banner.gif`). The design is unchanged: this is the v3 renderer's own pose, written out as vectors.

## Files

| File | Use |
|---|---|
| `oxyn-mark.svg` | Full-colour mark, transparent background, 512×512 viewBox. Made for dark backgrounds. |
| `oxyn-mark-dark-bg.svg` | Full-colour mark on a `#0B0B0C` rounded square. Use it for app icons, avatars and social images. |
| `oxyn-mark-mono.svg` | One colour (`currentColor`), no opacity. For print, embossing, and one-colour UI. |
| `oxyn-mark-24.svg` | Simplified mark drawn on a 24 px grid. Use it from 20 to 63 px. |
| `oxyn-mark-16.svg` | Simplified mark drawn on a 16 px grid. Use it for favicons and at 16 to 19 px. |
| `oxyn-wordmark.svg` | Lowercase "oxyn", set as outlined paths with no `<text>` elements. |
| `oxyn-lockup.svg` | Mark plus wordmark side by side, spaced as in the banner. |
| `generate_svg.py` | Deterministic generator for all of the above. |

## Pose

Loop frame 0 (t = 0.00 s of the 5 s loop) is the rounded-cube state. Here both orbits lie open in the picture plane as a clean X, and the two ember carets sit at the top-right and bottom-right tips. This is also the frame the banner opens on.

## Colours

| Role | Hex |
|---|---|
| Core dots and wordmark (off-white) | `#ECE8E0` |
| Background | `#0B0B0C` |
| Orbits in the large mark | `#3D3935` |
| Orbits in the 16/24 px marks | `#5E5851` (a little lighter so a 1 px line stays visible) |
| Ember carets and token trail | `#E27042` |

In the colour files the depth of the core is shown with `fill-opacity` on `#ECE8E0`, the same ink ramp the renderer uses. On `#0B0B0C` the result matches the render pixel for pixel. Use the colour files only on dark backgrounds. On light backgrounds, use `oxyn-mark-mono.svg` in a dark colour.

## Clear space and minimum sizes

- **Clear space:** keep at least 1/8 of the mark's width free on every side. The 512 viewBox already includes about 6% padding. For the lockup, keep clear space at least equal to the wordmark's x-height.
- **Full marks** (`oxyn-mark`, `-dark-bg`, `-mono`): 64 px minimum. The dot texture reads best from 128 px up.
- **20–63 px:** use `oxyn-mark-24.svg`. It is sharpest at 24 and 48 px.
- **16–19 px:** use `oxyn-mark-16.svg`.
- **Lockup:** at least 320 px wide. Below that, use the 24 px mark next to the wordmark.

## Do

- Keep the files as they are and scale them evenly.
- Place the colour versions on `#0B0B0C` or another very dark neutral.
- Choose the right size file from the table above.

## Don't

- Don't recolour the orbits, the core or the carets, and don't swap which elements use the ember colour.
- Don't add effects: no glow, shadow, gradient, bevel or outline.
- Don't rotate, skew or stretch the mark, and don't rearrange the lockup.
- Don't set the wordmark in another typeface or retype it as live text.
- Don't scale the full mark below 64 px. Use the small versions instead.

## How they were generated

`generate_svg.py` imports `../render_logo.py`, the v3 procedural renderer, and uses its maths directly at loop phase p = 0:

- **Dots:** the face-grid dot field, the cube shape weights, the view rotation and projection, the depth-based radius and ink, and the twinkle. All 726 dots become `<circle>` elements, z-sorted. The dots behind the orbits are drawn first, then the orbits, then the carets, then the dots in front.
- **Orbits:** two ellipses (semi-axes 1.95 and 0.78) rotated ±45°, drawn as strokes 1.8 units wide.
- **Carets:** a rounded bar plus five fading token squares, at the renderer's exact positions.
- **Mono version:** drops the back-face dots (depth below 0.42) and the faintest trail tokens. Depth is shown by dot size only, with no opacity.
- **16/24 px versions:** a simplification of the same pose and projection. The core shows the cube's visible corners as 2×2 px dots: 7 corners at 16 px, plus the three edges meeting at the front corner at 24 px. The dots sit on a faint hexagon tint and are aligned to the pixel grid. The orbits are 1 px strokes kept clear of the core, and the carets are solid ember pills.
- **Wordmark:** the glyph outlines of Inter Display SemiBold, extracted with fontTools at the banner's size and baseline.
- **Coordinates:** rounded to 2 decimals, with no metadata.

To regenerate, run `python3 generate_svg.py /path/to/InterDisplay-SemiBold.ttf`. It needs numpy, pillow and fonttools. The output is byte-identical on every run.

## Font licence

The wordmark uses **Inter Display SemiBold** (Inter 4.0, © 2016 The Inter Project Authors). It is licensed under the **SIL Open Font License 1.1** (https://openfontlicense.org). The OFL allows the glyph outlines to be embedded as vector paths in a logo, including commercially. The font files themselves are not included here.
