# Studio (UX blueprint)

The Studio is where anyone clicks a component, tunes it, watches it run through real agent states and
exports exact code. References: Paper Shaders (inspector order, scrub sliders), Magnific's design editor
(export options), Astryx Playground (click-to-edit, global motion multiplier), Originkit (controls map
1:1 to props, presets in the URL), Libraries.dev Studio (free runtime, paid editor, "copy prompt").

## Layout
```
┌ Outline ───────┬──────────── Canvas ─────────────┬ Inspector ─────┐
│ parts tree     │  live component, click any part │ presets        │
│ states         │  device / light-dark / bg       │ state          │
│ saved presets  │                                 │ colour         │
│                │                                 │ shape          │
│                │  ▶ ━━━━●━━━━━ timeline / scrub  │ sliders        │
├────────────────┴─────────────────────────────────┤ motion         │
│ Code drawer: Rust · props · tokens · prompt · CLI│ seed · speed   │
└──────────────────────────────────────────────────┴────────────────┘
```

## Inspector order
1. Presets (built-in + saved) · Randomize (with locks) · Reset
2. **Agent state**: idle, composing, submitted, thinking, streaming, tool, approval, queued, steering, done, error, stalled
3. Colour (swatch + hex/OKLCH)
4. Shape options (variant, radius, layout)
5. Sliders, 2 decimals, drag the label to scrub
6. Motion: duration, easing / spring, stagger, reduced-motion preview
7. Transforms, speed multiplier, seed

## Timeline
Play / pause / scrub through **Simulate** scripts: "short answer", "long stream", "tool + approval",
"error + retry", "queue + steer". Each script is a fake event stream fed to `oxyn-core`.

## Export
Rust source (GPUI or Dioxus) · `oxyn add` command with preset · tokens JSON / CSS vars · agent prompt ·
PNG / SVG · MP4 / WebM / GIF · share link (full state in URL, short-hash fallback).

## Extras (later / Pro)
"Ask" box that edits only the selected part as an undoable diff · import a theme from CSS variables or a
screenshot · saved team libraries · Remix from every gallery card · Open in category harness.
