---
title: Theming and tokens
description: One token file drives the web components, the desktop components and this site.
---

Oxyn's colours, type, radii and motion come from DTCG token files in `tokens/`: `oxyn.tokens.json` and `oxyn.motion.json`. A generator (`cargo run -p oxyn-tokens-gen`) writes them out as:

- `tokens/dist/oxyn.css`: CSS custom properties (`--oxyn-color-fg-default`, `--oxyn-dur-base`…) for Dioxus and the web;
- `oxyn-tokens`: a Rust theme for GPUI.

## Themes

Three themes share the same keys: **dark** (default), **light** and **hc-dark**. Switch with an attribute on any element:

```html
<html data-oxyn-theme="light">
```

## Colour rules

- **Chrome is monochrome**: black and greys in dark, white and greys in light.
- **Colour carries meaning only**: status (success, warning, danger, info), risk tiers, diffs, git and agent identity (monograms).
- Status is never colour alone: every coloured state also has a label, an icon or a shape.

The theme toggle in the top bar switches this whole site, the docs and every live preview together.
